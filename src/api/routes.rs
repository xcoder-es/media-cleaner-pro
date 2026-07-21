use axum::{
    Json, Router,
    extract::{Query, State},
    response::{Sse, sse::Event},
    routing::{get, post},
};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::Stream;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{
    api::models::*,
    state::{AppState, LogMessage, ProcessingStats, StageInfo, StageStatus, machine::StateMachine},
};

use crate::pipeline::run_streaming_pipeline;
use mc_core::PipelineConfig;

pub fn create_routes(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/api/status", get(get_status))
        .route("/api/start", post(start_job))
        .route("/api/control", post(control_job))
        .route("/api/progress", get(progress_stream))
        .route("/api/logs", get(get_logs))
        .route("/api/browse", get(browse_directory))
        .route("/api/openapi.json", get(serve_openapi))
        .route("/health", get(health_check))
        .with_state(state)
        .fallback(crate::frontend::serve_assets)
}

#[derive(OpenApi)]
#[openapi(
    paths(health_check, get_status, start_job, control_job, progress_stream, get_logs, browse_directory),
    components(schemas(
        StartJobRequest, JobResponse, StateResponse, ControlRequest,
        DirEntry, StageInfo, StageStatus, ProcessingStats, LogMessage, PipelineConfig,
    )),
    info(
        title = "MediaCleaner Pro API",
        description = "REST API for MediaCleaner Pro — perceptual duplicate image removal pipeline",
        version = "0.1.4-alpha",
        license(name = "MIT")
    ),
    tags(
        (name = "health", description = "Service health endpoints"),
        (name = "pipeline", description = "Image processing pipeline operations"),
        (name = "files", description = "File system browsing"),
    )
)]
struct ApiDoc;

async fn serve_openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[derive(serde::Serialize, ToSchema)]
struct DirEntry {
    name: String,
    path: String,
    is_dir: bool,
    image_count: usize,
}

#[utoipa::path(
    get,
    path = "/api/browse",
    tag = "files",
    params(
        ("path" = String, Query, description = "Directory path to browse"),
    ),
    responses(
        (status = 200, description = "Directory listing", body = Vec<DirEntry>),
    )
)]
async fn browse_directory(Query(params): Query<HashMap<String, String>>) -> Json<Vec<DirEntry>> {
    use std::path::Path;

    let current = params.get("path").map(|s| s.as_str()).unwrap_or("/");
    let dir = Path::new(current);

    if !dir.exists() || !dir.is_dir() {
        return Json(Vec::new());
    }

    let mut entries = Vec::new();
    let image_extensions = [
        "jpg", "jpeg", "png", "bmp", "webp", "gif", "tiff", "tif", "heic", "heif",
    ];

    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

            let image_count = if is_dir {
                std::fs::read_dir(entry.path())
                    .map(|rd| {
                        rd.flatten()
                            .filter(|e| {
                                e.file_type().map(|t| t.is_file()).unwrap_or(false)
                                    && e.path()
                                        .extension()
                                        .and_then(|e| e.to_str())
                                        .map(|e| {
                                            image_extensions.contains(&e.to_lowercase().as_str())
                                        })
                                        .unwrap_or(false)
                            })
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            };

            if is_dir {
                entries.push(DirEntry {
                    name,
                    path,
                    is_dir: true,
                    image_count,
                });
            }
        }
    }

    entries.sort_by_key(|a| a.name.to_lowercase());
    Json(entries)
}

#[utoipa::path(
    get,
    path = "/api/status",
    tag = "pipeline",
    responses(
        (status = 200, description = "Current pipeline state", body = StateResponse),
    )
)]
async fn get_status(State(state): State<Arc<RwLock<AppState>>>) -> Json<StateResponse> {
    let s = state.read().await;
    Json(StateResponse {
        stages: s.stages.clone(),
        stats: s.stats.clone(),
        is_running: s.is_running,
        is_paused: s.is_paused,
        log_messages: s.log_messages.clone(),
    })
}

#[utoipa::path(
    post,
    path = "/api/start",
    tag = "pipeline",
    request_body = StartJobRequest,
    responses(
        (status = 200, description = "Job started successfully", body = JobResponse),
    )
)]
async fn start_job(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<StartJobRequest>,
) -> Json<JobResponse> {
    let mut s = state.write().await;
    let job_id = Uuid::new_v4().to_string();

    // Update config with request params (keep existing values when empty)
    if !req.source_dir.is_empty() {
        s.config.source_dir = req.source_dir;
    }
    if !req.dest_dir.is_empty() {
        s.config.dest_dir = req.dest_dir;
    }
    if let Some(threshold) = req.hamming_threshold {
        s.config.hamming_threshold = threshold;
    }

    s.reset_cancel_token();
    StateMachine::start_job(&mut s, job_id.clone());

    // Spawn processing task
    let state_clone = Arc::clone(&state);
    let token = s.cancel_token.clone();
    tokio::spawn(async move {
        run_streaming_pipeline(state_clone, token).await;
    });

    Json(JobResponse {
        job_id,
        status: "started".to_string(),
        stages: s.stages.clone(),
        stats: s.stats.clone(),
        is_running: s.is_running,
        is_paused: s.is_paused,
    })
}

#[utoipa::path(
    post,
    path = "/api/control",
    tag = "pipeline",
    request_body = ControlRequest,
    responses(
        (status = 200, description = "Control action executed", body = serde_json::Value),
    )
)]
async fn control_job(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<ControlRequest>,
) -> Json<serde_json::Value> {
    let mut s = state.write().await;

    match req.action.as_str() {
        "pause" => StateMachine::pause_job(&mut s),
        "resume" => StateMachine::resume_job(&mut s),
        "cancel" => {
            s.cancel_token.cancel();
            StateMachine::cancel_job(&mut s);
        }
        _ => {}
    }

    Json(serde_json::json!({
        "success": true,
        "action": req.action,
        "is_running": s.is_running,
        "is_paused": s.is_paused,
    }))
}

#[utoipa::path(
    get,
    path = "/api/progress",
    tag = "pipeline",
    responses(
        (status = 200, description = "Server-Sent Events stream of StateResponse", content_type = "text/event-stream"),
    )
)]
async fn progress_stream(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let state_clone = Arc::clone(&state);

    let stream = async_stream::stream! {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let s = state_clone.read().await;
            let event = Event::default()
                .event("progress")
                .data(serde_json::to_string(&StateResponse {
                    stages: s.stages.clone(),
                    stats: s.stats.clone(),
                    is_running: s.is_running,
                    is_paused: s.is_paused,
                    log_messages: Vec::new(), // Don't stream all logs via SSE
                }).unwrap());
            yield Ok(event);
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    )
}

#[utoipa::path(
    get,
    path = "/api/logs",
    tag = "pipeline",
    responses(
        (status = 200, description = "Recent log messages", body = Vec<LogMessage>),
    )
)]
async fn get_logs(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<Vec<crate::state::LogMessage>> {
    let s = state.read().await;
    Json(s.log_messages.clone())
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = serde_json::Value),
    )
)]
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "mediacleaner-pro",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

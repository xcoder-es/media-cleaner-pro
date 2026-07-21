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

    StateMachine::start_job(&mut s, job_id.clone());

    // Spawn processing task
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        run_pipeline(state_clone).await;
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
        "cancel" => StateMachine::cancel_job(&mut s),
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

// The actual pipeline runner
pub(crate) async fn run_pipeline(state: Arc<RwLock<AppState>>) {
    use crate::processing::duplicate::DuplicateDetector;
    use crate::processing::stages::StageProcessor;
    use crate::processing::*;
    use std::collections::HashSet;
    use std::time::Instant;
    use walkdir::WalkDir;

    let (source_dir, _dest_dir, threshold) = {
        let s = state.read().await;
        (
            s.config.source_dir.clone(),
            s.config.dest_dir.clone(),
            s.config.hamming_threshold,
        )
    };

    let (file_system, exact_hasher, image_hasher, image_decoder) = {
        let s = state.read().await;
        (
            Arc::clone(&s.ctx.file_system),
            Arc::clone(&s.ctx.exact_hasher),
            Arc::clone(&s.ctx.image_hasher),
            Arc::clone(&s.ctx.image_decoder),
        )
    };

    // Pause/cancel check helper
    let check_control = || async {
        loop {
            let s = state.read().await;
            if !s.is_running {
                return false;
            }
            if !s.is_paused {
                return true;
            }
            drop(s);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    };

    // Stage 0: Scan + Hash + Exact Duplicate Removal (combined so progress shows immediately)
    {
        let mut s = state.write().await;
        s.stats.unique_count = 0;
        s.stats.duplicate_count = 0;
        s.stats.error_count = 0;
        StateMachine::start_stage(&mut s, 0);
    }

    // Collect paths + count files
    let mut image_paths = Vec::new();
    for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
        if !check_control().await {
            return;
        }
        if is_image_file(entry.path()) {
            image_paths.push(entry.path().to_path_buf());
        }
    }
    let total = image_paths.len();

    {
        let mut s = state.write().await;
        for stage in &mut s.stages {
            stage.total = total;
        }
    }

    let start_time = Instant::now();

    // Pre-compute metadata for all images (read once, hash, decode)
    let mut image_metas: Vec<Option<ImageMetadata>> = Vec::with_capacity(total);

    for (idx, path) in image_paths.iter().enumerate() {
        if !check_control().await {
            return;
        }

        let path_str = path.to_string_lossy().to_string();
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        {
            let mut s = state.write().await;
            s.stats.current_file = Some(filename.clone());
        }

        let data = match file_system.read_file(path).await {
            Ok(d) => d,
            Err(_) => {
                let mut s = state.write().await;
                s.stats.error_count += 1;
                image_metas.push(None);
                continue;
            }
        };

        let result = tokio::task::block_in_place(|| {
            let sha256 = exact_hasher.compute_sha256(&data)?;
            let dhash = image_hasher.compute_dhash(&data)?;
            let info = image_decoder.decode(&data)?;
            Ok::<_, anyhow::Error>((sha256, dhash, info.width, info.height))
        });

        let (sha256, dhash, width, height) = match result {
            Ok(r) => r,
            Err(_) => {
                let mut s = state.write().await;
                s.stats.error_count += 1;
                image_metas.push(None);
                continue;
            }
        };

        let format = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();

        image_metas.push(Some(ImageMetadata {
            path: path_str,
            filename,
            size_bytes: data.len() as u64,
            width,
            height,
            sha256,
            dhash: Some(dhash),
            format,
        }));

        let processed = idx + 1;
        if processed % 10 == 0 || processed == total {
            let mut s = state.write().await;
            StateMachine::update_stage_progress(&mut s, 0, processed, total);
            let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
            s.stats.speed = processed as f64 / elapsed;
            s.stats.eta_seconds =
                (total.saturating_sub(processed) as f64 / s.stats.speed.max(0.1)) as u64;
        }
    }

    // Collect results per image for file routing
    let mut results_map: std::collections::HashMap<String, Vec<StageResult>> =
        std::collections::HashMap::new();

    // Hash comparison (fast, remaining portion of stage 0)
    let mut seen_hashes = HashSet::new();

    for (processed, meta_opt) in image_metas.iter().enumerate() {
        if !check_control().await {
            return;
        }

        let meta = match meta_opt {
            Some(m) => m,
            None => continue,
        };

        {
            let mut s = state.write().await;
            s.stats.current_file = Some(meta.filename.clone());
        }

        let result = StageProcessor::exact_duplicate(meta, &seen_hashes);
        results_map
            .entry(meta.path.clone())
            .or_default()
            .push(result.clone());
        if result.passed {
            seen_hashes.insert(meta.sha256.clone());
        } else {
            let mut s = state.write().await;
            s.stats.duplicate_count += 1;
        }

        if processed % 100 == 0 || processed + 1 == total {
            let mut s = state.write().await;
            StateMachine::update_stage_progress(&mut s, 0, processed + 1, total);
            let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
            s.stats.speed = processed as f64 / elapsed;
            s.stats.eta_seconds =
                (total.saturating_sub(processed) as f64 / s.stats.speed.max(0.1)) as u64;
        }
    }

    {
        let mut s = state.write().await;
        StateMachine::complete_stage(&mut s, 0);
    }

    // Stage 1: Perceptual Duplicate Removal
    {
        let mut s = state.write().await;
        StateMachine::start_stage(&mut s, 1);
    }

    let mut detector = DuplicateDetector::new(threshold);
    let mut perceptual_processed = 0;

    for meta_opt in &image_metas {
        if !check_control().await {
            return;
        }

        let meta = match meta_opt {
            Some(m) => m,
            None => {
                perceptual_processed += 1;
                continue;
            }
        };

        let dhash = meta.dhash.unwrap_or(0);
        let duplicates = detector.add(meta.path.clone(), dhash);

        let result = StageProcessor::perceptual_duplicate(meta, &duplicates);
        results_map
            .entry(meta.path.clone())
            .or_default()
            .push(result);

        {
            let mut s = state.write().await;
            s.stats.current_dhash = Some(mc_core::format_dhash(dhash));
            if !duplicates.is_empty() {
                s.stats.duplicate_count += 1;
            } else {
                s.stats.unique_count += 1;
            }
        }

        perceptual_processed += 1;
        if perceptual_processed % 10 == 0 || perceptual_processed == total {
            let mut s = state.write().await;
            StateMachine::update_stage_progress(&mut s, 1, perceptual_processed, total);
        }
    }

    {
        let mut s = state.write().await;
        StateMachine::complete_stage(&mut s, 1);
    }

    // Stage 2: Tiny Image Detection
    let min_dims = {
        let s = state.read().await;
        (s.config.min_width, s.config.min_height)
    };

    for (path, result) in crate::processing::run_simple_stage(
        &state,
        2,
        &image_metas,
        total,
        &check_control,
        |meta| StageProcessor::tiny_image(meta, min_dims.0, min_dims.1),
    )
    .await
    {
        results_map.entry(path).or_default().push(result);
    }

    // Stage 3: Icon Detection
    for (path, result) in crate::processing::run_simple_stage(
        &state,
        3,
        &image_metas,
        total,
        &check_control,
        StageProcessor::icon_detection,
    )
    .await
    {
        results_map.entry(path).or_default().push(result);
    }

    // Stage 4: Thumbnail Detection
    for (path, result) in crate::processing::run_simple_stage(
        &state,
        4,
        &image_metas,
        total,
        &check_control,
        StageProcessor::thumbnail_detection,
    )
    .await
    {
        results_map.entry(path).or_default().push(result);
    }

    // Stage 5: Screenshot Detection
    for (path, result) in crate::processing::run_simple_stage(
        &state,
        5,
        &image_metas,
        total,
        &check_control,
        StageProcessor::screenshot_detection,
    )
    .await
    {
        results_map.entry(path).or_default().push(result);
    }

    // Stage 6: Wallpaper Detection
    for (path, result) in crate::processing::run_simple_stage(
        &state,
        6,
        &image_metas,
        total,
        &check_control,
        StageProcessor::wallpaper_detection,
    )
    .await
    {
        results_map.entry(path).or_default().push(result);
    }

    // Stage 7: Document Detection
    for (path, result) in crate::processing::run_simple_stage(
        &state,
        7,
        &image_metas,
        total,
        &check_control,
        StageProcessor::document_detection,
    )
    .await
    {
        results_map.entry(path).or_default().push(result);
    }

    // Stage 8: AI Classification
    for (path, result) in crate::processing::run_simple_stage(
        &state,
        8,
        &image_metas,
        total,
        &check_control,
        StageProcessor::ai_classification,
    )
    .await
    {
        results_map.entry(path).or_default().push(result);
    }

    // Stage 9: Quality Ranking
    for (path, result) in crate::processing::run_simple_stage(
        &state,
        9,
        &image_metas,
        total,
        &check_control,
        StageProcessor::quality_ranking,
    )
    .await
    {
        results_map.entry(path).or_default().push(result);
    }

    // Route files based on collected results
    crate::processing::route_files(&state, &image_metas, &results_map).await;

    {
        let mut s = state.write().await;
        StateMachine::complete_job(&mut s);
    }
}

use axum::{
    extract::{Multipart, State},
    response::{sse::Event, Sse},
    routing::{get, post},
    Json, Router,
};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::Stream;
use uuid::Uuid;

use crate::{
    api::models::*,
    state::{machine::StateMachine, AppState},
};

pub fn create_routes(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/api/status", get(get_status))
        .route("/api/start", post(start_job))
        .route("/api/control", post(control_job))
        .route("/api/progress", get(progress_stream))
        .route("/api/logs", get(get_logs))
        .route("/api/upload", post(upload_files))
        .route("/health", get(health_check))
        .with_state(state)
}

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

async fn start_job(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<StartJobRequest>,
) -> Json<JobResponse> {
    let mut s = state.write().await;
    let job_id = Uuid::new_v4().to_string();

    // Update config with request params
    s.config.source_dir = req.source_dir;
    s.config.dest_dir = req.dest_dir;
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

async fn get_logs(State(state): State<Arc<RwLock<AppState>>>) -> Json<Vec<crate::state::LogMessage>> {
    let s = state.read().await;
    Json(s.log_messages.clone())
}


async fn upload_files(
    State(state): State<Arc<RwLock<AppState>>>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    use std::path::Path;
    use tokio::io::AsyncWriteExt;

    let source_dir = {
        let s = state.read().await;
        s.config.source_dir.clone()
    };

    let dir = Path::new(&source_dir);
    tokio::fs::create_dir_all(dir).await.unwrap_or_default();

    let mut count = 0u32;
    let mut errors = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("file_{}", count));

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("{}: {}", file_name, e));
                continue;
            }
        };

        let dest = dir.join(&file_name);
        match tokio::fs::File::create(&dest).await {
            Ok(mut f) => {
                if f.write_all(&data).await.is_ok() {
                    count += 1;
                } else {
                    errors.push(format!("{}: write failed", file_name));
                }
            }
            Err(e) => errors.push(format!("{}: {}", file_name, e)),
        }
    }

    {
        let mut s = state.write().await;
        let job_id = Uuid::new_v4().to_string();
        StateMachine::start_job(&mut s, job_id);
    }

    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        run_pipeline(state_clone).await;
    });

    Json(serde_json::json!({
        "uploaded": count,
        "errors": errors,
        "source_dir": source_dir,
    }))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "mediacleaner-pro",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// The actual pipeline runner
async fn run_pipeline(state: Arc<RwLock<AppState>>) {
    use crate::processing::*;
    use crate::processing::hash::{compute_sha256, compute_dhash, format_dhash};
    use crate::processing::duplicate::DuplicateDetector;
    use crate::processing::stages::StageProcessor;
    use std::collections::HashSet;
    use walkdir::WalkDir;
    use image::ImageReader;
    use std::time::Instant;

    let (source_dir, _dest_dir, threshold) = {
        let s = state.read().await;
        (s.config.source_dir.clone(), s.config.dest_dir.clone(), s.config.hamming_threshold)
    };

    // Stage 0: Count files
    let mut image_paths = Vec::new();
    for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
        if is_image_file(entry.path()) {
            image_paths.push(entry.path().to_path_buf());
        }
    }
    let total = image_paths.len();

    {
        let mut s = state.write().await;
        s.stats.unique_count = 0;
        s.stats.duplicate_count = 0;
        s.stats.error_count = 0;
        for stage in &mut s.stages {
            stage.total = total;
        }
    }

    // Stage 1: Exact Duplicate Removal
    {
        let mut s = state.write().await;
        StateMachine::start_stage(&mut s, 0);
    }

    let mut seen_hashes = HashSet::new();
    let mut processed = 0;
    let start_time = Instant::now();

    for path in &image_paths {
        // Check pause/cancel
        {
            loop {
                let s = state.read().await;
                if !s.is_running {
                    return;
                }
                if !s.is_paused {
                    break;
                }
                drop(s);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        let path_str = path.to_string_lossy().to_string();
        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        // Update current file
        {
            let mut s = state.write().await;
            s.stats.current_file = Some(filename.clone());
        }

        // Load image and compute metadata
        let (width, height, format, sha256, dhash, size_bytes) = match tokio::task::block_in_place(|| {
            let img = ImageReader::open(path)?.decode();
            let sha256 = compute_sha256(path)?;
            let (width, height, dhash) = if let Ok(img) = img {
                let dhash = compute_dhash(&img);
                (img.width(), img.height(), Some(dhash))
            } else {
                (0, 0, None)
            };
            let format = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
                .to_string();
            let size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            Ok::<_, anyhow::Error>((width, height, format, sha256, dhash, size_bytes))
        }) {
            Ok((w, h, f, s, d, sb)) => (w, h, f, s, d, sb),
            Err(_) => {
                let mut s = state.write().await;
                s.stats.error_count += 1;
                continue;
            }
        };

        let meta = ImageMetadata {
            path: path_str.clone(),
            filename: filename.clone(),
            size_bytes,
            width,
            height,
            sha256: sha256.clone(),
            dhash,
            format,
        };

        // Stage 1: Exact duplicate
        let result1 = StageProcessor::exact_duplicate(&meta, &seen_hashes);
        if result1.passed {
            seen_hashes.insert(sha256);
        } else {
            let mut s = state.write().await;
            s.stats.duplicate_count += 1;
        }

        processed += 1;
        if processed % 10 == 0 || processed == total {
            let mut s = state.write().await;
            StateMachine::update_stage_progress(&mut s, 0, processed, total);
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                s.stats.speed = processed as f64 / elapsed;
                let remaining = total.saturating_sub(processed);
                s.stats.eta_seconds = (remaining as f64 / s.stats.speed.max(0.1)) as u64;
            }
        }
    }

    {
        let mut s = state.write().await;
        StateMachine::complete_stage(&mut s, 0);
    }

    // Stage 2: Perceptual Duplicate Removal
    {
        let mut s = state.write().await;
        StateMachine::start_stage(&mut s, 1);
    }

    let mut detector = DuplicateDetector::new(threshold);
    let mut perceptual_processed = 0;

    for path in &image_paths {
        {
            loop {
                let s = state.read().await;
                if !s.is_running {
                    return;
                }
                if !s.is_paused {
                    break;
                }
                drop(s);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        let path_str = path.to_string_lossy().to_string();
        let dhash = match tokio::task::block_in_place(|| {
            let img = ImageReader::open(path)?.decode()?;
            Ok::<_, anyhow::Error>(compute_dhash(&img))
        }) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let duplicates = detector.add(path_str, dhash);

        {
            let mut s = state.write().await;
            s.stats.current_dhash = Some(format_dhash(dhash));
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

    // Stage 3-10: Run remaining stages in batch
    for stage_idx in 2..10 {
        {
            let mut s = state.write().await;
            StateMachine::start_stage(&mut s, stage_idx);
        }

        let mut stage_processed = 0;
        for _path in &image_paths {
            {
                loop {
                    let s = state.read().await;
                    if !s.is_running {
                        return;
                    }
                    if !s.is_paused {
                        break;
                    }
                    drop(s);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }

            // For stages 3-10, we'd need to load metadata from DB or re-scan
            // For now, simulate progress
            stage_processed += 1;
            if stage_processed % 50 == 0 || stage_processed == total {
                let mut s = state.write().await;
                StateMachine::update_stage_progress(&mut s, stage_idx, stage_processed, total);
            }
        }

        {
            let mut s = state.write().await;
            StateMachine::complete_stage(&mut s, stage_idx);
        }
    }

    {
        let mut s = state.write().await;
        StateMachine::complete_job(&mut s);
    }
}

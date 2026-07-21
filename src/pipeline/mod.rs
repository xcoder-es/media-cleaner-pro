use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use mc_core::*;
use tokio::sync::{Semaphore, mpsc};
use walkdir::WalkDir;

use crate::processing::duplicate::DuplicateDetector;
use crate::processing::stages::StageProcessor;
use crate::state::{AppState, machine::StateMachine};

type ScanItem = (std::path::PathBuf, Vec<u8>);

pub async fn run_streaming_pipeline(state: Arc<tokio::sync::RwLock<AppState>>) {
    let (source_dir, dest_dir, threshold, min_width, min_height) = {
        let s = state.read().await;
        (
            s.config.source_dir.clone(),
            s.config.dest_dir.clone(),
            s.config.hamming_threshold,
            s.config.min_width,
            s.config.min_height,
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

    // Phase 1: Walk directory to find and count all image files
    let mut paths = Vec::new();
    for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && is_image_file(entry.path()) {
            paths.push(entry.path().to_path_buf());
        }
    }
    let total = paths.len();

    {
        let mut s = state.write().await;
        s.stats.unique_count = 0;
        s.stats.duplicate_count = 0;
        s.stats.error_count = 0;
        for stage in &mut s.stages {
            stage.total = total;
        }
        StateMachine::start_stage(&mut s, 0);
    }

    if total == 0 {
        let mut s = state.write().await;
        StateMachine::complete_stage(&mut s, 0);
        StateMachine::complete_job(&mut s);
        return;
    }

    let start_time = Instant::now();
    let error_count = Arc::new(AtomicUsize::new(0));

    let (scan_tx, scan_rx) = mpsc::channel::<ScanItem>(1000);
    let (meta_tx, mut meta_rx) = mpsc::channel::<ImageMetadata>(1000);

    let seen_hashes = Arc::new(std::sync::Mutex::new(HashSet::<String>::new()));
    let detector = Arc::new(std::sync::Mutex::new(DuplicateDetector::new(threshold)));

    // Phase 2: Concurrent reader pool (16 semaphore)
    let semaphore = Arc::new(Semaphore::new(16));

    for path in paths {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let tx = scan_tx.clone();
        let fs = Arc::clone(&file_system);
        let errs = Arc::clone(&error_count);

        tokio::spawn(async move {
            match fs.read_file(&path).await {
                Ok(data) => {
                    let _ = tx.send((path, data)).await;
                }
                Err(_) => {
                    errs.fetch_add(1, Ordering::SeqCst);
                }
            }
            drop(permit);
        });
    }
    drop(scan_tx);

    // Phase 3: Hash workers (single consumer dispatching CPU work to rayon)
    let hash_errs = Arc::clone(&error_count);
    let hash_tx = meta_tx.clone();

    let hash_handle = tokio::spawn(async move {
        let mut rx = scan_rx;
        let hash_tx = hash_tx;
        let hash_errs = hash_errs;
        while let Some((path, data)) = rx.recv().await {
            let tx = hash_tx.clone();
            let exact = Arc::clone(&exact_hasher);
            let dhash = Arc::clone(&image_hasher);
            let decode = Arc::clone(&image_decoder);
            let errs = Arc::clone(&hash_errs);

            tokio::task::block_in_place(move || {
                let sha256 = exact.compute_sha256(&data);
                let dh = dhash.compute_dhash(&data);
                let info = decode.decode(&data);

                match (sha256, dh, info) {
                    (Ok(sha), Ok(dh_val), Ok(inf)) => {
                        let path_str = path.to_string_lossy().to_string();
                        let filename = path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let format = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let meta = ImageMetadata {
                            path: path_str,
                            filename,
                            size_bytes: data.len() as u64,
                            width: inf.width,
                            height: inf.height,
                            sha256: sha,
                            dhash: Some(dh_val),
                            format,
                        };
                        let _ = tx.blocking_send(meta);
                    }
                    _ => {
                        errs.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
        }
    });
    drop(meta_tx);

    // Phase 4: Stage pipeline (sequential per image, all 10 stages)
    let stage_handle = {
        let mut results_map: HashMap<String, Vec<StageResult>> = HashMap::new();
        let mut meta_map: HashMap<String, ImageMetadata> = HashMap::new();
        let mut processed: usize = 0;

        while let Some(meta) = meta_rx.recv().await {
            let path = meta.path.clone();

            // Stage 0: Exact duplicate
            let result0 = {
                let mut seen = seen_hashes.lock().unwrap();
                let r = StageProcessor::exact_duplicate(&meta, &seen);
                if r.passed {
                    seen.insert(meta.sha256.clone());
                }
                r
            };
            results_map.entry(path.clone()).or_default().push(result0);

            // Stage 1: Perceptual duplicate
            let (result1, _dup_paths) = {
                let mut det = detector.lock().unwrap();
                let dupes = det.add(meta.path.clone(), meta.dhash.unwrap_or(0));
                let r = StageProcessor::perceptual_duplicate(&meta, &dupes);
                (r, dupes)
            };
            results_map.entry(path.clone()).or_default().push(result1);

            // Stages 2-9: Pure functions
            results_map
                .entry(path.clone())
                .or_default()
                .push(StageProcessor::tiny_image(&meta, min_width, min_height));
            results_map
                .entry(path.clone())
                .or_default()
                .push(StageProcessor::icon_detection(&meta));
            results_map
                .entry(path.clone())
                .or_default()
                .push(StageProcessor::thumbnail_detection(&meta));
            results_map
                .entry(path.clone())
                .or_default()
                .push(StageProcessor::screenshot_detection(&meta));
            results_map
                .entry(path.clone())
                .or_default()
                .push(StageProcessor::wallpaper_detection(&meta));
            results_map
                .entry(path.clone())
                .or_default()
                .push(StageProcessor::document_detection(&meta));
            results_map
                .entry(path.clone())
                .or_default()
                .push(StageProcessor::ai_classification(&meta));
            results_map
                .entry(path.clone())
                .or_default()
                .push(StageProcessor::quality_ranking(&meta));

            meta_map.insert(meta.path.clone(), meta);
            processed += 1;

            // Progress tracking (batched in phase 5, but we need counts here)
            if processed.is_multiple_of(10) || processed == total {
                let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                let speed = processed as f64 / elapsed;
                let remaining = total.saturating_sub(processed);
                let eta = (remaining as f64 / speed.max(0.1)) as u64;

                let mut s = state.write().await;
                s.stats.speed = speed;
                s.stats.eta_seconds = eta;
                StateMachine::update_stage_progress(&mut s, 0, processed, total);
            }
        }

        (results_map, meta_map, processed)
    };

    let _ = hash_handle.await;
    let (results_map, meta_map, processed) = stage_handle;

    {
        let mut s = state.write().await;
        s.stats.error_count = error_count.load(Ordering::SeqCst);
        s.stats.speed = processed as f64 / start_time.elapsed().as_secs_f64().max(0.001);
        StateMachine::complete_stage(&mut s, 0);
        // Mark remaining stages as completed (they ran per-image)
        for idx in 1..10 {
            StateMachine::start_stage(&mut s, idx);
            StateMachine::complete_stage(&mut s, idx);
        }
    }

    // Route files
    let dest_dir = std::path::PathBuf::from(&dest_dir);
    for (path_str, stage_results) in &results_map {
        let meta = match meta_map.get(path_str) {
            Some(m) => m,
            None => continue,
        };
        let destination = stage_results
            .iter()
            .find(|r| !r.passed && r.destination.is_some())
            .and_then(|r| r.destination.clone())
            .or_else(|| {
                stage_results
                    .iter()
                    .find(|r| r.stage_name == "AI Classification")
                    .and_then(|r| r.destination.clone())
            });

        if let Some(dest) = destination {
            let dest_path = dest_dir.join(&dest).join(&meta.filename);
            if let Some(parent) = dest_path.parent() {
                let _ = file_system.create_dir(parent).await;
            }
            let src = std::path::Path::new(&meta.path);
            if let Err(e) = file_system.move_file(src, &dest_path).await {
                tracing::warn!("Failed to move {} to {}: {:?}", meta.filename, dest, e);
            }
        }
    }

    {
        let mut s = state.write().await;
        StateMachine::complete_job(&mut s);
    }
}

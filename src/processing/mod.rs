pub mod duplicate;
pub mod hash;
pub mod stages;

pub use mc_core::{format_duration, is_image_file, ImageMetadata, StageResult};

use crate::state::machine::StateMachine;
use crate::state::AppState;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) async fn run_simple_stage<F, Fut>(
    state: &Arc<RwLock<AppState>>,
    stage_idx: usize,
    image_metas: &[Option<ImageMetadata>],
    total: usize,
    check_control: impl Fn() -> Fut,
    process: F,
) -> Vec<(String, StageResult)>
where
    F: Fn(&ImageMetadata) -> StageResult,
    Fut: Future<Output = bool>,
{
    let mut results = Vec::with_capacity(total);
    {
        let mut s = state.write().await;
        StateMachine::start_stage(&mut s, stage_idx);
    }

    let mut stage_processed = 0;
    for meta_opt in image_metas {
        if !check_control().await {
            return results;
        }

        let meta = match meta_opt {
            Some(m) => m,
            None => {
                stage_processed += 1;
                continue;
            }
        };

        {
            let mut s = state.write().await;
            s.stats.current_file = Some(meta.filename.clone());
        }

        let result = process(meta);
        results.push((meta.path.clone(), result));

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

    results
}

pub(crate) async fn route_files(
    state: &Arc<RwLock<AppState>>,
    image_metas: &[Option<ImageMetadata>],
    results_map: &std::collections::HashMap<String, Vec<StageResult>>,
) {
    let dest_dir = {
        let s = state.read().await;
        std::path::PathBuf::from(&s.config.dest_dir)
    };
    let file_system = {
        let s = state.read().await;
        Arc::clone(&s.ctx.file_system)
    };

    for meta_opt in image_metas {
        let meta = match meta_opt {
            Some(m) => m,
            None => continue,
        };

        let stage_results = match results_map.get(&meta.path) {
            Some(r) => r,
            None => continue,
        };

        // First failed stage (in priority order) determines destination
        let destination = stage_results
            .iter()
            .find(|r| !r.passed && r.destination.is_some())
            .and_then(|r| r.destination.clone())
            // If all stages passed, use AI classification destination
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
        // If no destination, file stays in source (unique kept file)
    }
}

pub mod hash;
pub mod duplicate;
pub mod stages;

pub use mc_core::{ImageMetadata, StageResult, is_image_file, format_duration};

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::state::AppState;
use crate::state::machine::StateMachine;
use std::future::Future;

pub(crate) async fn run_simple_stage<F, Fut>(
    state: &Arc<RwLock<AppState>>,
    stage_idx: usize,
    image_metas: &[Option<ImageMetadata>],
    total: usize,
    check_control: impl Fn() -> Fut,
    process: F,
) where
    F: Fn(&ImageMetadata) -> StageResult,
    Fut: Future<Output = bool>,
{
    {
        let mut s = state.write().await;
        StateMachine::start_stage(&mut s, stage_idx);
    }

    let mut stage_processed = 0;
    for meta_opt in image_metas {
        if !check_control().await {
            return;
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

        let _result = process(meta);

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

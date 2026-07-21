use super::{AppState, LogMessage, StageStatus};
use chrono::Utc;

pub struct StateMachine;

impl StateMachine {
    pub fn start_job(state: &mut AppState, job_id: String) {
        state.job_id = Some(job_id);
        state.is_running = true;
        state.is_paused = false;
        state.log_messages.clear();

        for stage in &mut state.stages {
            stage.status = StageStatus::Pending;
            stage.progress = 0.0;
            stage.processed = 0;
            stage.error = None;
        }

        Self::log(state, "system", "Job started".to_string());
    }

    pub fn start_stage(state: &mut AppState, stage_idx: usize) {
        let stage_name = state
            .stages
            .get(stage_idx)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        if let Some(stage) = state.stages.get_mut(stage_idx) {
            stage.status = StageStatus::Running;
            stage.started_at = Some(Utc::now().to_rfc3339());
        }
        Self::log(state, &stage_name, format!("Stage started: {}", stage_name));
    }

    pub fn update_stage_progress(
        state: &mut AppState,
        stage_idx: usize,
        processed: usize,
        total: usize,
    ) {
        if let Some(stage) = state.stages.get_mut(stage_idx) {
            stage.processed = processed;
            stage.total = total;
            if total > 0 {
                stage.progress = (processed as f64 / total as f64) * 100.0;
            }
        }
        if total > 0 {
            state.stats.speed = processed as f64 / (Utc::now().timestamp() as f64).max(1.0);
            let remaining = total.saturating_sub(processed);
            state.stats.eta_seconds = (remaining as f64 / state.stats.speed.max(0.1)) as u64;
        }
    }

    pub fn complete_stage(state: &mut AppState, stage_idx: usize) {
        let stage_name = state
            .stages
            .get(stage_idx)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        if let Some(stage) = state.stages.get_mut(stage_idx) {
            stage.status = StageStatus::Completed;
            stage.progress = 100.0;
            stage.completed_at = Some(Utc::now().to_rfc3339());
        }
        Self::log(
            state,
            &stage_name,
            format!("Stage completed: {}", stage_name),
        );
    }

    pub fn fail_stage(state: &mut AppState, stage_idx: usize, error: String) {
        let stage_name = state
            .stages
            .get(stage_idx)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        if let Some(stage) = state.stages.get_mut(stage_idx) {
            stage.status = StageStatus::Failed;
            stage.error = Some(error.clone());
        }
        Self::log(
            state,
            &stage_name,
            format!("Stage failed: {} - {}", stage_name, error),
        );
    }

    pub fn complete_job(state: &mut AppState) {
        state.is_running = false;
        Self::log(state, "system", "Job completed".to_string());
    }

    pub fn pause_job(state: &mut AppState) {
        state.is_paused = true;
        Self::log(state, "system", "Job paused".to_string());
    }

    pub fn resume_job(state: &mut AppState) {
        state.is_paused = false;
        Self::log(state, "system", "Job resumed".to_string());
    }

    pub fn cancel_job(state: &mut AppState) {
        state.is_running = false;
        state.is_paused = false;
        Self::log(state, "system", "Job cancelled".to_string());
    }

    fn log(state: &mut AppState, stage: &str, message: String) {
        state.log_messages.push(LogMessage {
            timestamp: Utc::now().to_rfc3339(),
            level: "INFO".to_string(),
            stage: stage.to_string(),
            message,
        });
        if state.log_messages.len() > 1000 {
            state.log_messages.remove(0);
        }
    }
}

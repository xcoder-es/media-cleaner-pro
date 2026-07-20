use serde::{Deserialize, Serialize};
use crate::state::{StageInfo, ProcessingStats, LogMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartJobRequest {
    pub source_dir: String,
    pub dest_dir: String,
    pub hamming_threshold: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub job_id: String,
    pub status: String,
    pub stages: Vec<StageInfo>,
    pub stats: ProcessingStats,
    pub is_running: bool,
    pub is_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateResponse {
    pub stages: Vec<StageInfo>,
    pub stats: ProcessingStats,
    pub is_running: bool,
    pub is_paused: bool,
    pub log_messages: Vec<LogMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRequest {
    pub action: String, // start, pause, resume, cancel
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub event_type: String,
    pub data: serde_json::Value,
}

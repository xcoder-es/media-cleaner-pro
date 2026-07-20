pub mod machine;
pub mod db;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInfo {
    pub name: String,
    pub description: String,
    pub status: StageStatus,
    pub progress: f64,
    pub processed: usize,
    pub total: usize,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub current_file: Option<String>,
    pub current_dhash: Option<String>,
    pub unique_count: usize,
    pub duplicate_count: usize,
    pub error_count: usize,
    pub speed: f64, // images per second
    pub eta_seconds: u64,
    pub memory_mb: u64,
    pub cpu_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub job_id: Option<String>,
    pub stages: Vec<StageInfo>,
    pub stats: ProcessingStats,
    pub is_running: bool,
    pub is_paused: bool,
    pub log_messages: Vec<LogMessage>,
    pub config: crate::config::Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub timestamp: String,
    pub level: String,
    pub stage: String,
    pub message: String,
}

impl AppState {
    pub fn new(config: crate::config::Config) -> Self {
        let stages = vec![
            StageInfo {
                name: "Exact Duplicate Removal".to_string(),
                description: "SHA-256 hash comparison for byte-identical files".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "Perceptual Duplicate Removal".to_string(),
                description: "dHash + Hamming distance for visually similar images".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "Tiny Image Detection".to_string(),
                description: "Detect and flag images below configurable resolution threshold".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "Icon Detection".to_string(),
                description: "Multi-factor scoring for application icons and UI elements".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "Thumbnail Detection".to_string(),
                description: "Size + filename pattern analysis for thumbnail images".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "Screenshot Detection".to_string(),
                description: "Monitor resolution matching + UI element heuristics".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "Wallpaper Detection".to_string(),
                description: "Ultra-wide + high resolution aspect ratio detection".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "Document Detection".to_string(),
                description: "Paper ratio + OCR heuristic analysis".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "AI Classification".to_string(),
                description: "20-category heuristic classification".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
            StageInfo {
                name: "Quality Ranking".to_string(),
                description: "Multi-factor quality score 0-100".to_string(),
                status: StageStatus::Pending,
                progress: 0.0,
                processed: 0,
                total: 0,
                started_at: None,
                completed_at: None,
                error: None,
            },
        ];

        AppState {
            job_id: None,
            stages,
            stats: ProcessingStats {
                current_file: None,
                current_dhash: None,
                unique_count: 0,
                duplicate_count: 0,
                error_count: 0,
                speed: 0.0,
                eta_seconds: 0,
                memory_mb: 0,
                cpu_percent: 0.0,
            },
            is_running: false,
            is_paused: false,
            log_messages: Vec::new(),
            config,
        }
    }
}

pub mod machine;
pub mod db;

pub use mc_core::{StageInfo, StageStatus, ProcessingStats, LogMessage};

use serde::{Deserialize, Serialize};

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

use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    pub dhash: Option<u64>,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    pub stage_name: String,
    pub passed: bool,
    pub destination: Option<String>,
    pub reason: Option<String>,
    pub score: Option<f64>,
    pub category: Option<String>,
}

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
    pub speed: f64,
    pub eta_seconds: u64,
    pub memory_mb: u64,
    pub cpu_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub timestamp: String,
    pub level: String,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub hamming_threshold: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub detect_icons: bool,
    pub detect_thumbnails: bool,
    pub detect_screenshots: bool,
    pub detect_wallpapers: bool,
    pub detect_documents: bool,
    pub classification_enabled: bool,
    pub quality_ranking_enabled: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            hamming_threshold: 4,
            min_width: 100,
            min_height: 100,
            detect_icons: true,
            detect_thumbnails: true,
            detect_screenshots: true,
            detect_wallpapers: true,
            detect_documents: true,
            classification_enabled: true,
            quality_ranking_enabled: true,
        }
    }
}

pub fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp" | "webp" | "gif" | "tiff" | "tif")
    } else {
        false
    }
}

pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

pub fn format_dhash(hash: u64) -> String {
    format!("{:016X}", hash)
}

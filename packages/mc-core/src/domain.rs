use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct JobId(pub String);

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for JobId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for JobId {
    fn from(s: String) -> Self {
        JobId(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct UserId(pub String);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for UserId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        UserId(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct TeamId(pub String);

impl fmt::Display for TeamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TeamId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TeamId {
    fn from(s: String) -> Self {
        TeamId(s)
    }
}

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
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct LogMessage {
    pub timestamp: String,
    pub level: String,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
        matches!(
            ext.as_str(),
            "jpg" | "jpeg" | "png" | "bmp" | "webp" | "gif" | "tiff" | "tif"
        )
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    NotSynced,
    Syncing,
    Synced,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub user_id: Option<String>,
    pub team_id: Option<String>,
    pub source_dir: String,
    pub dest_dir: String,
    pub config: PipelineConfig,
    pub stages: Vec<StageInfo>,
    pub stats: ProcessingStats,
    pub status: JobStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub sync_status: SyncStatus,
}

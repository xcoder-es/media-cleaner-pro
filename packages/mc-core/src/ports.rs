use std::path::Path;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::*;
use crate::error::DomainError;

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn list_dir(&self, path: &Path) -> Result<Vec<DirEntry>, DomainError>;
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, DomainError>;
    async fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), DomainError>;
    async fn create_dir(&self, path: &Path) -> Result<(), DomainError>;
    async fn delete_file(&self, path: &Path) -> Result<(), DomainError>;
    async fn copy_file(&self, src: &Path, dest: &Path) -> Result<(), DomainError>;
    async fn move_file(&self, src: &Path, dest: &Path) -> Result<(), DomainError>;
    async fn canonicalize(&self, path: &Path) -> Result<String, DomainError>;
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create_job(&self, job: &Job) -> Result<(), DomainError>;
    async fn get_job(&self, id: &str) -> Result<Option<Job>, DomainError>;
    async fn update_job(&self, job: &Job) -> Result<(), DomainError>;
    async fn list_jobs(&self, user_id: &str, limit: usize) -> Result<Vec<Job>, DomainError>;
    async fn delete_job(&self, id: &str) -> Result<(), DomainError>;
}

pub trait ImageHasher: Send + Sync {
    fn compute_dhash(&self, data: &[u8]) -> Result<u64, DomainError>;
    fn hamming_distance(&self, a: u64, b: u64) -> u32;
}

pub trait ExactHasher: Send + Sync {
    fn compute_sha256(&self, data: &[u8]) -> Result<String, DomainError>;
}

pub trait PipelineStage: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn process(&self, meta: &ImageMetadata, context: &StageContext) -> Result<StageResult, DomainError>;
}

pub struct StageContext {
    pub config: PipelineConfig,
    pub seen_hashes: std::collections::HashSet<String>,
    pub duplicate_paths: Vec<String>,
}

pub trait NotificationBus: Send + Sync {
    fn broadcast(&self, event: &PipelineEvent);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineEvent {
    StageStarted { stage: usize, name: String },
    StageProgress { stage: usize, processed: usize, total: usize },
    StageCompleted { stage: usize, results: StageResult },
    JobCompleted { job_id: String },
    JobPaused,
    JobResumed,
    JobCancelled,
    Error { stage: usize, message: String, path: Option<String> },
    Log { stage: String, level: String, message: String },
}

impl std::fmt::Display for PipelineEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineEvent::Log { message, .. } => write!(f, "{}", message),
            PipelineEvent::Error { message, .. } => write!(f, "ERROR: {}", message),
            _ => write!(f, "{:?}", self),
        }
    }
}



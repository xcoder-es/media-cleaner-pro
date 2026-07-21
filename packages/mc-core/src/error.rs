use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Sync conflict: {0}")]
    SyncConflict(String),

    #[error("Pipeline error in stage {stage}: {message}")]
    PipelineError { stage: usize, message: String },
}

impl From<DomainError> for String {
    fn from(e: DomainError) -> String {
        e.to_string()
    }
}

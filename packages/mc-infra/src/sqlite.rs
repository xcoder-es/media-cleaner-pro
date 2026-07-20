use std::path::Path;

use async_trait::async_trait;
use mc_core::{DomainError, JobRepository, Job, StageInfo, ProcessingStats};
use rusqlite::Connection;

pub struct SqliteJobRepo {
    conn: Connection,
}

impl SqliteJobRepo {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, DomainError> {
        let conn = Connection::open(path)
            .map_err(|e| DomainError::StorageError(format!("open db: {}", e)))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| DomainError::StorageError(format!("pragmas: {}", e)))?;

        Self::init_tables(&conn)?;

        Ok(SqliteJobRepo { conn })
    }

    fn init_tables(conn: &Connection) -> Result<(), DomainError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                source_dir TEXT NOT NULL,
                dest_dir TEXT NOT NULL,
                hamming_threshold INTEGER NOT NULL DEFAULT 4,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                path TEXT NOT NULL,
                filename TEXT NOT NULL,
                size_bytes INTEGER NOT NULL DEFAULT 0,
                width INTEGER NOT NULL DEFAULT 0,
                height INTEGER NOT NULL DEFAULT 0,
                sha256 TEXT NOT NULL DEFAULT '',
                dhash TEXT,
                stage_results TEXT,
                final_destination TEXT,
                quality_score REAL,
                category TEXT,
                processed_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (job_id) REFERENCES jobs(id)
            );

            CREATE TABLE IF NOT EXISTS duplicates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                original_id INTEGER NOT NULL,
                duplicate_id INTEGER NOT NULL,
                similarity_score REAL NOT NULL,
                detected_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (job_id) REFERENCES jobs(id),
                FOREIGN KEY (original_id) REFERENCES images(id),
                FOREIGN KEY (duplicate_id) REFERENCES images(id)
            );

            CREATE INDEX IF NOT EXISTS idx_images_job_id ON images(job_id);
            CREATE INDEX IF NOT EXISTS idx_images_dhash ON images(dhash);
            CREATE INDEX IF NOT EXISTS idx_images_sha256 ON images(sha256);"
        )
        .map_err(|e| DomainError::StorageError(format!("init tables: {}", e)))?;

        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

#[async_trait]
impl JobRepository for SqliteJobRepo {
    async fn create_job(&self, _job: &Job) -> Result<(), DomainError> {
        Err(DomainError::OperationFailed("not implemented".into()))
    }

    async fn get_job(&self, _id: &str) -> Result<Option<Job>, DomainError> {
        Err(DomainError::OperationFailed("not implemented".into()))
    }

    async fn update_job(&self, _job: &Job) -> Result<(), DomainError> {
        Err(DomainError::OperationFailed("not implemented".into()))
    }

    async fn list_jobs(&self, _user_id: &str, _limit: usize) -> Result<Vec<Job>, DomainError> {
        Err(DomainError::OperationFailed("not implemented".into()))
    }

    async fn delete_job(&self, _id: &str) -> Result<(), DomainError> {
        Err(DomainError::OperationFailed("not implemented".into()))
    }
}

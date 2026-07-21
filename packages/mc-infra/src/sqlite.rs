use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use mc_core::{DomainError, Job, JobId, JobRepository, TeamId, UserId};
use rusqlite::Connection;

pub struct SqliteJobRepo {
    conn: Mutex<Connection>,
}

impl SqliteJobRepo {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, DomainError> {
        let conn = Connection::open(path)
            .map_err(|e| DomainError::StorageError(format!("open db: {}", e)))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| DomainError::StorageError(format!("pragmas: {}", e)))?;

        Self::init_tables(&conn)?;

        Ok(SqliteJobRepo {
            conn: Mutex::new(conn),
        })
    }

    fn map_job_row(row: &rusqlite::Row) -> rusqlite::Result<Job> {
        // Returns rusqlite::Result because stmt.query_map requires it.
        // Non-rusqlite errors (serde, chrono) are converted via
        // ToSqlConversionFailure, then unwrapped to DomainError by callers.
        let id_str: String = row.get(0)?;
        let config_str: String = row.get(5)?;
        let stages_str: String = row.get(6)?;
        let stats_str: String = row.get(7)?;
        let status_str: String = row.get(8)?;
        let created_at_str: String = row.get(9)?;
        let sync_status_str: String = row.get(11)?;

        Ok(Job {
            id: JobId::from(id_str),
            user_id: row.get::<_, Option<String>>(1)?.map(UserId::from),
            team_id: row.get::<_, Option<String>>(2)?.map(TeamId::from),
            source_dir: PathBuf::from(row.get::<_, String>(3)?),
            dest_dir: PathBuf::from(row.get::<_, String>(4)?),
            config: serde_json::from_str(&config_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            stages: serde_json::from_str(&stages_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            stats: serde_json::from_str(&stats_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            status: serde_json::from_str(&status_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                .with_timezone(&chrono::Utc),
            completed_at: row
                .get::<_, Option<String>>(10)?
                .map(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                })
                .transpose()
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            sync_status: serde_json::from_str(&sync_status_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        })
    }

    fn init_tables(conn: &Connection) -> Result<(), DomainError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                team_id TEXT,
                source_dir TEXT NOT NULL,
                dest_dir TEXT NOT NULL,
                config TEXT NOT NULL DEFAULT '{}',
                stages TEXT NOT NULL DEFAULT '[]',
                stats TEXT NOT NULL DEFAULT '{}',
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                completed_at TEXT,
                sync_status TEXT NOT NULL DEFAULT 'not_synced'
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
                FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS duplicates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                original_id INTEGER NOT NULL,
                duplicate_id INTEGER NOT NULL,
                similarity_score REAL NOT NULL,
                detected_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
                FOREIGN KEY (original_id) REFERENCES images(id) ON DELETE CASCADE,
                FOREIGN KEY (duplicate_id) REFERENCES images(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_images_job_id ON images(job_id);
            CREATE INDEX IF NOT EXISTS idx_images_dhash ON images(dhash);
            CREATE INDEX IF NOT EXISTS idx_images_sha256 ON images(sha256);
            CREATE INDEX IF NOT EXISTS idx_jobs_user_id ON jobs(user_id);",
        )
        .map_err(|e| DomainError::StorageError(format!("init tables: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl JobRepository for SqliteJobRepo {
    async fn create_job(&self, job: &Job) -> Result<(), DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::StorageError(format!("lock error: {}", e)))?;

        let config = serde_json::to_string(&job.config)
            .map_err(|e| DomainError::StorageError(format!("serialize config: {}", e)))?;
        let stages = serde_json::to_string(&job.stages)
            .map_err(|e| DomainError::StorageError(format!("serialize stages: {}", e)))?;
        let stats = serde_json::to_string(&job.stats)
            .map_err(|e| DomainError::StorageError(format!("serialize stats: {}", e)))?;

        conn.execute(
            "INSERT INTO jobs (id, user_id, team_id, source_dir, dest_dir, config, stages, stats, status, created_at, completed_at, sync_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                job.id.as_ref(),
                job.user_id.as_ref().map(|u| u.as_ref()),
                job.team_id.as_ref().map(|t| t.as_ref()),
                job.source_dir.to_string_lossy(),
                job.dest_dir.to_string_lossy(),
                config,
                stages,
                stats,
                serde_json::to_string(&job.status)
                    .map_err(|e| DomainError::StorageError(format!("serialize status: {}", e)))?,
                job.created_at.to_rfc3339(),
                job.completed_at.map(|dt| dt.to_rfc3339()),
                serde_json::to_string(&job.sync_status)
                    .map_err(|e| DomainError::StorageError(format!("serialize sync_status: {}", e)))?,
            ],
        )
        .map_err(|e| DomainError::StorageError(format!("insert job: {}", e)))?;

        Ok(())
    }

    async fn get_job(&self, id: &JobId) -> Result<Option<Job>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::StorageError(format!("lock error: {}", e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, team_id, source_dir, dest_dir, config, stages, stats, status, created_at, completed_at, sync_status
                 FROM jobs WHERE id = ?1",
            )
            .map_err(|e| DomainError::StorageError(format!("prepare get_job: {}", e)))?;

        let result = stmt.query_row(rusqlite::params![id.as_ref()], Self::map_job_row);

        match result {
            Ok(job) => Ok(Some(job)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::StorageError(format!("get_job: {}", e))),
        }
    }

    async fn update_job(&self, job: &Job) -> Result<(), DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::StorageError(format!("lock error: {}", e)))?;

        let config = serde_json::to_string(&job.config)
            .map_err(|e| DomainError::StorageError(format!("serialize config: {}", e)))?;
        let stages = serde_json::to_string(&job.stages)
            .map_err(|e| DomainError::StorageError(format!("serialize stages: {}", e)))?;
        let stats = serde_json::to_string(&job.stats)
            .map_err(|e| DomainError::StorageError(format!("serialize stats: {}", e)))?;

        conn.execute(
            "UPDATE jobs SET
                user_id = ?1, team_id = ?2, source_dir = ?3, dest_dir = ?4,
                config = ?5, stages = ?6, stats = ?7, status = ?8,
                created_at = ?9, completed_at = ?10, sync_status = ?11
             WHERE id = ?12",
            rusqlite::params![
                job.user_id.as_ref().map(|u| u.as_ref()),
                job.team_id.as_ref().map(|t| t.as_ref()),
                job.source_dir.to_string_lossy(),
                job.dest_dir.to_string_lossy(),
                config,
                stages,
                stats,
                serde_json::to_string(&job.status)
                    .map_err(|e| DomainError::StorageError(format!("serialize status: {}", e)))?,
                job.created_at.to_rfc3339(),
                job.completed_at.map(|dt| dt.to_rfc3339()),
                serde_json::to_string(&job.sync_status).map_err(|e| DomainError::StorageError(
                    format!("serialize sync_status: {}", e)
                ))?,
                job.id.as_ref(),
            ],
        )
        .map_err(|e| DomainError::StorageError(format!("update job: {}", e)))?;

        Ok(())
    }

    async fn list_jobs(
        &self,
        user_id: Option<&UserId>,
        limit: usize,
    ) -> Result<Vec<Job>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::StorageError(format!("lock error: {}", e)))?;

        let limit = limit as i64;

        let collect_jobs =
            |rows: Vec<Result<Job, rusqlite::Error>>| -> Result<Vec<Job>, DomainError> {
                rows.into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| DomainError::StorageError(format!("read job row: {}", e)))
            };

        match user_id {
            None => {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, user_id, team_id, source_dir, dest_dir, config, stages, stats, status, created_at, completed_at, sync_status
                         FROM jobs ORDER BY created_at DESC LIMIT ?1",
                    )
                    .map_err(|e| DomainError::StorageError(format!("prepare list_jobs: {}", e)))?;
                let rows: Vec<Result<Job, _>> = stmt
                    .query_map(rusqlite::params![limit], Self::map_job_row)
                    .map_err(|e| DomainError::StorageError(format!("query list_jobs: {}", e)))?
                    .collect();
                collect_jobs(rows)
            }
            Some(uid) => {
                let uid_str = uid.as_ref();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, user_id, team_id, source_dir, dest_dir, config, stages, stats, status, created_at, completed_at, sync_status
                         FROM jobs WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2",
                    )
                    .map_err(|e| DomainError::StorageError(format!("prepare list_jobs: {}", e)))?;
                let rows: Vec<Result<Job, _>> = stmt
                    .query_map(rusqlite::params![uid_str, limit], Self::map_job_row)
                    .map_err(|e| DomainError::StorageError(format!("query list_jobs: {}", e)))?
                    .collect();
                collect_jobs(rows)
            }
        }
    }

    async fn delete_job(&self, id: &JobId) -> Result<(), DomainError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::StorageError(format!("lock error: {}", e)))?;

        let tx = conn
            .transaction()
            .map_err(|e| DomainError::StorageError(format!("begin tx: {}", e)))?;

        let id_str = id.as_ref();
        tx.execute(
            "DELETE FROM duplicates WHERE job_id = ?1",
            rusqlite::params![id_str],
        )
        .map_err(|e| DomainError::StorageError(format!("delete duplicates: {}", e)))?;

        tx.execute(
            "DELETE FROM images WHERE job_id = ?1",
            rusqlite::params![id_str],
        )
        .map_err(|e| DomainError::StorageError(format!("delete images: {}", e)))?;

        tx.execute("DELETE FROM jobs WHERE id = ?1", rusqlite::params![id_str])
            .map_err(|e| DomainError::StorageError(format!("delete job: {}", e)))?;

        tx.commit()
            .map_err(|e| DomainError::StorageError(format!("commit tx: {}", e)))?;

        Ok(())
    }

    async fn query_by_team(&self, team_id: &TeamId) -> Result<Vec<Job>, DomainError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::StorageError(format!("lock error: {}", e)))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, team_id, source_dir, dest_dir, config, stages, stats, status, created_at, completed_at, sync_status
                 FROM jobs WHERE team_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| DomainError::StorageError(format!("prepare query_by_team: {}", e)))?;

        let rows: Vec<Result<Job, _>> = stmt
            .query_map(rusqlite::params![team_id.as_ref()], Self::map_job_row)
            .map_err(|e| DomainError::StorageError(format!("query query_by_team: {}", e)))?
            .collect();

        rows.into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DomainError::StorageError(format!("read job row: {}", e)))
    }
}

use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                source_dir TEXT NOT NULL,
                dest_dir TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                completed_at TEXT
            );
            CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL,
                path TEXT NOT NULL,
                filename TEXT NOT NULL,
                size_bytes INTEGER,
                width INTEGER,
                height INTEGER,
                sha256 TEXT,
                dhash TEXT,
                stage_results TEXT,
                final_destination TEXT,
                quality_score REAL,
                category TEXT,
                processed_at TEXT,
                FOREIGN KEY (job_id) REFERENCES jobs(id)
            );
            CREATE TABLE IF NOT EXISTS duplicates (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL,
                original_id TEXT NOT NULL,
                duplicate_id TEXT NOT NULL,
                similarity_score REAL,
                detected_at TEXT NOT NULL,
                FOREIGN KEY (job_id) REFERENCES jobs(id)
            );
            CREATE INDEX IF NOT EXISTS idx_images_job ON images(job_id);
            CREATE INDEX IF NOT EXISTS idx_images_dhash ON images(dhash);
            CREATE INDEX IF NOT EXISTS idx_images_sha256 ON images(sha256);"
        )?;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

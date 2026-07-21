use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub source_dir: String,
    pub dest_dir: String,
    pub hamming_threshold: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub worker_threads: usize,
    pub db_path: String,
    pub temporal_host: Option<String>,
    pub temporal_namespace: String,
    pub temporal_task_queue: String,
    pub supabase_url: Option<String>,
    pub supabase_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Config {
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            source_dir: env::var("SOURCE_DIR").unwrap_or_else(|_| "./data/source".to_string()),
            dest_dir: env::var("DEST_DIR").unwrap_or_else(|_| "./data/output".to_string()),
            hamming_threshold: env::var("HAMMING_THRESHOLD")
                .unwrap_or_else(|_| "4".to_string())
                .parse()?,
            min_width: env::var("MIN_WIDTH")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            min_height: env::var("MIN_HEIGHT")
                .unwrap_or_else(|_| "100".to_string())
                .parse()?,
            worker_threads: env::var("WORKER_THREADS")
                .unwrap_or_else(|_| "0".to_string())
                .parse()?,
            db_path: env::var("DB_PATH").unwrap_or_else(|_| "./mediacleaner.db".to_string()),
            temporal_host: env::var("TEMPORAL_HOST").ok(),
            temporal_namespace: env::var("TEMPORAL_NAMESPACE")
                .unwrap_or_else(|_| "default".to_string()),
            temporal_task_queue: env::var("TEMPORAL_TASK_QUEUE")
                .unwrap_or_else(|_| "mediacleaner".to_string()),
            supabase_url: env::var("SUPABASE_URL").ok(),
            supabase_key: env::var("SUPABASE_KEY").ok(),
        })
    }
}

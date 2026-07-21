use async_trait::async_trait;
use mc_core::{DomainError, FileScanner};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct NativeFileScanner;

#[async_trait]
impl FileScanner for NativeFileScanner {
    async fn scan(&self, path: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, DomainError> {
        let mut results = Vec::new();
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext.to_lowercase().as_str()) {
                        results.push(entry.path().to_path_buf());
                    }
                }
            }
        }
        Ok(results)
    }
}

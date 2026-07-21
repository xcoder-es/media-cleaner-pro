use std::path::{Path, PathBuf};

use async_trait::async_trait;
use mc_core::{DirEntry, DomainError, FileSystem};

pub struct NativeFileSystem {
    root: PathBuf,
}

impl NativeFileSystem {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        NativeFileSystem { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[async_trait]
impl FileSystem for NativeFileSystem {
    async fn list_dir(&self, path: &Path) -> Result<Vec<DirEntry>, DomainError> {
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };

        let mut entries = Vec::new();
        let mut rd = tokio::fs::read_dir(&full_path)
            .await
            .map_err(|e| DomainError::OperationFailed(format!("read_dir: {}", e)))?;

        while let Some(entry) = rd
            .next_entry()
            .await
            .map_err(|e| DomainError::OperationFailed(format!("next_entry: {}", e)))?
        {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            let size = if !is_dir {
                entry.metadata().await.map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            entries.push(DirEntry {
                name,
                path,
                is_dir,
                size,
            });
        }

        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(entries)
    }

    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, DomainError> {
        let data = tokio::fs::read(path).await.map_err(|e| {
            DomainError::OperationFailed(format!("read_file {}: {}", path.display(), e))
        })?;
        Ok(data)
    }

    async fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), DomainError> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                DomainError::OperationFailed(format!("create_dir {}: {}", parent.display(), e))
            })?;
        }
        tokio::fs::write(path, data).await.map_err(|e| {
            DomainError::OperationFailed(format!("write_file {}: {}", path.display(), e))
        })?;
        Ok(())
    }

    async fn create_dir(&self, path: &Path) -> Result<(), DomainError> {
        tokio::fs::create_dir_all(path).await.map_err(|e| {
            DomainError::OperationFailed(format!("create_dir {}: {}", path.display(), e))
        })?;
        Ok(())
    }

    async fn delete_file(&self, path: &Path) -> Result<(), DomainError> {
        tokio::fs::remove_file(path).await.map_err(|e| {
            DomainError::OperationFailed(format!("delete_file {}: {}", path.display(), e))
        })?;
        Ok(())
    }

    async fn copy_file(&self, src: &Path, dest: &Path) -> Result<(), DomainError> {
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                DomainError::OperationFailed(format!("create_dir {}: {}", parent.display(), e))
            })?;
        }
        tokio::fs::copy(src, dest).await.map_err(|e| {
            DomainError::OperationFailed(format!(
                "copy {} -> {}: {}",
                src.display(),
                dest.display(),
                e
            ))
        })?;
        Ok(())
    }

    async fn move_file(&self, src: &Path, dest: &Path) -> Result<(), DomainError> {
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                DomainError::OperationFailed(format!("create_dir {}: {}", parent.display(), e))
            })?;
        }
        tokio::fs::rename(src, dest).await.map_err(|e| {
            DomainError::OperationFailed(format!(
                "move {} -> {}: {}",
                src.display(),
                dest.display(),
                e
            ))
        })?;
        Ok(())
    }

    async fn canonicalize(&self, path: &Path) -> Result<String, DomainError> {
        let canonical = tokio::fs::canonicalize(path).await.map_err(|e| {
            DomainError::OperationFailed(format!("canonicalize {}: {}", path.display(), e))
        })?;
        Ok(canonical.to_string_lossy().to_string())
    }
}

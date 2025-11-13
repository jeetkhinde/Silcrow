//! Filesystem storage backend for ISR cache

use crate::cache::CachedPage;
use crate::config::FilesystemConfig;
use crate::storage::Storage;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use walkdir::WalkDir;

/// Filesystem storage backend
///
/// Stores cached pages as JSON files on disk.
/// Persistent across restarts, suitable for single-instance deployments.
#[derive(Clone)]
pub struct FilesystemStorage {
    config: FilesystemConfig,
}

impl FilesystemStorage {
    /// Create a new filesystem storage backend
    pub async fn new(config: FilesystemConfig) -> Result<Self> {
        // Create cache directory if it doesn't exist
        fs::create_dir_all(&config.path)
            .await
            .context("Failed to create cache directory")?;

        Ok(Self { config })
    }

    /// Get the file path for a cache key
    fn key_to_path(&self, key: &str) -> PathBuf {
        // Sanitize key to make it filesystem-safe
        let safe_key = key
            .replace('/', "_")
            .replace('\\', "_")
            .replace(':', "_");

        self.config.path.join(format!("{}.json", safe_key))
    }

    /// Get cache directory size in bytes
    pub async fn total_size_bytes(&self) -> Result<u64> {
        let mut total = 0u64;

        for entry in WalkDir::new(&self.config.path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total += metadata.len();
                }
            }
        }

        Ok(total)
    }

    /// Check if cache size exceeds maximum
    pub async fn is_over_limit(&self) -> Result<bool> {
        let total_bytes = self.total_size_bytes().await?;
        let max_bytes = self.config.max_size_mb * 1024 * 1024;

        Ok(total_bytes > max_bytes)
    }

    /// Evict least recently used entries until under limit
    pub async fn evict_if_needed(&self) -> Result<()> {
        if !self.is_over_limit().await? {
            return Ok(());
        }

        // Get all entries with their access times
        let mut entries: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        for entry in WalkDir::new(&self.config.path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(accessed) = metadata.accessed() {
                        entries.push((entry.path().to_path_buf(), accessed));
                    }
                }
            }
        }

        // Sort by access time (oldest first)
        entries.sort_by_key(|(_, time)| *time);

        // Delete oldest entries until under limit
        for (path, _) in entries.iter() {
            fs::remove_file(path).await.ok();

            if !self.is_over_limit().await? {
                break;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Storage for FilesystemStorage {
    async fn get(&self, key: &str) -> Result<Option<CachedPage>> {
        let path = self.key_to_path(key);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .await
            .context("Failed to read cache file")?;

        let page: CachedPage = serde_json::from_str(&content)
            .context("Failed to deserialize cached page")?;

        Ok(Some(page))
    }

    async fn set(&self, key: &str, page: CachedPage) -> Result<()> {
        let path = self.key_to_path(key);

        let json = serde_json::to_string_pretty(&page)
            .context("Failed to serialize page")?;

        fs::write(&path, json)
            .await
            .context("Failed to write cache file")?;

        // Evict old entries if over limit
        self.evict_if_needed().await.ok();

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.key_to_path(key);

        if path.exists() {
            fs::remove_file(&path)
                .await
                .context("Failed to delete cache file")?;
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.key_to_path(key);
        Ok(path.exists())
    }

    async fn clear(&self) -> Result<()> {
        // Remove all files in cache directory
        let mut entries = fs::read_dir(&self.config.path)
            .await
            .context("Failed to read cache directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                fs::remove_file(&path).await.ok();
            }
        }

        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut entries = fs::read_dir(&self.config.path)
            .await
            .context("Failed to read cache directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                if let Some(file_name) = path.file_stem() {
                    if let Some(name_str) = file_name.to_str() {
                        // Reverse the sanitization
                        let key = name_str.to_string();
                        keys.push(key);
                    }
                }
            }
        }

        Ok(keys)
    }

    fn name(&self) -> &'static str {
        "filesystem"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_filesystem_storage_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig {
            path: temp_dir.path().to_path_buf(),
            max_size_mb: 100,
        };

        let storage = FilesystemStorage::new(config).await.unwrap();
        let page = CachedPage::new("test content".to_string(), Duration::from_secs(60));

        // Set
        storage.set("test-key", page.clone()).await.unwrap();

        // Get
        let retrieved = storage.get("test-key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().html, "test content");

        // Exists
        assert!(storage.exists("test-key").await.unwrap());

        // Delete
        storage.delete("test-key").await.unwrap();
        assert!(!storage.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_filesystem_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config = FilesystemConfig {
            path: temp_dir.path().to_path_buf(),
            max_size_mb: 100,
        };

        // Create storage and set a value
        {
            let storage = FilesystemStorage::new(config.clone()).await.unwrap();
            storage.set("persistent-key", CachedPage::new("persistent".to_string(), Duration::from_secs(60))).await.unwrap();
        }

        // Create new storage instance (simulating restart)
        {
            let storage = FilesystemStorage::new(config).await.unwrap();
            let retrieved = storage.get("persistent-key").await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().html, "persistent");
        }
    }
}

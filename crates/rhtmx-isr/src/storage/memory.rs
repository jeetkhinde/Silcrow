//! In-memory storage backend for ISR cache

use crate::cache::CachedPage;
use crate::storage::Storage;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory storage backend
///
/// Stores cached pages in memory using a HashMap.
/// Fast but non-persistent - cache is lost on restart.
#[derive(Clone)]
pub struct MemoryStorage {
    cache: Arc<RwLock<HashMap<String, CachedPage>>>,
}

impl MemoryStorage {
    /// Create a new memory storage backend
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cache size (number of entries)
    pub async fn size(&self) -> usize {
        self.cache.read().await.len()
    }

    /// Get total bytes stored
    pub async fn total_bytes(&self) -> usize {
        self.cache
            .read()
            .await
            .values()
            .map(|page| page.html.len())
            .sum()
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<CachedPage>> {
        let cache = self.cache.read().await;
        Ok(cache.get(key).cloned())
    }

    async fn set(&self, key: &str, page: CachedPage) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), page);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let cache = self.cache.read().await;
        Ok(cache.contains_key(key))
    }

    async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.clear();
        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>> {
        let cache = self.cache.read().await;
        Ok(cache.keys().cloned().collect())
    }

    fn name(&self) -> &'static str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_memory_storage_basic() {
        let storage = MemoryStorage::new();
        let page = CachedPage::new("test content".to_string(), Duration::from_secs(60));

        // Set
        storage.set("test-key", page.clone()).await.unwrap();

        // Get
        let retrieved = storage.get("test-key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().html, "test content");

        // Exists
        assert!(storage.exists("test-key").await.unwrap());
        assert!(!storage.exists("nonexistent").await.unwrap());

        // Delete
        storage.delete("test-key").await.unwrap();
        assert!(!storage.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_storage_clear() {
        let storage = MemoryStorage::new();

        storage.set("key1", CachedPage::new("content1".to_string(), Duration::from_secs(60))).await.unwrap();
        storage.set("key2", CachedPage::new("content2".to_string(), Duration::from_secs(60))).await.unwrap();

        assert_eq!(storage.size().await, 2);

        storage.clear().await.unwrap();

        assert_eq!(storage.size().await, 0);
    }

    #[tokio::test]
    async fn test_memory_storage_keys() {
        let storage = MemoryStorage::new();

        storage.set("key1", CachedPage::new("content1".to_string(), Duration::from_secs(60))).await.unwrap();
        storage.set("key2", CachedPage::new("content2".to_string(), Duration::from_secs(60))).await.unwrap();

        let keys = storage.keys().await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }
}

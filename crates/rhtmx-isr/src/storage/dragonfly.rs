//! Dragonfly (Redis-compatible) storage backend for ISR cache

use crate::cache::CachedPage;
use crate::config::DragonflyConfig;
use crate::storage::Storage;
use anyhow::{Context, Result};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};

/// Dragonfly storage backend
///
/// Stores cached pages in Dragonfly (or Redis).
/// Fast, distributed, and persistent.
/// Dragonfly is 25x faster than Redis with lower memory usage.
#[derive(Clone)]
pub struct DragonflyStorage {
    client: Client,
    manager: ConnectionManager,
    config: DragonflyConfig,
}

impl DragonflyStorage {
    /// Create a new Dragonfly storage backend
    pub async fn new(config: DragonflyConfig) -> Result<Self> {
        let client = Client::open(config.url.as_str())
            .context("Failed to create Redis/Dragonfly client")?;

        let manager = ConnectionManager::new(client.clone())
            .await
            .context("Failed to create connection manager")?;

        Ok(Self {
            client,
            manager,
            config,
        })
    }

    /// Get the full key with prefix
    fn full_key(&self, key: &str) -> String {
        format!("{}{}", self.config.key_prefix, key)
    }

    /// Test connection to Dragonfly/Redis
    pub async fn ping(&self) -> Result<bool> {
        let mut conn = self.manager.clone();
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .context("Failed to ping Redis/Dragonfly")?;

        Ok(pong == "PONG")
    }

    /// Get info about the Redis/Dragonfly server
    pub async fn info(&self) -> Result<String> {
        let mut conn = self.manager.clone();
        redis::cmd("INFO")
            .query_async(&mut conn)
            .await
            .context("Failed to get server info")
    }

    /// Get database size (number of keys)
    pub async fn dbsize(&self) -> Result<usize> {
        let mut conn = self.manager.clone();
        redis::cmd("DBSIZE")
            .query_async(&mut conn)
            .await
            .context("Failed to get database size")
    }
}

#[async_trait]
impl Storage for DragonflyStorage {
    async fn get(&self, key: &str) -> Result<Option<CachedPage>> {
        let full_key = self.full_key(key);
        let mut conn = self.manager.clone();

        let json: Option<String> = conn
            .get(&full_key)
            .await
            .context("Failed to get from Redis/Dragonfly")?;

        match json {
            Some(json_str) => {
                let page: CachedPage = serde_json::from_str(&json_str)
                    .context("Failed to deserialize cached page")?;
                Ok(Some(page))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, page: CachedPage) -> Result<()> {
        let full_key = self.full_key(key);
        let mut conn = self.manager.clone();

        let json = serde_json::to_string(&page)
            .context("Failed to serialize page")?;

        // Set with TTL based on revalidate_after
        let ttl_secs = page.revalidate_after.as_secs() as usize;

        conn.set_ex(&full_key, json, ttl_secs)
            .await
            .context("Failed to set in Redis/Dragonfly")?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.full_key(key);
        let mut conn = self.manager.clone();

        conn.del(&full_key)
            .await
            .context("Failed to delete from Redis/Dragonfly")?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let full_key = self.full_key(key);
        let mut conn = self.manager.clone();

        let exists: bool = conn
            .exists(&full_key)
            .await
            .context("Failed to check existence in Redis/Dragonfly")?;

        Ok(exists)
    }

    async fn clear(&self) -> Result<()> {
        // Delete all keys matching the prefix
        let mut conn = self.manager.clone();
        let pattern = format!("{}*", self.config.key_prefix);

        // Use SCAN to iterate over keys (safer than KEYS in production)
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .context("Failed to scan keys")?;

        if !keys.is_empty() {
            conn.del(&keys)
                .await
                .context("Failed to delete keys")?;
        }

        Ok(())
    }

    async fn keys(&self) -> Result<Vec<String>> {
        let mut conn = self.manager.clone();
        let pattern = format!("{}*", self.config.key_prefix);

        let full_keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .context("Failed to get keys")?;

        // Remove prefix from keys
        let prefix_len = self.config.key_prefix.len();
        let keys = full_keys
            .into_iter()
            .map(|k| k[prefix_len..].to_string())
            .collect();

        Ok(keys)
    }

    fn name(&self) -> &'static str {
        "dragonfly"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Note: These tests require a running Redis/Dragonfly instance
    // Skip them if not available

    async fn create_test_storage() -> Option<DragonflyStorage> {
        let config = DragonflyConfig {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            key_prefix: "rhtmx:isr:test:".to_string(),
        };

        DragonflyStorage::new(config).await.ok()
    }

    #[tokio::test]
    #[ignore] // Requires Redis/Dragonfly to be running
    async fn test_dragonfly_storage_basic() {
        let storage = match create_test_storage().await {
            Some(s) => s,
            None => {
                println!("Skipping test: Redis/Dragonfly not available");
                return;
            }
        };

        // Clear test data
        storage.clear().await.unwrap();

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
    #[ignore] // Requires Redis/Dragonfly to be running
    async fn test_dragonfly_ping() {
        let storage = match create_test_storage().await {
            Some(s) => s,
            None => {
                println!("Skipping test: Redis/Dragonfly not available");
                return;
            }
        };

        assert!(storage.ping().await.unwrap());
    }
}

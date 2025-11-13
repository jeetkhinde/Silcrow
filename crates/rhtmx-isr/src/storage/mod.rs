//! Storage backends for ISR cache

use crate::cache::CachedPage;
use anyhow::Result;
use async_trait::async_trait;

pub mod memory;
pub mod filesystem;

#[cfg(feature = "dragonfly")]
pub mod dragonfly;

/// Trait for ISR storage backends
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get a cached page by key
    async fn get(&self, key: &str) -> Result<Option<CachedPage>>;

    /// Set a cached page
    async fn set(&self, key: &str, page: CachedPage) -> Result<()>;

    /// Delete a cached page
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Clear all cached pages
    async fn clear(&self) -> Result<()>;

    /// Get all cache keys
    async fn keys(&self) -> Result<Vec<String>>;

    /// Get storage backend name
    fn name(&self) -> &'static str;
}

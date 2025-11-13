//! ISR engine core - handles caching and revalidation

use crate::cache::{CachedPage, CacheStats};
use crate::config::{IsrConfig, StorageBackend};
use crate::storage::Storage;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// ISR engine for managing cached pages
pub struct IsrEngine {
    config: IsrConfig,
    primary_storage: Arc<dyn Storage>,
    fallback_storage: Option<Arc<dyn Storage>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl IsrEngine {
    /// Create a new ISR engine
    pub async fn new(config: IsrConfig) -> Result<Self> {
        let primary_storage = Self::create_storage(&config.storage).await?;

        let fallback_storage = if let Some(ref fallback_config) = config.fallback {
            Some(Self::create_storage(fallback_config).await?)
        } else {
            None
        };

        Ok(Self {
            config,
            primary_storage,
            fallback_storage,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }

    /// Create a storage backend from config
    async fn create_storage(backend: &StorageBackend) -> Result<Arc<dyn Storage>> {
        match backend {
            StorageBackend::Memory => {
                use crate::storage::memory::MemoryStorage;
                Ok(Arc::new(MemoryStorage::new()))
            }
            StorageBackend::Filesystem(config) => {
                use crate::storage::filesystem::FilesystemStorage;
                let storage = FilesystemStorage::new(config.clone()).await?;
                Ok(Arc::new(storage))
            }
            #[cfg(feature = "dragonfly")]
            StorageBackend::Dragonfly(config) => {
                use crate::storage::dragonfly::DragonflyStorage;
                let storage = DragonflyStorage::new(config.clone()).await?;
                Ok(Arc::new(storage))
            }
            #[cfg(not(feature = "dragonfly"))]
            StorageBackend::Dragonfly(_) => {
                anyhow::bail!("Dragonfly storage requires the 'dragonfly' feature to be enabled")
            }
        }
    }

    /// Get a cached page or generate it
    ///
    /// Implements stale-while-revalidate:
    /// - If page is fresh, return it immediately
    /// - If page is stale, return it and regenerate in background
    /// - If page is missing, generate it now
    pub async fn get_or_generate<F, Fut, T>(
        &self,
        key: &str,
        revalidate_after: Duration,
        generator: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Try to get from cache
        if let Some(page) = self.get(key).await? {
            // Record hit
            self.record_hit().await;

            // Check if stale
            if page.is_stale() {
                // Stale-while-revalidate: serve old page, regenerate in background
                let engine = self.clone();
                let key = key.to_string();
                let revalidate_after_clone = revalidate_after;

                tokio::spawn(async move {
                    // This is a simplified version - in real implementation,
                    // we'd need to handle the actual regeneration
                    // For now, this just shows the pattern
                    let _ = engine.regenerate(&key, revalidate_after_clone).await;
                });
            }

            // This is a stub - in real implementation, we'd deserialize T from cached HTML
            // For now, we just regenerate
            Ok(generator().await)
        } else {
            // Record miss
            self.record_miss().await;

            // Not cached, generate now
            let result = generator().await;

            // This is a stub - in real implementation, we'd serialize T to HTML and cache it
            // For now, we skip caching

            Ok(result)
        }
    }

    /// Get a cached page by key
    pub async fn get(&self, key: &str) -> Result<Option<CachedPage>> {
        // Try primary storage
        if let Ok(Some(page)) = self.primary_storage.get(key).await {
            return Ok(Some(page));
        }

        // Try fallback if available
        if let Some(ref fallback) = self.fallback_storage {
            if let Ok(Some(page)) = fallback.get(key).await {
                // Promote to primary storage
                self.primary_storage.set(key, page.clone()).await.ok();
                return Ok(Some(page));
            }
        }

        Ok(None)
    }

    /// Set a cached page
    pub async fn set(&self, key: &str, page: CachedPage) -> Result<()> {
        // Set in primary storage
        self.primary_storage.set(key, page.clone()).await?;

        // Also set in fallback if available
        if let Some(ref fallback) = self.fallback_storage {
            fallback.set(key, page).await.ok();
        }

        Ok(())
    }

    /// Delete a cached page
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.primary_storage.delete(key).await?;

        if let Some(ref fallback) = self.fallback_storage {
            fallback.delete(key).await.ok();
        }

        Ok(())
    }

    /// Regenerate a cached page
    async fn regenerate(&self, key: &str, revalidate_after: Duration) -> Result<()> {
        // In a real implementation, this would:
        // 1. Call the original route handler
        // 2. Serialize the result to HTML
        // 3. Create a new CachedPage
        // 4. Store it in cache

        // For now, this is a stub
        let _ = (key, revalidate_after);
        Ok(())
    }

    /// Clear all cached pages
    pub async fn clear(&self) -> Result<()> {
        self.primary_storage.clear().await?;

        if let Some(ref fallback) = self.fallback_storage {
            fallback.clear().await.ok();
        }

        // Reset stats
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();

        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Record a cache hit
    async fn record_hit(&self) {
        let mut stats = self.stats.write().await;
        stats.hits += 1;
    }

    /// Record a cache miss
    async fn record_miss(&self) {
        let mut stats = self.stats.write().await;
        stats.misses += 1;
    }

    /// Revalidate a specific page on-demand
    pub async fn revalidate(&self, key: &str) -> Result<()> {
        self.delete(key).await
    }

    /// Revalidate multiple pages
    pub async fn revalidate_many(&self, keys: &[String]) -> Result<()> {
        for key in keys {
            self.revalidate(key).await.ok();
        }
        Ok(())
    }

    /// Get all cached page keys
    pub async fn keys(&self) -> Result<Vec<String>> {
        self.primary_storage.keys().await
    }
}

impl Clone for IsrEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            primary_storage: Arc::clone(&self.primary_storage),
            fallback_storage: self.fallback_storage.as_ref().map(Arc::clone),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageBackend;

    #[tokio::test]
    async fn test_isr_engine_memory_backend() {
        let config = IsrConfig {
            default_revalidate: Duration::from_secs(60),
            storage: StorageBackend::Memory,
            fallback: None,
        };

        let engine = IsrEngine::new(config).await.unwrap();

        // Set a page
        let page = CachedPage::new("test content".to_string(), Duration::from_secs(60));
        engine.set("test-key", page).await.unwrap();

        // Get it back
        let retrieved = engine.get("test-key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().html, "test content");

        // Delete it
        engine.delete("test-key").await.unwrap();
        assert!(engine.get("test-key").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_isr_engine_stats() {
        let config = IsrConfig {
            default_revalidate: Duration::from_secs(60),
            storage: StorageBackend::Memory,
            fallback: None,
        };

        let engine = IsrEngine::new(config).await.unwrap();

        // Record some hits and misses
        engine.record_hit().await;
        engine.record_hit().await;
        engine.record_miss().await;

        let stats = engine.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }
}

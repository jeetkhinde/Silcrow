//! Cached page types and utilities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A cached page with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPage {
    /// The HTML content
    pub html: String,

    /// When the page was generated
    pub generated_at: DateTime<Utc>,

    /// How long before revalidation is needed
    pub revalidate_after: Duration,

    /// Metadata about the page
    pub metadata: PageMetadata,
}

impl CachedPage {
    /// Create a new cached page
    pub fn new(html: String, revalidate_after: Duration) -> Self {
        Self {
            html,
            generated_at: Utc::now(),
            revalidate_after,
            metadata: PageMetadata::default(),
        }
    }

    /// Check if the cached page is stale
    pub fn is_stale(&self) -> bool {
        let age = Utc::now()
            .signed_duration_since(self.generated_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0));

        age >= self.revalidate_after
    }

    /// Check if the cached page is fresh
    pub fn is_fresh(&self) -> bool {
        !self.is_stale()
    }

    /// Get the age of the cached page
    pub fn age(&self) -> Duration {
        Utc::now()
            .signed_duration_since(self.generated_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0))
    }
}

/// Metadata about a cached page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageMetadata {
    /// Number of times this page has been served from cache
    pub hits: u64,

    /// Number of times this page has been regenerated
    pub regenerations: u64,

    /// Last revalidation timestamp
    pub last_revalidated: Option<DateTime<Utc>>,

    /// Size of the page in bytes
    pub size_bytes: usize,
}

impl PageMetadata {
    /// Increment hit count
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Increment regeneration count
    pub fn record_regeneration(&mut self) {
        self.regenerations += 1;
        self.last_revalidated = Some(Utc::now());
    }
}

/// Statistics for the ISR cache
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,

    /// Total number of cache misses
    pub misses: u64,

    /// Total number of regenerations
    pub regenerations: u64,

    /// Total size of cached pages in bytes
    pub total_size_bytes: usize,

    /// Number of cached pages
    pub page_count: usize,
}

impl CacheStats {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get average page size
    pub fn avg_page_size(&self) -> usize {
        if self.page_count == 0 {
            0
        } else {
            self.total_size_bytes / self.page_count
        }
    }
}

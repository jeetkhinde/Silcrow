//! ISR configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// ISR engine configuration
#[derive(Debug, Clone)]
pub struct IsrConfig {
    /// Default revalidation period
    pub default_revalidate: Duration,

    /// Primary storage backend
    pub storage: StorageBackend,

    /// Optional fallback storage backend
    pub fallback: Option<Box<StorageBackend>>,
}

impl Default for IsrConfig {
    fn default() -> Self {
        Self {
            default_revalidate: Duration::from_secs(60),
            storage: StorageBackend::Memory,
            fallback: None,
        }
    }
}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StorageBackend {
    /// In-memory storage (fast, non-persistent)
    Memory,

    /// Filesystem storage (persistent, single-instance)
    Filesystem(FilesystemConfig),

    /// Dragonfly/Redis storage (fast, distributed)
    Dragonfly(DragonflyConfig),
}

/// Filesystem storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    /// Cache directory path
    pub path: PathBuf,

    /// Maximum cache size in megabytes
    pub max_size_mb: u64,
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from(".rhtmx/cache"),
            max_size_mb: 500,
        }
    }
}

/// Dragonfly (Redis-compatible) storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragonflyConfig {
    /// Redis/Dragonfly connection URL
    pub url: String,

    /// Connection pool size
    pub pool_size: u32,

    /// Key prefix for ISR cache entries
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,
}

fn default_key_prefix() -> String {
    "rhtmx:isr:".to_string()
}

impl Default for DragonflyConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            key_prefix: default_key_prefix(),
        }
    }
}

/// TOML configuration for rhtmx.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsrTomlConfig {
    /// Default revalidation period in seconds
    pub default_revalidate: u64,

    /// Storage configuration
    pub storage: StorageTomlConfig,
}

/// Storage configuration in TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageTomlConfig {
    /// Primary storage backend: "memory", "filesystem", or "dragonfly"
    pub primary: String,

    /// Optional fallback storage backend
    pub fallback: Option<String>,

    /// Dragonfly-specific config
    pub dragonfly: Option<DragonflyConfig>,

    /// Filesystem-specific config
    pub filesystem: Option<FilesystemConfig>,
}

impl IsrTomlConfig {
    /// Convert TOML config to runtime config
    pub fn to_runtime_config(&self) -> anyhow::Result<IsrConfig> {
        let storage = self.parse_storage_backend(&self.storage.primary)?;

        let fallback = if let Some(ref fallback_type) = self.storage.fallback {
            Some(Box::new(self.parse_storage_backend(fallback_type)?))
        } else {
            None
        };

        Ok(IsrConfig {
            default_revalidate: Duration::from_secs(self.default_revalidate),
            storage,
            fallback,
        })
    }

    fn parse_storage_backend(&self, backend_type: &str) -> anyhow::Result<StorageBackend> {
        match backend_type {
            "memory" => Ok(StorageBackend::Memory),
            "filesystem" => {
                let config = self.storage.filesystem.clone()
                    .unwrap_or_default();
                Ok(StorageBackend::Filesystem(config))
            }
            "dragonfly" => {
                let config = self.storage.dragonfly.clone()
                    .ok_or_else(|| anyhow::anyhow!("Dragonfly storage requires [isr.storage.dragonfly] configuration"))?;
                Ok(StorageBackend::Dragonfly(config))
            }
            _ => Err(anyhow::anyhow!("Unknown storage backend: {}", backend_type))
        }
    }
}

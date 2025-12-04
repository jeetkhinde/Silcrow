// File: rusty-sync/src/syncable.rs
// Purpose: Syncable trait definition

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Trait for entities that can be synchronized to IndexedDB
///
/// This is automatically implemented by the #[derive(Syncable)] macro
pub trait Syncable: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// Get the entity name (table name)
    fn entity_name() -> &'static str;

    /// Get the primary key value as a string
    fn id(&self) -> String;

    /// Get the version number for optimistic concurrency control
    fn version(&self) -> Option<i64> {
        None
    }

    /// Set the version number
    fn set_version(&mut self, _version: i64) {}

    /// Get the last modified timestamp
    fn modified_at(&self) -> Option<DateTime<Utc>> {
        None
    }

    /// Set the last modified timestamp
    fn set_modified_at(&mut self, _timestamp: DateTime<Utc>) {}

    /// Check if this entity has sync metadata fields
    fn has_sync_metadata() -> bool {
        false
    }
}

/// Metadata for sync operations (added automatically to entities)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    /// Version number for optimistic concurrency control
    pub version: i64,

    /// Last modification timestamp
    pub modified_at: DateTime<Utc>,

    /// Client ID that made the last change
    pub client_id: Option<String>,
}

impl Default for SyncMetadata {
    fn default() -> Self {
        Self {
            version: 1,
            modified_at: Utc::now(),
            client_id: None,
        }
    }
}

// File: rusty-sync/src/conflict.rs
// Purpose: Conflict resolution strategies for sync

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStrategy {
    /// Last write wins (based on timestamp)
    LastWriteWins,
    /// Client changes always win
    ClientWins,
    /// Server changes always win
    ServerWins,
}

impl Default for SyncStrategy {
    fn default() -> Self {
        Self::LastWriteWins
    }
}

/// Trait for custom conflict resolution
pub trait ConflictResolver<T>: Send + Sync {
    /// Resolve conflict between server and client versions
    fn resolve(&self, server: T, client: T) -> T;
}

/// Default conflict resolver based on strategy
pub struct DefaultResolver {
    strategy: SyncStrategy,
}

impl DefaultResolver {
    pub fn new(strategy: SyncStrategy) -> Self {
        Self { strategy }
    }

    /// Resolve conflict between two entities with timestamps
    pub fn resolve_with_timestamp<T>(
        &self,
        server: T,
        server_ts: DateTime<Utc>,
        client: T,
        client_ts: DateTime<Utc>,
    ) -> T {
        match self.strategy {
            SyncStrategy::LastWriteWins => {
                if server_ts > client_ts {
                    server
                } else {
                    client
                }
            }
            SyncStrategy::ClientWins => client,
            SyncStrategy::ServerWins => server,
        }
    }
}

/// Represents a conflict that needs resolution
#[derive(Debug, Serialize, Deserialize)]
pub struct Conflict {
    pub entity: String,
    pub entity_id: String,
    pub server_version: i64,
    pub client_version: i64,
    pub reason: String,
}

impl Conflict {
    pub fn new(
        entity: String,
        entity_id: String,
        server_version: i64,
        client_version: i64,
        reason: String,
    ) -> Self {
        Self {
            entity,
            entity_id,
            server_version,
            client_version,
            reason,
        }
    }
}

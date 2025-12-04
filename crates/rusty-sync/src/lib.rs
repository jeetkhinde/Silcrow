// File: rusty-sync/src/lib.rs
// Purpose: Main entry point for rusty-sync library

//! # rusty-sync
//!
//! Automatic IndexedDB synchronization for Rust web applications.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rusty_sync::{Syncable, SyncEngine, SyncConfig};
//!
//! // 1. Add #[derive(Syncable)] to your models
//! #[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Syncable)]
//! pub struct User {
//!     pub id: i32,
//!     pub name: String,
//!     pub email: String,
//! }
//!
//! // 2. Initialize sync engine
//! let sync_engine = SyncEngine::new(SyncConfig {
//!     db_pool: pool.clone(),
//!     entities: vec!["users"],
//! }).await?;
//!
//! // 3. Add routes to your Axum app
//! let app = Router::new()
//!     .merge(sync_engine.routes());
//! ```

pub mod syncable;
pub mod change_tracker;
pub mod field_tracker;
pub mod sse;
pub mod websocket;
pub mod field_websocket;
pub mod sync_api;
pub mod field_sync_api;
pub mod conflict;
pub mod engine;
pub mod compression;
pub mod schema;
pub mod models;
pub mod db;
pub mod postgres_notify;

// Re-export main types
pub use syncable::Syncable;
pub use engine::{SyncEngine, SyncConfig};
pub use conflict::{SyncStrategy, ConflictResolver};
pub use change_tracker::{ChangeLog, ChangeAction};
pub use field_tracker::{FieldTracker, FieldChange, FieldAction, FieldMergeStrategy, FieldConflict};
pub use compression::CompressionConfig;
pub use db::DbPool;
pub use postgres_notify::{PostgresNotifyListener, PostgresNotification};

// The Syncable derive macro is provided by rhtmx-macro

/// Version of the sync protocol
pub const SYNC_PROTOCOL_VERSION: &str = "1.0.0";

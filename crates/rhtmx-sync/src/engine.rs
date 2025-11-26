// File: rhtmx-sync/src/engine.rs
// Purpose: Main sync engine orchestration

use axum::{
    routing::{get, post},
    Router, Extension,
};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::{
    change_tracker::ChangeTracker,
    field_tracker::{FieldTracker, FieldMergeStrategy},
    conflict::SyncStrategy,
    sse::sync_events_handler,
    websocket::ws_sync_handler,
    field_websocket::ws_field_sync_handler,
    sync_api::{get_sync_handler, post_sync_handler},
    field_sync_api::{get_field_sync_handler, post_field_sync_handler, get_latest_fields_handler},
};

/// Configuration for the sync engine
#[derive(Clone)]
pub struct SyncConfig {
    /// Database connection pool
    pub db_pool: SqlitePool,

    /// Entities to sync (table names)
    pub entities: Vec<String>,

    /// Conflict resolution strategy (for entity-level sync)
    pub strategy: SyncStrategy,

    /// Field-level merge strategy (for field-level sync)
    pub field_strategy: FieldMergeStrategy,

    /// Enable field-level sync (default: false for backward compatibility)
    pub enable_field_sync: bool,

    /// Enable debug logging
    pub debug: bool,
}

impl SyncConfig {
    pub fn new(db_pool: SqlitePool, entities: Vec<String>) -> Self {
        Self {
            db_pool,
            entities,
            strategy: SyncStrategy::default(),
            field_strategy: FieldMergeStrategy::default(),
            enable_field_sync: false,
            debug: false,
        }
    }

    /// Enable field-level synchronization
    pub fn with_field_sync(mut self, strategy: FieldMergeStrategy) -> Self {
        self.enable_field_sync = true;
        self.field_strategy = strategy;
        self
    }
}

/// Main sync engine
pub struct SyncEngine {
    #[allow(dead_code)]
    config: SyncConfig,
    change_tracker: Arc<ChangeTracker>,
    field_tracker: Option<Arc<FieldTracker>>,
}

impl SyncEngine {
    /// Create a new sync engine
    pub async fn new(config: SyncConfig) -> anyhow::Result<Self> {
        let db_pool = Arc::new(config.db_pool.clone());
        let change_tracker = Arc::new(ChangeTracker::new(db_pool.clone()).await?);

        // Initialize field tracker if enabled
        let field_tracker = if config.enable_field_sync {
            Some(Arc::new(
                FieldTracker::new(db_pool, config.field_strategy).await?,
            ))
        } else {
            None
        };

        Ok(Self {
            config,
            change_tracker,
            field_tracker,
        })
    }

    /// Get Axum routes for the sync API
    pub fn routes(&self) -> Router {
        let tracker = self.change_tracker.clone();
        let broadcast_tx = Arc::new(tracker.subscribe().resubscribe());

        let mut router = Router::new()
            // Sync API endpoints (entity-level)
            .route("/api/sync/:entity", get(get_sync_handler))
            .route("/api/sync/:entity", post(post_sync_handler))
            // Real-time updates - WebSocket (preferred) and SSE (fallback)
            .route("/api/sync/ws", get(ws_sync_handler))
            .route("/api/sync/events", get(sync_events_handler))
            // Client JavaScript libraries
            .route("/api/sync/client.js", get(serve_client_js))
            .route("/api/sync/field-client.js", get(serve_field_client_js))
            // Inject dependencies
            .with_state(tracker.clone())
            .layer(Extension(tracker))
            .layer(Extension(broadcast_tx));

        // Add field-level sync routes if enabled
        if let Some(field_tracker) = &self.field_tracker {
            let field_router = Router::new()
                .route("/api/field-sync/:entity", get(get_field_sync_handler))
                .route("/api/field-sync/:entity", post(post_field_sync_handler))
                .route(
                    "/api/field-sync/:entity/:entity_id/latest",
                    get(get_latest_fields_handler),
                )
                // WebSocket for field-level sync
                .route("/api/field-sync/ws", get(ws_field_sync_handler))
                .with_state(field_tracker.clone());

            router = router.merge(field_router);
        }

        router
    }

    /// Get the change tracker (entity-level)
    pub fn tracker(&self) -> &Arc<ChangeTracker> {
        &self.change_tracker
    }

    /// Get the field tracker (field-level)
    pub fn field_tracker(&self) -> Option<&Arc<FieldTracker>> {
        self.field_tracker.as_ref()
    }

    /// Clean up old sync log entries
    pub async fn cleanup(&self, days: i64) -> anyhow::Result<u64> {
        let entity_cleaned = self.change_tracker.cleanup_old_entries(days).await?;

        let field_cleaned = if let Some(field_tracker) = &self.field_tracker {
            field_tracker.cleanup_old_entries(days).await?
        } else {
            0
        };

        Ok(entity_cleaned + field_cleaned)
    }
}

/// Serve the client-side JavaScript library (entity-level sync)
async fn serve_client_js() -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        include_str!("js/rhtmx-sync.js"),
    )
}

/// Serve the field-level sync JavaScript library
async fn serve_field_client_js() -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        include_str!("js/rhtmx-field-sync.js"),
    )
}

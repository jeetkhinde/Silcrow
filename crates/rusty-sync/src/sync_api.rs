// File: rusty-sync/src/sync_api.rs
// Purpose: HTTP API endpoints for sync operations

use axum::{
    extract::{Path, Query, Extension},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::change_tracker::{ChangeTracker, ChangeLog, ChangeAction};

/// Query parameters for sync endpoint
#[derive(Debug, Deserialize)]
pub struct SyncQuery {
    /// Get changes since this version
    #[serde(default)]
    pub since: i64,
}

/// Response for sync GET request
#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub entity: String,
    pub version: i64,
    pub changes: Vec<ChangeLog>,
}

/// Request body for sync POST request
#[derive(Debug, Deserialize)]
pub struct SyncPushRequest {
    pub changes: Vec<ClientChange>,
}

/// A change from the client
#[derive(Debug, Deserialize)]
pub struct ClientChange {
    pub id: String,
    pub action: ChangeAction,
    pub data: Option<serde_json::Value>,
    pub client_version: Option<i64>,
}

/// Response for sync POST request
#[derive(Debug, Serialize)]
pub struct SyncPushResponse {
    pub version: i64,
    pub conflicts: Vec<String>,
    pub applied: Vec<String>,
}

/// Handler for GET /api/sync/:entity
///
/// Returns changes since the specified version
pub async fn get_sync_handler(
    Path(entity): Path<String>,
    Query(params): Query<SyncQuery>,
    Extension(tracker): Extension<Arc<ChangeTracker>>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let changes = tracker
        .get_changes_since(&entity, params.since)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let version = tracker
        .latest_version(&entity)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SyncResponse {
        entity,
        version,
        changes,
    }))
}

/// Handler for POST /api/sync/:entity
///
/// Push client changes to server
pub async fn post_sync_handler(
    Path(entity): Path<String>,
    Extension(tracker): Extension<Arc<ChangeTracker>>,
    Json(request): Json<SyncPushRequest>,
) -> Result<Json<SyncPushResponse>, StatusCode> {
    let mut applied = Vec::new();
    let mut conflicts = Vec::new();

    for change in request.changes {
        // Record the change
        match tracker
            .record_change(&entity, &change.id, change.action, change.data, None)
            .await
        {
            Ok(_) => {
                applied.push(change.id);
            }
            Err(e) => {
                conflicts.push(format!("{}: {}", change.id, e));
            }
        }
    }

    let version = tracker
        .latest_version(&entity)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SyncPushResponse {
        version,
        conflicts,
        applied,
    }))
}

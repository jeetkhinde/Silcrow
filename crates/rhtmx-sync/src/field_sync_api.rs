// File: rhtmx-sync/src/field_sync_api.rs
// Purpose: HTTP API endpoints for field-level synchronization

use crate::field_tracker::{FieldAction, FieldChange, FieldConflict, FieldTracker};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Query parameters for field sync requests
#[derive(Debug, Deserialize)]
pub struct FieldSyncQuery {
    /// Get changes since this version
    #[serde(default)]
    pub since: i64,
}

/// Response for GET /api/field-sync/:entity
#[derive(Debug, Serialize)]
pub struct FieldSyncResponse {
    pub entity: String,
    pub version: i64,
    pub changes: Vec<FieldChange>,
}

/// A field change from the client
#[derive(Debug, Deserialize, Serialize)]
pub struct ClientFieldChange {
    pub entity_id: String,
    pub field: String,
    pub value: Option<serde_json::Value>,
    pub action: FieldAction,
    pub timestamp: DateTime<Utc>,
}

/// Request body for POST /api/field-sync/:entity
#[derive(Debug, Deserialize, Serialize)]
pub struct FieldSyncPushRequest {
    pub changes: Vec<ClientFieldChange>,
}

/// Response for POST /api/field-sync/:entity
#[derive(Debug, Serialize)]
pub struct FieldSyncPushResponse {
    pub applied: Vec<FieldChange>,
    pub conflicts: Vec<FieldConflict>,
}

/// GET /api/field-sync/:entity?since=:version
/// Get field-level changes for an entity since a specific version
pub async fn get_field_sync_handler(
    State(tracker): State<Arc<FieldTracker>>,
    Path(entity): Path<String>,
    Query(query): Query<FieldSyncQuery>,
) -> Result<Json<FieldSyncResponse>, (StatusCode, String)> {
    // Get changes since specified version
    let changes = tracker
        .get_field_changes_since(&entity, query.since)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Get latest version
    let version = tracker
        .latest_version(&entity)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(FieldSyncResponse {
        entity,
        version,
        changes,
    }))
}

/// POST /api/field-sync/:entity
/// Push client field changes to server
pub async fn post_field_sync_handler(
    State(tracker): State<Arc<FieldTracker>>,
    Path(entity): Path<String>,
    Json(request): Json<FieldSyncPushRequest>,
) -> Result<Json<FieldSyncPushResponse>, (StatusCode, String)> {
    let mut all_applied = Vec::new();
    let mut all_conflicts = Vec::new();

    // Group changes by entity_id for efficient merging
    let mut changes_by_entity: HashMap<String, Vec<(String, serde_json::Value, DateTime<Utc>)>> =
        HashMap::new();

    for change in request.changes {
        if change.action == FieldAction::Update {
            if let Some(value) = change.value {
                changes_by_entity
                    .entry(change.entity_id)
                    .or_insert_with(Vec::new)
                    .push((change.field, value, change.timestamp));
            }
        } else if change.action == FieldAction::Delete {
            // Handle delete action
            let field_change = tracker
                .record_field_change(
                    &entity,
                    &change.entity_id,
                    &change.field,
                    None,
                    FieldAction::Delete,
                    None,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            all_applied.push(field_change);
        }
    }

    // Merge changes for each entity instance
    for (entity_id, field_changes) in changes_by_entity {
        let (applied, conflicts) = tracker
            .merge_field_changes(&entity, &entity_id, field_changes)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        all_applied.extend(applied);
        all_conflicts.extend(conflicts);
    }

    Ok(Json(FieldSyncPushResponse {
        applied: all_applied,
        conflicts: all_conflicts,
    }))
}

/// GET /api/field-sync/:entity/:entity_id/latest
/// Get the latest field values for a specific entity instance
pub async fn get_latest_fields_handler(
    State(tracker): State<Arc<FieldTracker>>,
    Path((entity, entity_id)): Path<(String, String)>,
) -> Result<Json<HashMap<String, serde_json::Value>>, (StatusCode, String)> {
    let fields = tracker
        .get_latest_fields(&entity, &entity_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(fields))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field_tracker::FieldMergeStrategy;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::{get, post};
    use axum::Router;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn create_test_app() -> Router {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let tracker = Arc::new(
            FieldTracker::new(Arc::new(pool), FieldMergeStrategy::LastWriteWins)
                .await
                .unwrap(),
        );

        Router::new()
            .route("/api/field-sync/:entity", get(get_field_sync_handler))
            .route("/api/field-sync/:entity", post(post_field_sync_handler))
            .route(
                "/api/field-sync/:entity/:entity_id/latest",
                get(get_latest_fields_handler),
            )
            .with_state(tracker)
    }

    #[tokio::test]
    async fn test_get_field_sync() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/field-sync/users?since=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_field_sync() {
        let app = create_test_app().await;

        let push_request = FieldSyncPushRequest {
            changes: vec![ClientFieldChange {
                entity_id: "1".to_string(),
                field: "name".to_string(),
                value: Some(serde_json::json!("Alice")),
                action: FieldAction::Update,
                timestamp: Utc::now(),
            }],
        };

        let body = serde_json::to_string(&push_request).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/field-sync/users")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

// File: rhtmx-sync/src/field_tracker.rs
// Purpose: Track field-level changes for fine-grained synchronization
// Similar to Yjs/Automerge - supports CRDT-like field-level sync

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Action performed on a field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FieldAction {
    /// Field value updated
    Update,
    /// Field deleted/cleared
    Delete,
}

impl std::fmt::Display for FieldAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldAction::Update => write!(f, "update"),
            FieldAction::Delete => write!(f, "delete"),
        }
    }
}

/// A single field-level change entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub id: i64,
    pub entity: String,
    pub entity_id: String,
    pub field: String,
    pub value: Option<serde_json::Value>,
    pub action: FieldAction,
    pub version: i64,
    pub client_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Batch of field changes for an entity instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChangeBatch {
    pub entity: String,
    pub entity_id: String,
    pub changes: Vec<FieldChange>,
    pub batch_version: i64,
}

/// Field-level merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldMergeStrategy {
    /// Last write wins (timestamp-based)
    LastWriteWins,
    /// Keep both values and let application decide
    KeepBoth,
    /// Always prefer server value
    ServerWins,
    /// Always prefer client value
    ClientWins,
}

impl Default for FieldMergeStrategy {
    fn default() -> Self {
        Self::LastWriteWins
    }
}

/// Conflict at field level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConflict {
    pub entity: String,
    pub entity_id: String,
    pub field: String,
    pub server_value: Option<serde_json::Value>,
    pub server_timestamp: DateTime<Utc>,
    pub client_value: Option<serde_json::Value>,
    pub client_timestamp: DateTime<Utc>,
    pub resolution: FieldMergeStrategy,
}

/// Manages field-level change tracking
pub struct FieldTracker {
    db_pool: Arc<SqlitePool>,
    broadcast_tx: broadcast::Sender<FieldChange>,
    merge_strategy: FieldMergeStrategy,
}

impl FieldTracker {
    /// Create a new field tracker
    pub async fn new(
        db_pool: Arc<SqlitePool>,
        merge_strategy: FieldMergeStrategy,
    ) -> anyhow::Result<Self> {
        Self::init_field_sync_table(&db_pool).await?;

        let (broadcast_tx, _) = broadcast::channel(1000);

        Ok(Self {
            db_pool,
            broadcast_tx,
            merge_strategy,
        })
    }

    /// Initialize the field sync log table
    async fn init_field_sync_table(pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _rhtmx_field_sync_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                field TEXT NOT NULL,
                value TEXT,
                action TEXT NOT NULL,
                version INTEGER NOT NULL,
                client_id TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Create composite index for efficient field-level queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_field_sync_entity_field
            ON _rhtmx_field_sync_log(entity, entity_id, field, version)
            "#,
        )
        .execute(pool)
        .await?;

        // Create index for version-based queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_field_sync_version
            ON _rhtmx_field_sync_log(entity, version)
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record a field-level change
    pub async fn record_field_change(
        &self,
        entity: &str,
        entity_id: &str,
        field: &str,
        value: Option<serde_json::Value>,
        action: FieldAction,
        client_id: Option<String>,
    ) -> anyhow::Result<FieldChange> {
        use sqlx::Row;

        // Get next version number for this entity
        let version = self.next_version(entity).await?;

        // Serialize value to JSON string if present
        let value_json = value.as_ref().map(|v| serde_json::to_string(v).unwrap());

        // Insert into field sync log
        let row = sqlx::query(
            r#"
            INSERT INTO _rhtmx_field_sync_log
            (entity, entity_id, field, value, action, version, client_id, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            RETURNING id, entity, entity_id, field, value, action, version, client_id, timestamp
            "#
        )
        .bind(entity)
        .bind(entity_id)
        .bind(field)
        .bind(value_json)
        .bind(action.to_string())
        .bind(version)
        .bind(&client_id)
        .fetch_one(&*self.db_pool)
        .await?;

        // Parse row into FieldChange
        let action_str: String = row.get("action");
        let action_parsed = match action_str.as_str() {
            "update" => FieldAction::Update,
            "delete" => FieldAction::Delete,
            _ => FieldAction::Update,
        };

        let value_str: Option<String> = row.get("value");
        let value_parsed = value_str.and_then(|s| serde_json::from_str(&s).ok());

        let change = FieldChange {
            id: row.get("id"),
            entity: row.get("entity"),
            entity_id: row.get("entity_id"),
            field: row.get("field"),
            value: value_parsed,
            action: action_parsed,
            version: row.get("version"),
            client_id: row.get("client_id"),
            timestamp: row.get("timestamp"),
        };

        // Broadcast the change
        let _ = self.broadcast_tx.send(change.clone());

        Ok(change)
    }

    /// Record multiple field changes as a batch
    pub async fn record_field_batch(
        &self,
        entity: &str,
        entity_id: &str,
        fields: Vec<(&str, Option<serde_json::Value>, FieldAction)>,
        client_id: Option<String>,
    ) -> anyhow::Result<Vec<FieldChange>> {
        let mut changes = Vec::new();

        for (field, value, action) in fields {
            let change = self
                .record_field_change(entity, entity_id, field, value, action, client_id.clone())
                .await?;
            changes.push(change);
        }

        Ok(changes)
    }

    /// Get all field changes since a specific version
    pub async fn get_field_changes_since(
        &self,
        entity: &str,
        since_version: i64,
    ) -> anyhow::Result<Vec<FieldChange>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, entity, entity_id, field, value, action, version, client_id, timestamp
            FROM _rhtmx_field_sync_log
            WHERE entity = ? AND version > ?
            ORDER BY version ASC, id ASC
            "#
        )
        .bind(entity)
        .bind(since_version)
        .fetch_all(&*self.db_pool)
        .await?;

        let changes = rows
            .iter()
            .map(|row| {
                let action_str: String = row.get("action");
                let action = match action_str.as_str() {
                    "update" => FieldAction::Update,
                    "delete" => FieldAction::Delete,
                    _ => FieldAction::Update,
                };

                let value_str: Option<String> = row.get("value");
                let value = value_str.and_then(|s| serde_json::from_str(&s).ok());

                FieldChange {
                    id: row.get("id"),
                    entity: row.get("entity"),
                    entity_id: row.get("entity_id"),
                    field: row.get("field"),
                    value,
                    action,
                    version: row.get("version"),
                    client_id: row.get("client_id"),
                    timestamp: row.get("timestamp"),
                }
            })
            .collect();

        Ok(changes)
    }

    /// Get latest field values for a specific entity instance
    pub async fn get_latest_fields(
        &self,
        entity: &str,
        entity_id: &str,
    ) -> anyhow::Result<HashMap<String, serde_json::Value>> {
        use sqlx::Row;

        // Get the latest change for each field
        let rows = sqlx::query(
            r#"
            SELECT field, value, action
            FROM _rhtmx_field_sync_log
            WHERE entity = ? AND entity_id = ?
            AND id IN (
                SELECT MAX(id)
                FROM _rhtmx_field_sync_log
                WHERE entity = ? AND entity_id = ?
                GROUP BY field
            )
            "#
        )
        .bind(entity)
        .bind(entity_id)
        .bind(entity)
        .bind(entity_id)
        .fetch_all(&*self.db_pool)
        .await?;

        let mut fields = HashMap::new();

        for row in rows {
            let field: String = row.get("field");
            let action_str: String = row.get("action");
            let value_str: Option<String> = row.get("value");

            // Skip deleted fields
            if action_str == "delete" {
                continue;
            }

            if let Some(v) = value_str.and_then(|s| serde_json::from_str(&s).ok()) {
                fields.insert(field, v);
            }
        }

        Ok(fields)
    }

    /// Merge client field changes with server state
    pub async fn merge_field_changes(
        &self,
        entity: &str,
        entity_id: &str,
        client_changes: Vec<(String, serde_json::Value, DateTime<Utc>)>,
    ) -> anyhow::Result<(Vec<FieldChange>, Vec<FieldConflict>)> {
        let mut applied_changes = Vec::new();
        let mut conflicts = Vec::new();

        for (field, client_value, client_timestamp) in client_changes {
            // Get latest server value for this field
            let server_state = self.get_latest_field_value(entity, entity_id, &field).await?;

            let should_apply = match server_state {
                Some((server_value, server_timestamp)) => {
                    // Check for conflict
                    if server_timestamp > client_timestamp {
                        // Server has newer change
                        match self.merge_strategy {
                            FieldMergeStrategy::LastWriteWins => false,
                            FieldMergeStrategy::ServerWins => false,
                            FieldMergeStrategy::ClientWins => true,
                            FieldMergeStrategy::KeepBoth => {
                                conflicts.push(FieldConflict {
                                    entity: entity.to_string(),
                                    entity_id: entity_id.to_string(),
                                    field: field.clone(),
                                    server_value: Some(server_value),
                                    server_timestamp,
                                    client_value: Some(client_value.clone()),
                                    client_timestamp,
                                    resolution: self.merge_strategy,
                                });
                                false
                            }
                        }
                    } else {
                        true
                    }
                }
                None => true, // No server value, apply client change
            };

            if should_apply {
                let change = self
                    .record_field_change(
                        entity,
                        entity_id,
                        &field,
                        Some(client_value),
                        FieldAction::Update,
                        None,
                    )
                    .await?;
                applied_changes.push(change);
            }
        }

        Ok((applied_changes, conflicts))
    }

    /// Get latest value for a specific field
    async fn get_latest_field_value(
        &self,
        entity: &str,
        entity_id: &str,
        field: &str,
    ) -> anyhow::Result<Option<(serde_json::Value, DateTime<Utc>)>> {
        use sqlx::Row;

        let row = sqlx::query(
            r#"
            SELECT value, timestamp, action
            FROM _rhtmx_field_sync_log
            WHERE entity = ? AND entity_id = ? AND field = ?
            ORDER BY id DESC
            LIMIT 1
            "#
        )
        .bind(entity)
        .bind(entity_id)
        .bind(field)
        .fetch_optional(&*self.db_pool)
        .await?;

        if let Some(row) = row {
            let action_str: String = row.get("action");
            if action_str == "delete" {
                return Ok(None);
            }

            let value_str: Option<String> = row.get("value");
            let timestamp: DateTime<Utc> = row.get("timestamp");

            if let Some(value) = value_str.and_then(|s| serde_json::from_str(&s).ok()) {
                return Ok(Some((value, timestamp)));
            }
        }

        Ok(None)
    }

    /// Get the latest version for an entity
    pub async fn latest_version(&self, entity: &str) -> anyhow::Result<i64> {
        let result: Option<i64> = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM _rhtmx_field_sync_log WHERE entity = ?"
        )
        .bind(entity)
        .fetch_one(&*self.db_pool)
        .await?;

        Ok(result.unwrap_or(0))
    }

    /// Get next version number for an entity
    async fn next_version(&self, entity: &str) -> anyhow::Result<i64> {
        let current = self.latest_version(entity).await?;
        Ok(current + 1)
    }

    /// Subscribe to field change events
    pub fn subscribe(&self) -> broadcast::Receiver<FieldChange> {
        self.broadcast_tx.subscribe()
    }

    /// Clean up old field sync log entries
    pub async fn cleanup_old_entries(&self, days: i64) -> anyhow::Result<u64> {
        let days_param = format!("-{} days", days);
        let result = sqlx::query(
            "DELETE FROM _rhtmx_field_sync_log WHERE timestamp < datetime('now', ?)"
        )
        .bind(days_param)
        .execute(&*self.db_pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_field_tracker() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let tracker = FieldTracker::new(Arc::new(pool), FieldMergeStrategy::LastWriteWins)
            .await
            .unwrap();

        // Record a field change
        let change = tracker
            .record_field_change(
                "users",
                "1",
                "name",
                Some(serde_json::json!("Alice")),
                FieldAction::Update,
                None,
            )
            .await
            .unwrap();

        assert_eq!(change.entity, "users");
        assert_eq!(change.field, "name");
        assert_eq!(change.version, 1);

        // Record another field change
        tracker
            .record_field_change(
                "users",
                "1",
                "email",
                Some(serde_json::json!("alice@example.com")),
                FieldAction::Update,
                None,
            )
            .await
            .unwrap();

        // Get latest fields for entity
        let fields = tracker.get_latest_fields("users", "1").await.unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields.get("name").unwrap(), &serde_json::json!("Alice"));
    }

    #[tokio::test]
    async fn test_field_changes_since() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let tracker = FieldTracker::new(Arc::new(pool), FieldMergeStrategy::LastWriteWins)
            .await
            .unwrap();

        // Record multiple field changes
        tracker
            .record_field_change(
                "users",
                "1",
                "name",
                Some(serde_json::json!("Alice")),
                FieldAction::Update,
                None,
            )
            .await
            .unwrap();

        tracker
            .record_field_change(
                "users",
                "1",
                "email",
                Some(serde_json::json!("alice@example.com")),
                FieldAction::Update,
                None,
            )
            .await
            .unwrap();

        // Get changes since version 0
        let changes = tracker.get_field_changes_since("users", 0).await.unwrap();
        assert_eq!(changes.len(), 2);
    }

    #[tokio::test]
    async fn test_field_merge_conflict() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let tracker = FieldTracker::new(Arc::new(pool), FieldMergeStrategy::LastWriteWins)
            .await
            .unwrap();

        // Server change (older)
        tracker
            .record_field_change(
                "users",
                "1",
                "name",
                Some(serde_json::json!("Alice")),
                FieldAction::Update,
                None,
            )
            .await
            .unwrap();

        // Client change (newer)
        let client_time = Utc::now();
        let client_changes = vec![(
            "name".to_string(),
            serde_json::json!("Bob"),
            client_time,
        )];

        let (applied, conflicts) = tracker
            .merge_field_changes("users", "1", client_changes)
            .await
            .unwrap();

        // With LastWriteWins, newer client change should be applied
        assert_eq!(applied.len(), 1);
        assert_eq!(conflicts.len(), 0);
    }
}

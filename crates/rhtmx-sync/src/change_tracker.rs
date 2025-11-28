// File: rhtmx-sync/src/change_tracker.rs
// Purpose: Track database changes for synchronization
// PostgreSQL uses Diesel (PRIMARY), SQLite uses sqlx (OPTIONAL)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::db::DbPool;

/// Action performed on an entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeAction {
    Create,
    Update,
    Delete,
}

impl std::fmt::Display for ChangeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeAction::Create => write!(f, "create"),
            ChangeAction::Update => write!(f, "update"),
            ChangeAction::Delete => write!(f, "delete"),
        }
    }
}

/// A single change entry in the sync log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeLog {
    pub id: i64,
    pub entity: String,
    pub entity_id: String,
    pub action: ChangeAction,
    pub data: Option<serde_json::Value>,
    pub version: i64,
    pub client_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Manages change tracking and broadcasts
pub struct ChangeTracker {
    db_pool: Arc<DbPool>,
    broadcast_tx: broadcast::Sender<ChangeLog>,
}

impl ChangeTracker {
    /// Create a new change tracker
    pub async fn new(db_pool: Arc<DbPool>) -> anyhow::Result<Self> {
        // Initialize database tables based on type
        Self::init_tables(&db_pool).await?;

        let (broadcast_tx, _) = broadcast::channel(1000);

        Ok(Self {
            db_pool,
            broadcast_tx,
        })
    }

    /// Initialize database tables
    async fn init_tables(pool: &DbPool) -> anyhow::Result<()> {
        match pool {
            DbPool::Postgres(_) => {
                // PostgreSQL tables are created via Diesel migrations
                // Run: diesel migration run --database-url=<postgres_url>
                tracing::info!("PostgreSQL tables managed via Diesel migrations");
                Ok(())
            }
            DbPool::Sqlite(sqlite_pool) => {
                // SQLite uses sqlx for backward compatibility
                Self::init_sqlite_table(sqlite_pool).await
            }
        }
    }

    /// Initialize SQLite table using sqlx (backward compatibility)
    async fn init_sqlite_table(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _rhtmx_sync_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                data TEXT,
                version INTEGER NOT NULL,
                client_id TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sync_entity_version
            ON _rhtmx_sync_log(entity, version)
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record a change in the sync log
    pub async fn record_change(
        &self,
        entity: &str,
        entity_id: &str,
        action: ChangeAction,
        data: Option<serde_json::Value>,
        client_id: Option<String>,
    ) -> anyhow::Result<ChangeLog> {
        match self.db_pool.as_ref() {
            DbPool::Postgres(_) => {
                self.record_change_postgres(entity, entity_id, action, data, client_id)
                    .await
            }
            DbPool::Sqlite(_) => {
                self.record_change_sqlite(entity, entity_id, action, data, client_id)
                    .await
            }
        }
    }

    /// Record a change using PostgreSQL with Diesel
    async fn record_change_postgres(
        &self,
        entity: &str,
        entity_id: &str,
        action: ChangeAction,
        data: Option<serde_json::Value>,
        client_id: Option<String>,
    ) -> anyhow::Result<ChangeLog> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        use crate::models::{NewSyncLog, SyncLog};
        use crate::schema::_rhtmx_sync_log;

        // Get next version
        let version = self.next_version(entity).await?;

        // Create new sync log entry
        let new_log = NewSyncLog::new(
            entity.to_string(),
            entity_id.to_string(),
            action.clone(),
            data.clone(),
            version,
            client_id.clone(),
        );

        // Insert and return the created record
        let mut conn = self.db_pool.get_postgres().await?;

        let sync_log = diesel::insert_into(_rhtmx_sync_log::table)
            .values(&new_log)
            .get_result::<SyncLog>(&mut conn)
            .await?;

        // Convert to ChangeLog and broadcast
        let change = sync_log.to_change_log();
        let _ = self.broadcast_tx.send(change.clone());

        Ok(change)
    }

    /// Record a change using SQLite with sqlx
    async fn record_change_sqlite(
        &self,
        entity: &str,
        entity_id: &str,
        action: ChangeAction,
        data: Option<serde_json::Value>,
        client_id: Option<String>,
    ) -> anyhow::Result<ChangeLog> {
        use sqlx::Row;

        let pool = self.db_pool.get_sqlite()?;
        let version = self.next_version(entity).await?;
        let data_json = data.as_ref().map(|d| serde_json::to_string(d).unwrap());

        let row = sqlx::query(
            r#"
            INSERT INTO _rhtmx_sync_log (entity, entity_id, action, data, version, client_id)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, entity, entity_id, action, data, version, client_id, created_at
            "#
        )
        .bind(entity)
        .bind(entity_id)
        .bind(action.to_string())
        .bind(data_json)
        .bind(version)
        .bind(&client_id)
        .fetch_one(pool.as_ref())
        .await?;

        let action_str: String = row.get("action");
        let action_parsed = match action_str.as_str() {
            "create" => ChangeAction::Create,
            "update" => ChangeAction::Update,
            "delete" => ChangeAction::Delete,
            _ => ChangeAction::Update,
        };

        let data_str: Option<String> = row.get("data");
        let data_parsed = data_str.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

        let change = ChangeLog {
            id: row.get("id"),
            entity: row.get("entity"),
            entity_id: row.get("entity_id"),
            action: action_parsed,
            data: data_parsed,
            version: row.get("version"),
            client_id: row.get("client_id"),
            created_at: row.get("created_at"),
        };

        let _ = self.broadcast_tx.send(change.clone());
        Ok(change)
    }

    /// Get all changes since a specific version
    pub async fn get_changes_since(
        &self,
        entity: &str,
        since_version: i64,
    ) -> anyhow::Result<Vec<ChangeLog>> {
        match self.db_pool.as_ref() {
            DbPool::Postgres(_) => self.get_changes_since_postgres(entity, since_version).await,
            DbPool::Sqlite(_) => self.get_changes_since_sqlite(entity, since_version).await,
        }
    }

    /// Get changes using PostgreSQL with Diesel
    async fn get_changes_since_postgres(
        &self,
        entity_name: &str,
        since_version: i64,
    ) -> anyhow::Result<Vec<ChangeLog>> {
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        use crate::models::SyncLog;
        use crate::schema::_rhtmx_sync_log::dsl::*;

        let mut conn = self.db_pool.get_postgres().await?;

        let results = _rhtmx_sync_log
            .filter(entity.eq(entity_name))
            .filter(version.gt(since_version))
            .order(version.asc())
            .load::<SyncLog>(&mut conn)
            .await?;

        Ok(results.into_iter().map(|r| r.to_change_log()).collect())
    }

    /// Get changes using SQLite with sqlx
    async fn get_changes_since_sqlite(
        &self,
        entity: &str,
        since_version: i64,
    ) -> anyhow::Result<Vec<ChangeLog>> {
        use sqlx::Row;

        let pool = self.db_pool.get_sqlite()?;

        let rows = sqlx::query(
            r#"
            SELECT id, entity, entity_id, action, data, version, client_id, created_at
            FROM _rhtmx_sync_log
            WHERE entity = ? AND version > ?
            ORDER BY version ASC
            "#
        )
        .bind(entity)
        .bind(since_version)
        .fetch_all(pool.as_ref())
        .await?;

        let changes = rows
            .iter()
            .map(|row| {
                let action_str: String = row.get("action");
                let action = match action_str.as_str() {
                    "create" => ChangeAction::Create,
                    "update" => ChangeAction::Update,
                    "delete" => ChangeAction::Delete,
                    _ => ChangeAction::Update,
                };

                let data_str: Option<String> = row.get("data");
                let data = data_str.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

                ChangeLog {
                    id: row.get("id"),
                    entity: row.get("entity"),
                    entity_id: row.get("entity_id"),
                    action,
                    data,
                    version: row.get("version"),
                    client_id: row.get("client_id"),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        Ok(changes)
    }

    /// Get the latest version for an entity
    pub async fn latest_version(&self, entity: &str) -> anyhow::Result<i64> {
        match self.db_pool.as_ref() {
            DbPool::Postgres(_) => self.latest_version_postgres(entity).await,
            DbPool::Sqlite(_) => self.latest_version_sqlite(entity).await,
        }
    }

    /// Get latest version using PostgreSQL with Diesel
    async fn latest_version_postgres(&self, entity_name: &str) -> anyhow::Result<i64> {
        use diesel::dsl::max;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        use crate::schema::_rhtmx_sync_log::dsl::*;

        let mut conn = self.db_pool.get_postgres().await?;

        let result: Option<i64> = _rhtmx_sync_log
            .filter(entity.eq(entity_name))
            .select(max(version))
            .first::<Option<i64>>(&mut conn)
            .await
            .unwrap_or(None);

        Ok(result.unwrap_or(0))
    }

    /// Get latest version using SQLite with sqlx
    async fn latest_version_sqlite(&self, entity: &str) -> anyhow::Result<i64> {
        let pool = self.db_pool.get_sqlite()?;

        let result: Option<i64> = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM _rhtmx_sync_log WHERE entity = ?"
        )
        .bind(entity)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(result.unwrap_or(0))
    }

    /// Get next version number for an entity
    async fn next_version(&self, entity: &str) -> anyhow::Result<i64> {
        let current = self.latest_version(entity).await?;
        Ok(current + 1)
    }

    /// Subscribe to change events
    pub fn subscribe(&self) -> broadcast::Receiver<ChangeLog> {
        self.broadcast_tx.subscribe()
    }

    /// Clean up old sync log entries (call periodically)
    pub async fn cleanup_old_entries(&self, days: i64) -> anyhow::Result<u64> {
        match self.db_pool.as_ref() {
            DbPool::Postgres(_) => self.cleanup_old_entries_postgres(days).await,
            DbPool::Sqlite(_) => self.cleanup_old_entries_sqlite(days).await,
        }
    }

    /// Cleanup old entries using PostgreSQL with Diesel
    async fn cleanup_old_entries_postgres(&self, days: i64) -> anyhow::Result<u64> {
        use chrono::Duration;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        use crate::schema::_rhtmx_sync_log::dsl::*;

        let cutoff = Utc::now() - Duration::days(days);
        let cutoff_naive = cutoff.naive_utc();

        let mut conn = self.db_pool.get_postgres().await?;

        let deleted = diesel::delete(_rhtmx_sync_log.filter(created_at.lt(cutoff_naive)))
            .execute(&mut conn)
            .await?;

        Ok(deleted as u64)
    }

    /// Cleanup old entries using SQLite with sqlx
    async fn cleanup_old_entries_sqlite(&self, days: i64) -> anyhow::Result<u64> {
        let pool = self.db_pool.get_sqlite()?;
        let days_param = format!("-{} days", days);

        let result = sqlx::query(
            "DELETE FROM _rhtmx_sync_log WHERE created_at < datetime('now', ?)"
        )
        .bind(days_param)
        .execute(pool.as_ref())
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_change_tracker_sqlite() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let db_pool = DbPool::Sqlite(Arc::new(pool));
        let tracker = ChangeTracker::new(Arc::new(db_pool)).await.unwrap();

        // Record a change
        let change = tracker
            .record_change(
                "users",
                "1",
                ChangeAction::Create,
                Some(serde_json::json!({"name": "Alice"})),
                None,
            )
            .await
            .unwrap();

        assert_eq!(change.entity, "users");
        assert_eq!(change.version, 1);

        // Get latest version
        let version = tracker.latest_version("users").await.unwrap();
        assert_eq!(version, 1);

        // Get changes since version 0
        let changes = tracker.get_changes_since("users", 0).await.unwrap();
        assert_eq!(changes.len(), 1);
    }

    #[tokio::test]
    async fn test_change_tracker_postgres() {
        // This test requires a running PostgreSQL instance
        // Set DATABASE_URL environment variable to run this test
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            if db_url.starts_with("postgres") {
                let pool = DbPool::from_url(&db_url).await.unwrap();
                let tracker = ChangeTracker::new(Arc::new(pool)).await.unwrap();

                // Record a change
                let change = tracker
                    .record_change(
                        "users",
                        "1",
                        ChangeAction::Create,
                        Some(serde_json::json!({"name": "Alice"})),
                        None,
                    )
                    .await
                    .unwrap();

                assert_eq!(change.entity, "users");
                assert!(change.version >= 1);

                // Get changes since version 0
                let changes = tracker.get_changes_since("users", 0).await.unwrap();
                assert!(!changes.is_empty());
            }
        }
    }
}

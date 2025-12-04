// File: rusty-sync/src/models.rs
// Purpose: Diesel models for database tables

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::change_tracker::ChangeAction;
use crate::field_tracker::FieldAction;
use crate::schema::{_rhtmx_sync_log, _rhtmx_field_sync_log};

/// Queryable model for reading from _rhtmx_sync_log table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = _rhtmx_sync_log)]
pub struct SyncLog {
    pub id: i64,
    pub entity: String,
    pub entity_id: String,
    pub action: String,
    pub data: Option<String>,
    pub version: i64,
    pub client_id: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

/// Insertable model for inserting into _rhtmx_sync_log table
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = _rhtmx_sync_log)]
pub struct NewSyncLog {
    pub entity: String,
    pub entity_id: String,
    pub action: String,
    pub data: Option<String>,
    pub version: i64,
    pub client_id: Option<String>,
}

impl NewSyncLog {
    pub fn new(
        entity: String,
        entity_id: String,
        action: ChangeAction,
        data: Option<serde_json::Value>,
        version: i64,
        client_id: Option<String>,
    ) -> Self {
        let data_json = data.map(|d| serde_json::to_string(&d).unwrap());

        Self {
            entity,
            entity_id,
            action: action.to_string(),
            data: data_json,
            version,
            client_id,
        }
    }
}

impl SyncLog {
    /// Convert to ChangeLog for the public API
    pub fn to_change_log(&self) -> crate::change_tracker::ChangeLog {
        let action = match self.action.as_str() {
            "create" => ChangeAction::Create,
            "update" => ChangeAction::Update,
            "delete" => ChangeAction::Delete,
            _ => ChangeAction::Update,
        };

        let data = self.data.as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        crate::change_tracker::ChangeLog {
            id: self.id,
            entity: self.entity.clone(),
            entity_id: self.entity_id.clone(),
            action,
            data,
            version: self.version,
            client_id: self.client_id.clone(),
            created_at: DateTime::<Utc>::from_naive_utc_and_offset(self.created_at, Utc),
        }
    }
}

/// Queryable model for reading from _rhtmx_field_sync_log table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = _rhtmx_field_sync_log)]
pub struct FieldSyncLog {
    pub id: i64,
    pub entity: String,
    pub entity_id: String,
    pub field: String,
    pub value: Option<String>,
    pub action: String,
    pub version: i64,
    pub client_id: Option<String>,
    pub timestamp: chrono::NaiveDateTime,
}

/// Insertable model for inserting into _rhtmx_field_sync_log table
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = _rhtmx_field_sync_log)]
pub struct NewFieldSyncLog {
    pub entity: String,
    pub entity_id: String,
    pub field: String,
    pub value: Option<String>,
    pub action: String,
    pub version: i64,
    pub client_id: Option<String>,
}

impl NewFieldSyncLog {
    pub fn new(
        entity: String,
        entity_id: String,
        field: String,
        value: Option<serde_json::Value>,
        action: FieldAction,
        version: i64,
        client_id: Option<String>,
    ) -> Self {
        let value_json = value.map(|v| serde_json::to_string(&v).unwrap());

        Self {
            entity,
            entity_id,
            field,
            value: value_json,
            action: action.to_string(),
            version,
            client_id,
        }
    }
}

impl FieldSyncLog {
    /// Convert to FieldChange for the public API
    pub fn to_field_change(&self) -> crate::field_tracker::FieldChange {
        let action = match self.action.as_str() {
            "update" => FieldAction::Update,
            "delete" => FieldAction::Delete,
            _ => FieldAction::Update,
        };

        let value = self.value.as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        crate::field_tracker::FieldChange {
            id: self.id,
            entity: self.entity.clone(),
            entity_id: self.entity_id.clone(),
            field: self.field.clone(),
            value,
            action,
            version: self.version,
            client_id: self.client_id.clone(),
            timestamp: DateTime::<Utc>::from_naive_utc_and_offset(self.timestamp, Utc),
        }
    }
}

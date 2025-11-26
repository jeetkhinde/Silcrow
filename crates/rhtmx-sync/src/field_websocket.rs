// File: rhtmx-sync/src/field_websocket.rs
// Purpose: WebSocket-based field-level sync

use crate::field_tracker::{FieldAction, FieldChange, FieldTracker};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// WebSocket message types for field-level sync
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldSyncMessage {
    /// Client subscribes to entity field changes
    Subscribe {
        entities: Vec<String>,
    },
    /// Client requests field sync since version
    Sync {
        entity: String,
        since: i64,
    },
    /// Server sends field change notification
    FieldChange {
        change: FieldChange,
    },
    /// Client pushes field changes
    PushFields {
        entity: String,
        entity_id: String,
        fields: Vec<FieldUpdate>,
    },
    /// Server acknowledges field push
    PushAck {
        entity: String,
        entity_id: String,
        applied: usize,
        conflicts: usize,
    },
    /// Server reports field conflict
    Conflict {
        entity: String,
        entity_id: String,
        field: String,
        server_value: Option<serde_json::Value>,
        server_timestamp: DateTime<Utc>,
        client_value: Option<serde_json::Value>,
        client_timestamp: DateTime<Utc>,
    },
    /// Error response
    Error {
        message: String,
    },
    /// Heartbeat/ping
    Ping,
    /// Heartbeat/pong
    Pong,
}

/// Field update from client
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldUpdate {
    pub field: String,
    pub value: Option<serde_json::Value>,
    pub action: FieldAction,
    pub timestamp: DateTime<Utc>,
}

/// WebSocket handler for field-level sync
pub async fn ws_field_sync_handler(
    State(tracker): State<Arc<FieldTracker>>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|socket| handle_field_sync_socket(socket, tracker))
}

async fn handle_field_sync_socket(socket: WebSocket, tracker: Arc<FieldTracker>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast field changes
    let mut broadcast_rx = tracker.subscribe();
    let tracker_clone = tracker.clone();

    // Handle incoming messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<FieldSyncMessage>(&text) {
                    Ok(sync_msg) => {
                        if let Some(response) =
                            handle_client_field_message(sync_msg, &tracker_clone).await
                        {
                            // Would send response back through sender
                            // For now we'll handle via broadcast
                            tracing::debug!("Response ready: {:?}", response);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse field message: {}", e);
                    }
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Send broadcast field changes to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(change) = broadcast_rx.recv().await {
            let msg = FieldSyncMessage::FieldChange { change };

            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    }

    tracing::debug!("Field sync WebSocket connection closed");
}

async fn handle_client_field_message(
    msg: FieldSyncMessage,
    tracker: &Arc<FieldTracker>,
) -> Option<FieldSyncMessage> {
    match msg {
        FieldSyncMessage::Sync { entity, since } => {
            // Client requests field sync
            match tracker.get_field_changes_since(&entity, since).await {
                Ok(changes) => {
                    tracing::debug!("Sending {} field changes for {}", changes.len(), entity);
                    // Would send changes back - for now handled via HTTP
                    None
                }
                Err(e) => Some(FieldSyncMessage::Error {
                    message: format!("Failed to get changes: {}", e),
                }),
            }
        }
        FieldSyncMessage::PushFields {
            entity,
            entity_id,
            fields,
        } => {
            // Client pushes field changes
            let mut applied = 0;
            let mut conflicts = 0;

            // Convert to merge format
            let field_changes: Vec<(String, serde_json::Value, DateTime<Utc>)> = fields
                .iter()
                .filter_map(|f| {
                    if f.action == FieldAction::Update {
                        f.value
                            .clone()
                            .map(|v| (f.field.clone(), v, f.timestamp))
                    } else {
                        None
                    }
                })
                .collect();

            match tracker
                .merge_field_changes(&entity, &entity_id, field_changes)
                .await
            {
                Ok((applied_changes, conflict_list)) => {
                    applied = applied_changes.len();
                    conflicts = conflict_list.len();

                    tracing::debug!(
                        "Applied {} field changes, {} conflicts for {}:{}",
                        applied,
                        conflicts,
                        entity,
                        entity_id
                    );

                    // Handle deletes
                    for field_update in fields.iter() {
                        if field_update.action == FieldAction::Delete {
                            if let Err(e) = tracker
                                .record_field_change(
                                    &entity,
                                    &entity_id,
                                    &field_update.field,
                                    None,
                                    FieldAction::Delete,
                                    None,
                                )
                                .await
                            {
                                tracing::error!("Failed to record delete: {}", e);
                            } else {
                                applied += 1;
                            }
                        }
                    }

                    Some(FieldSyncMessage::PushAck {
                        entity,
                        entity_id,
                        applied,
                        conflicts,
                    })
                }
                Err(e) => Some(FieldSyncMessage::Error {
                    message: format!("Failed to merge changes: {}", e),
                }),
            }
        }
        FieldSyncMessage::Subscribe { entities } => {
            tracing::debug!("Client subscribed to field changes: {:?}", entities);
            None
        }
        FieldSyncMessage::Ping => {
            Some(FieldSyncMessage::Pong)
        }
        _ => {
            tracing::warn!("Unexpected field message from client: {:?}", msg);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_sync_message_serialization() {
        let msg = FieldSyncMessage::Subscribe {
            entities: vec!["users".to_string()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));

        let deserialized: FieldSyncMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            FieldSyncMessage::Subscribe { entities } => {
                assert_eq!(entities, vec!["users"]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_push_fields_message() {
        let msg = FieldSyncMessage::PushFields {
            entity: "users".to_string(),
            entity_id: "1".to_string(),
            fields: vec![FieldUpdate {
                field: "name".to_string(),
                value: Some(serde_json::json!("Alice")),
                action: FieldAction::Update,
                timestamp: Utc::now(),
            }],
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: FieldSyncMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            FieldSyncMessage::PushFields {
                entity,
                entity_id,
                fields,
            } => {
                assert_eq!(entity, "users");
                assert_eq!(entity_id, "1");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].field, "name");
            }
            _ => panic!("Wrong message type"),
        }
    }
}

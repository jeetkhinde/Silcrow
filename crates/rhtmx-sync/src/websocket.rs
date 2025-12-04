// File: rusty-sync/src/websocket.rs
// Purpose: WebSocket-based real-time sync (better than SSE for bidirectional sync)

use crate::change_tracker::{ChangeAction, ChangeLog, ChangeTracker};
use crate::compression::{compress_message, decompress, CompressedMessage, CompressionConfig};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// WebSocket message types for sync protocol
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncMessage {
    /// Client subscribes to entity changes
    Subscribe {
        entities: Vec<String>,
    },
    /// Client requests sync since version
    Sync {
        entity: String,
        since: i64,
    },
    /// Server sends change notification
    Change {
        change: ChangeLog,
    },
    /// Client pushes a change
    Push {
        entity: String,
        entity_id: String,
        action: ChangeAction,
        data: Option<serde_json::Value>,
    },
    /// Server acknowledges push
    PushAck {
        entity: String,
        entity_id: String,
        version: i64,
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

/// WebSocket state with compression config
#[derive(Clone)]
pub struct WebSocketState {
    pub tracker: Arc<ChangeTracker>,
    pub compression: CompressionConfig,
}

impl WebSocketState {
    pub fn new(tracker: Arc<ChangeTracker>, compression: CompressionConfig) -> Self {
        Self {
            tracker,
            compression,
        }
    }
}

/// WebSocket handler for entity-level sync
pub async fn ws_sync_handler(State(state): State<Arc<WebSocketState>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_sync_socket(socket, state))
}

async fn handle_sync_socket(socket: WebSocket, state: Arc<WebSocketState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast changes
    let mut broadcast_rx = state.tracker.subscribe();
    let tracker_clone = state.tracker.clone();
    let compression_config = state.compression.clone();

    // Handle incoming messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let text = match msg {
                Message::Text(t) => t,
                Message::Binary(data) => {
                    // Decompress binary message
                    match decompress(&data) {
                        Ok(decompressed) => match String::from_utf8(decompressed) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::error!("Failed to decode decompressed message: {}", e);
                                continue;
                            }
                        },
                        Err(e) => {
                            tracing::error!("Failed to decompress message: {}", e);
                            continue;
                        }
                    }
                }
                Message::Close(_) => break,
                _ => continue,
            };

            match serde_json::from_str::<SyncMessage>(&text) {
                Ok(sync_msg) => {
                    if let Err(e) = handle_client_message(sync_msg, &tracker_clone).await {
                        tracing::error!("Error handling client message: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse message: {}", e);
                }
            }
        }
    });

    // Send broadcast changes to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(change) = broadcast_rx.recv().await {
            let msg = SyncMessage::Change { change };

            if let Ok(json) = serde_json::to_string(&msg) {
                // Compress if needed
                match compress_message(&json, &compression_config) {
                    Ok(CompressedMessage::Compressed(data)) => {
                        if sender.send(Message::Binary(data)).await.is_err() {
                            break;
                        }
                    }
                    Ok(CompressedMessage::Uncompressed(text)) => {
                        if sender.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to compress message: {}", e);
                        // Fall back to uncompressed
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Wait for either task to finish (client disconnect or error)
    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    }

    tracing::debug!("WebSocket connection closed");
}

async fn handle_client_message(
    msg: SyncMessage,
    tracker: &Arc<ChangeTracker>,
) -> anyhow::Result<()> {
    match msg {
        SyncMessage::Sync { entity, since } => {
            // Client requests sync - would send response back
            // For now, client can use HTTP endpoint for initial sync
            tracing::debug!("Sync request for {} since {}", entity, since);
        }
        SyncMessage::Push {
            entity,
            entity_id,
            action,
            data,
        } => {
            // Client pushes a change
            tracker
                .record_change(&entity, &entity_id, action, data, None)
                .await?;
        }
        SyncMessage::Subscribe { entities } => {
            tracing::debug!("Client subscribed to: {:?}", entities);
        }
        SyncMessage::Ping => {
            // Heartbeat - would send Pong back
            tracing::trace!("Received ping");
        }
        _ => {
            tracing::warn!("Unexpected message from client: {:?}", msg);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_serialization() {
        let msg = SyncMessage::Subscribe {
            entities: vec!["users".to_string()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("users"));

        let deserialized: SyncMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            SyncMessage::Subscribe { entities } => {
                assert_eq!(entities, vec!["users"]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_push_message() {
        let msg = SyncMessage::Push {
            entity: "users".to_string(),
            entity_id: "1".to_string(),
            action: ChangeAction::Update,
            data: Some(serde_json::json!({"name": "Alice"})),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: SyncMessage = serde_json::from_str(&json).unwrap();

        match deserialized {
            SyncMessage::Push {
                entity,
                entity_id,
                action,
                data,
            } => {
                assert_eq!(entity, "users");
                assert_eq!(entity_id, "1");
                assert_eq!(action, ChangeAction::Update);
                assert!(data.is_some());
            }
            _ => panic!("Wrong message type"),
        }
    }
}

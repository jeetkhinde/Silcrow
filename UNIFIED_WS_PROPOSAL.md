# Unified WebSocket Protocol for RHTMX

## Problem

Currently, rhtmx-sync creates separate WebSocket connections for different features:
- `/api/sync/ws` - Entity-level sync
- `/api/field-sync/ws` - Field-level sync
- (Future) Form validation would need another connection

This creates multiple connections per browser client, which is inefficient.

## Solution: Single Multiplexed WebSocket Connection

One connection at `/api/ws` that handles all message types.

## Proposed Message Protocol

```rust
// File: rusty-sync/src/unified_websocket.rs

use serde::{Deserialize, Serialize};
use crate::change_tracker::{ChangeLog, ChangeAction};
use crate::field_tracker::{FieldChange, FieldAction};
use chrono::{DateTime, Utc};

/// Unified WebSocket message protocol
/// All messages go over a single WebSocket connection
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnifiedMessage {
    // ========================================
    // Entity-Level Sync
    // ========================================

    /// Subscribe to entity changes
    Subscribe {
        entities: Vec<String>,
    },

    /// Request sync since version
    Sync {
        entity: String,
        since: i64,
    },

    /// Server sends entity change
    Change {
        change: ChangeLog,
    },

    /// Client pushes entity change
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

    // ========================================
    // Field-Level Sync
    // ========================================

    /// Subscribe to field changes
    SubscribeFields {
        entities: Vec<String>,
    },

    /// Server sends field change
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
    FieldPushAck {
        entity: String,
        entity_id: String,
        applied: usize,
        conflicts: usize,
    },

    /// Field conflict notification
    FieldConflict {
        entity: String,
        entity_id: String,
        field: String,
        server_value: Option<serde_json::Value>,
        server_timestamp: DateTime<Utc>,
        client_value: Option<serde_json::Value>,
        client_timestamp: DateTime<Utc>,
    },

    // ========================================
    // Form Validation (NEW)
    // ========================================

    /// Client requests field validation
    ValidateField {
        /// Request ID for correlation
        request_id: String,
        /// Form identifier
        form: String,
        /// Field name
        field: String,
        /// Field value to validate
        value: String,
    },

    /// Server returns validation result
    ValidationResult {
        /// Correlate with request
        request_id: String,
        /// Form identifier
        form: String,
        /// Field name
        field: String,
        /// Is valid?
        valid: bool,
        /// Error message if invalid
        error: Option<String>,
        /// Optional validation metadata
        metadata: Option<serde_json::Value>,
    },

    /// Batch validate multiple fields
    ValidateForm {
        request_id: String,
        form: String,
        fields: Vec<FormFieldValue>,
    },

    /// Batch validation result
    FormValidationResult {
        request_id: String,
        form: String,
        valid: bool,
        errors: Vec<FieldError>,
    },

    // ========================================
    // Common
    // ========================================

    /// Error response
    Error {
        message: String,
        code: Option<String>,
    },

    /// Heartbeat ping
    Ping,

    /// Heartbeat pong
    Pong,
}

/// Field value for validation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormFieldValue {
    pub field: String,
    pub value: String,
}

/// Field validation error
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldError {
    pub field: String,
    pub error: String,
}

/// Field update (for field sync)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldUpdate {
    pub field: String,
    pub value: Option<serde_json::Value>,
    pub action: FieldAction,
    pub timestamp: DateTime<Utc>,
}
```

## Unified WebSocket Handler

```rust
// File: rusty-sync/src/unified_websocket.rs (continued)

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;

/// Unified state containing all trackers
#[derive(Clone)]
pub struct UnifiedWebSocketState {
    pub change_tracker: Arc<ChangeTracker>,
    pub field_tracker: Option<Arc<FieldTracker>>,
    pub validation_handler: Arc<ValidationHandler>,
    pub compression: CompressionConfig,
}

/// Unified WebSocket handler
pub async fn unified_ws_handler(
    State(state): State<Arc<UnifiedWebSocketState>>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|socket| handle_unified_socket(socket, state))
}

async fn handle_unified_socket(socket: WebSocket, state: Arc<UnifiedWebSocketState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to all relevant broadcasts
    let entity_broadcast = state.change_tracker.subscribe();
    let field_broadcast = state.field_tracker.as_ref().map(|t| t.subscribe());

    // Spawn receiver task
    let state_clone = state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<UnifiedMessage>(&text) {
                    Ok(msg) => {
                        // Route to appropriate handler
                        handle_unified_message(msg, &state_clone).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse message: {}", e);
                    }
                }
            }
        }
    });

    // Spawn sender task (broadcasts from all sources)
    let send_task = tokio::spawn(async move {
        // Multiplex entity changes, field changes, etc.
        // Send to client over single connection
        todo!("Implement multiplexing")
    });

    // Wait for completion
    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }
}

async fn handle_unified_message(msg: UnifiedMessage, state: &Arc<UnifiedWebSocketState>) {
    match msg {
        // Entity sync messages
        UnifiedMessage::Subscribe { .. } => {
            // Handle via change_tracker
        }
        UnifiedMessage::Push { .. } => {
            // Handle via change_tracker
        }

        // Field sync messages
        UnifiedMessage::PushFields { .. } => {
            // Handle via field_tracker
        }

        // Validation messages
        UnifiedMessage::ValidateField { request_id, form, field, value } => {
            // Handle via validation_handler
            let result = state.validation_handler.validate_field(&form, &field, &value).await;

            // Send response back (need sender reference)
            // Response: UnifiedMessage::ValidationResult { ... }
        }

        UnifiedMessage::ValidateForm { request_id, form, fields } => {
            // Batch validation
            let results = state.validation_handler.validate_form(&form, fields).await;
            // Send back FormValidationResult
        }

        UnifiedMessage::Ping => {
            // Send Pong
        }

        _ => {}
    }
}
```

## Validation Handler

```rust
// File: rusty-sync/src/validation_handler.rs

use std::sync::Arc;
use anyhow::Result;

pub struct ValidationHandler {
    // Could store registered validators, Nutype types, etc.
}

impl ValidationHandler {
    pub async fn validate_field(
        &self,
        form: &str,
        field: &str,
        value: &str,
    ) -> ValidationResult {
        // Example: validate email field
        match (form, field) {
            ("contact", "email") => {
                match EmailAddress::try_new(value) {
                    Ok(_) => ValidationResult::Valid,
                    Err(e) => ValidationResult::Invalid(format!("{}", e)),
                }
            }
            ("contact", "message") => {
                if value.len() < 10 {
                    ValidationResult::Invalid("Message too short".into())
                } else {
                    ValidationResult::Valid
                }
            }
            _ => ValidationResult::Valid
        }
    }

    pub async fn validate_form(
        &self,
        form: &str,
        fields: Vec<FormFieldValue>,
    ) -> FormValidationResults {
        // Validate all fields, return batch result
        todo!()
    }
}

pub enum ValidationResult {
    Valid,
    Invalid(String),
}
```

## Client-Side Usage

```javascript
// One WebSocket connection per browser session
const ws = new WebSocket('ws://localhost:3000/api/ws');

// Subscribe to entity changes
ws.send(JSON.stringify({
    type: 'subscribe',
    entities: ['users', 'posts']
}));

// Subscribe to field changes
ws.send(JSON.stringify({
    type: 'subscribe_fields',
    entities: ['users']
}));

// Validate field as user types
function validateField(form, field, value) {
    const requestId = crypto.randomUUID();

    ws.send(JSON.stringify({
        type: 'validate_field',
        request_id: requestId,
        form: form,
        field: field,
        value: value
    }));

    // Response comes back via ws.onmessage:
    // { type: 'validation_result', request_id: '...', valid: true/false, error: '...' }
}

// Handle all message types from one connection
ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);

    switch (msg.type) {
        case 'change':
            // Handle entity change
            updateIndexedDB(msg.change);
            break;

        case 'field_change':
            // Handle field change
            updateField(msg.change);
            break;

        case 'validation_result':
            // Show validation feedback
            showValidationError(msg.field, msg.error);
            break;
    }
};
```

## Migration Path

1. **Phase 1:** Create `unified_websocket.rs` with new protocol
2. **Phase 2:** Keep existing endpoints for backward compatibility:
   - `/api/sync/ws` → wraps unified handler, filters to entity messages
   - `/api/field-sync/ws` → wraps unified handler, filters to field messages
3. **Phase 3:** Deprecate old endpoints, encourage `/api/ws`
4. **Phase 4:** Remove old endpoints in next major version

## Benefits

✅ One WebSocket connection per client (efficient)
✅ Multiplexed message types (entity, field, validation)
✅ Easy to extend (add new message types)
✅ Simpler client-side code (one connection to manage)
✅ Better resource utilization (fewer connections)
✅ Backward compatible (via wrapper endpoints)

## Implementation Checklist

- [ ] Create `unified_websocket.rs` with `UnifiedMessage` enum
- [ ] Create `validation_handler.rs` for form validation
- [ ] Create `UnifiedWebSocketState` combining all trackers
- [ ] Implement `unified_ws_handler` with message routing
- [ ] Update `engine.rs` to expose `/api/ws` endpoint
- [ ] Create client-side `unified-sync.js` library
- [ ] Add tests for validation message flow
- [ ] Document migration from old endpoints

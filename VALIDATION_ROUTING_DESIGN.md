# Validation Message Routing Design

## Problem

With 1000 forms and multiple users/tabs:
- Validation results must go to the correct tab
- No broadcasting to other users or tabs
- Same user can have multiple tabs with different forms
- Same form type can have multiple instances

## Solution: Connection-Scoped Request-Response

### Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Server                                 │
│  ┌────────────────────────────────────────────────┐      │
│  │ WebSocket Connection Manager                   │      │
│  │  - Tracks active connections by client_id      │      │
│  │  - Routes responses to specific connections    │      │
│  └────────────────────────────────────────────────┘      │
│                                                           │
│  Connection 1 (client_id: abc123) ←─┐                    │
│  Connection 2 (client_id: xyz789) ←─┼─ Unique connections│
│  Connection 3 (client_id: def456) ←─┘                    │
└──────────────────────────────────────────────────────────┘
         ↑              ↑              ↑
         │              │              │
    Tab 1          Tab 2          Tab 3
  (User A)      (User A)      (User B)
  Form: signup  Form: contact Form: signup
  Instance: 1   Instance: 2   Instance: 3
```

### Key Principles

1. **Each tab = One WebSocket connection** (no connection sharing)
2. **Validation = Request-Response** (not broadcast)
3. **Entity/Field Sync = Broadcast** (all connected clients)
4. **Form instance IDs** prevent cross-contamination

## Implementation

### 1. Client-Side: Tab Initialization

```javascript
// File: rusty-sync/src/js/unified-sync.js

class UnifiedSyncClient {
    constructor(options) {
        // Generate unique IDs per tab
        this.clientId = crypto.randomUUID();
        this.sessionId = this.getOrCreateSessionId();

        // Track pending validation requests
        this.pendingValidations = new Map();

        // Connect WebSocket with client_id
        this.connect();
    }

    connect() {
        const url = `${this.wsUrl}?client_id=${this.clientId}&session_id=${this.sessionId}`;
        this.ws = new WebSocket(url);

        this.ws.onmessage = (event) => this.handleMessage(event);
        this.ws.onopen = () => this.onConnected();
    }

    // Session ID persists across page reloads (same tab)
    getOrCreateSessionId() {
        let sessionId = sessionStorage.getItem('rusty_session_id');
        if (!sessionId) {
            sessionId = crypto.randomUUID();
            sessionStorage.setItem('rusty_session_id', sessionId);
        }
        return sessionId;
    }

    handleMessage(event) {
        const msg = JSON.parse(event.data);

        switch (msg.type) {
            case 'change':
            case 'field_change':
                // These are BROADCAST - all tabs receive
                this.handleBroadcastChange(msg);
                break;

            case 'validation_result':
                // This is TARGETED - only requesting tab receives
                this.handleValidationResult(msg);
                break;
        }
    }

    handleValidationResult(msg) {
        const pending = this.pendingValidations.get(msg.request_id);

        if (!pending) {
            console.warn('Received validation for unknown request:', msg.request_id);
            return;
        }

        // Verify it's for the correct form instance
        if (msg.form_instance !== pending.formInstance) {
            console.warn('Form instance mismatch:', msg.form_instance, pending.formInstance);
            return;
        }

        // Update UI for this specific form field
        this.showValidationFeedback(
            pending.formInstance,
            pending.field,
            msg.valid,
            msg.error
        );

        // Clean up
        this.pendingValidations.delete(msg.request_id);

        // Dispatch event for this specific form
        const event = new CustomEvent('rusty:validation:result', {
            detail: {
                formInstance: msg.form_instance,
                field: msg.field,
                valid: msg.valid,
                error: msg.error
            }
        });
        document.dispatchEvent(event);
    }
}
```

### 2. Client-Side: Form Field Validation

```javascript
class FormValidator {
    constructor(formElement, syncClient) {
        this.form = formElement;
        this.syncClient = syncClient;

        // Generate unique instance ID for this form
        this.formInstance = formElement.id || `form-${crypto.randomUUID()}`;
        this.formType = formElement.dataset.formType || 'unknown';

        // Attach to form element
        formElement.dataset.formInstance = this.formInstance;

        // Set up field listeners
        this.setupFieldListeners();
    }

    setupFieldListeners() {
        const fields = this.form.querySelectorAll('input, textarea, select');

        fields.forEach(field => {
            // Debounced validation on input
            let timeout;
            field.addEventListener('input', (e) => {
                clearTimeout(timeout);
                timeout = setTimeout(() => {
                    this.validateField(field.name, field.value);
                }, 300); // 300ms debounce
            });

            // Immediate validation on blur
            field.addEventListener('blur', (e) => {
                this.validateField(field.name, field.value);
            });
        });
    }

    validateField(fieldName, value) {
        const requestId = crypto.randomUUID();

        // Send validation request
        this.syncClient.ws.send(JSON.stringify({
            type: 'validate_field',
            request_id: requestId,
            client_id: this.syncClient.clientId,
            form_type: this.formType,
            form_instance: this.formInstance,  // ← Unique per form
            field: fieldName,
            value: value
        }));

        // Track pending request
        this.syncClient.pendingValidations.set(requestId, {
            formInstance: this.formInstance,
            formType: this.formType,
            field: fieldName,
            timestamp: Date.now()
        });
    }
}

// Usage
document.addEventListener('DOMContentLoaded', () => {
    const syncClient = new UnifiedSyncClient({ wsUrl: 'ws://localhost:3000/api/ws' });

    // Initialize validator for each form on the page
    document.querySelectorAll('form[data-validate]').forEach(form => {
        new FormValidator(form, syncClient);
    });
});
```

### 3. Server-Side: Connection Manager

```rust
// File: rusty-sync/src/unified_websocket.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::extract::ws::{WebSocket, Message};
use futures::stream::SplitSink;

/// Manages active WebSocket connections
pub struct ConnectionManager {
    /// Map client_id -> WebSocket sender
    connections: Arc<RwLock<HashMap<String, SplitSink<WebSocket, Message>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new connection
    pub async fn register(&self, client_id: String, sender: SplitSink<WebSocket, Message>) {
        let mut conns = self.connections.write().await;
        conns.insert(client_id, sender);
        tracing::info!("Registered connection: {}", client_id);
    }

    /// Remove a connection
    pub async fn unregister(&self, client_id: &str) {
        let mut conns = self.connections.write().await;
        conns.remove(client_id);
        tracing::info!("Unregistered connection: {}", client_id);
    }

    /// Send a message to a specific client (for validation responses)
    pub async fn send_to_client(&self, client_id: &str, message: UnifiedMessage) -> Result<(), String> {
        let mut conns = self.connections.write().await;

        if let Some(sender) = conns.get_mut(client_id) {
            let json = serde_json::to_string(&message)
                .map_err(|e| format!("Serialization error: {}", e))?;

            sender.send(Message::Text(json))
                .await
                .map_err(|e| format!("Send error: {}", e))?;

            Ok(())
        } else {
            Err(format!("Client not found: {}", client_id))
        }
    }

    /// Broadcast to all connections (for entity/field sync changes)
    pub async fn broadcast(&self, message: UnifiedMessage) {
        let conns = self.connections.read().await;
        let json = match serde_json::to_string(&message) {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };

        for (client_id, sender) in conns.iter() {
            if let Err(e) = sender.send(Message::Text(json.clone())).await {
                tracing::warn!("Failed to send to {}: {}", client_id, e);
            }
        }
    }
}
```

### 4. Server-Side: Message Handler

```rust
// File: rusty-sync/src/unified_websocket.rs (continued)

async fn handle_unified_message(
    msg: UnifiedMessage,
    client_id: &str,
    state: &Arc<UnifiedWebSocketState>,
    conn_manager: &Arc<ConnectionManager>,
) -> Result<(), String> {
    match msg {
        // ============================================
        // VALIDATION (Request-Response, NOT broadcast)
        // ============================================

        UnifiedMessage::ValidateField {
            request_id,
            form_type,
            form_instance,
            field,
            value,
            ..
        } => {
            // Validate the field
            let result = state.validation_handler
                .validate_field(&form_type, &field, &value)
                .await;

            // Send response ONLY to the requesting client
            let response = UnifiedMessage::ValidationResult {
                request_id,
                client_id: client_id.to_string(),
                form_type,
                form_instance,  // ← Echo back for verification
                field,
                valid: result.is_valid(),
                error: result.error(),
                metadata: None,
            };

            // Send to specific client (NOT broadcast)
            conn_manager.send_to_client(client_id, response).await?;
        }

        // ============================================
        // ENTITY SYNC (Broadcast to all)
        // ============================================

        UnifiedMessage::Push { entity, entity_id, action, data } => {
            // Record change
            state.change_tracker
                .record_change(&entity, &entity_id, action, data, Some(client_id.to_string()))
                .await?;

            // Broadcast to all connected clients
            let broadcast = UnifiedMessage::Change {
                change: /* ... */
            };
            conn_manager.broadcast(broadcast).await;
        }

        // ============================================
        // FIELD SYNC (Broadcast to all)
        // ============================================

        UnifiedMessage::PushFields { entity, entity_id, fields } => {
            // Merge field changes
            if let Some(field_tracker) = &state.field_tracker {
                let (applied, conflicts) = field_tracker
                    .merge_field_changes(&entity, &entity_id, fields)
                    .await?;

                // Broadcast field changes to all clients
                for change in applied {
                    let broadcast = UnifiedMessage::FieldChange { change };
                    conn_manager.broadcast(broadcast).await;
                }
            }
        }

        _ => {}
    }

    Ok(())
}
```

### 5. HTML Example: Multiple Forms

```html
<!DOCTYPE html>
<html>
<head>
    <script src="/api/sync/unified-client.js"></script>
</head>
<body>
    <!-- Form Instance 1: Signup form -->
    <form id="signup-form-1" data-form-type="signup" data-validate>
        <h2>Sign Up</h2>
        <input type="email" name="email" placeholder="Email">
        <input type="password" name="password" placeholder="Password">
        <div class="validation-errors" data-for="signup-form-1"></div>
        <button type="submit">Sign Up</button>
    </form>

    <!-- Form Instance 2: Contact form (same page, different form) -->
    <form id="contact-form-1" data-form-type="contact" data-validate>
        <h2>Contact Us</h2>
        <input type="email" name="email" placeholder="Email">
        <textarea name="message" placeholder="Message"></textarea>
        <div class="validation-errors" data-for="contact-form-1"></div>
        <button type="submit">Send</button>
    </form>

    <script>
        // Each form gets its own validator instance
        const syncClient = new UnifiedSyncClient({ wsUrl: 'ws://localhost:3000/api/ws' });

        // Initialize validators
        new FormValidator(document.getElementById('signup-form-1'), syncClient);
        new FormValidator(document.getElementById('contact-form-1'), syncClient);

        // Listen for validation results
        document.addEventListener('rusty:validation:result', (e) => {
            const { formInstance, field, valid, error } = e.detail;

            // Update only the specific form instance
            const errorDiv = document.querySelector(`[data-for="${formInstance}"]`);
            if (!valid) {
                errorDiv.textContent = `${field}: ${error}`;
            } else {
                errorDiv.textContent = '';
            }
        });
    </script>
</body>
</html>
```

## Message Flow Example

### Scenario: User A has 2 tabs open, User B has 1 tab

```
User A - Tab 1: Signup form (instance: signup-abc)
User A - Tab 2: Contact form (instance: contact-xyz)
User B - Tab 1: Signup form (instance: signup-def)
```

### User A - Tab 1 validates email:

```
1. Client sends:
   {
     type: 'validate_field',
     request_id: 'req-123',
     client_id: 'client-a-tab1',
     form_instance: 'signup-abc',
     field: 'email',
     value: 'invalid-email'
   }

2. Server validates and sends response ONLY to client-a-tab1:
   {
     type: 'validation_result',
     request_id: 'req-123',
     client_id: 'client-a-tab1',
     form_instance: 'signup-abc',
     field: 'email',
     valid: false,
     error: 'Invalid email format'
   }

3. ONLY User A - Tab 1 receives this message
   ✅ Tab 1 shows error
   ❌ Tab 2 (User A) does NOT receive (different connection)
   ❌ User B's tab does NOT receive (different connection)
```

### User A - Tab 1 creates a new post (entity sync):

```
1. Client sends:
   {
     type: 'push',
     entity: 'posts',
     entity_id: '123',
     action: 'create',
     data: { title: 'Hello' }
   }

2. Server broadcasts to ALL connections:
   {
     type: 'change',
     change: { entity: 'posts', id: '123', action: 'create', ... }
   }

3. ALL tabs receive this:
   ✅ User A - Tab 1 (updates IndexedDB)
   ✅ User A - Tab 2 (updates IndexedDB)
   ✅ User B - Tab 1 (updates IndexedDB)
```

## Summary

### Validation Routing (Targeted)
- ✅ Each tab has unique `client_id`
- ✅ Each form has unique `form_instance`
- ✅ Server sends validation ONLY to requesting client
- ✅ Request-response correlation via `request_id`
- ✅ No broadcasting of validation results

### Entity/Field Sync (Broadcast)
- ✅ Changes broadcast to ALL connected clients
- ✅ All tabs stay in sync
- ✅ Multi-tab updates via BroadcastChannel

### Benefits
1. **Correct routing**: Validation goes to right tab
2. **Privacy**: Users don't see each other's validation errors
3. **Scalability**: 1000 forms, no problem
4. **Multi-tab**: Same user can have multiple forms open
5. **One connection**: All message types multiplexed

## Testing Checklist

- [ ] Open 2 tabs with same form → each validates independently
- [ ] Open 2 tabs with different forms → no cross-contamination
- [ ] Validate field in Tab 1 → Tab 2 doesn't show error
- [ ] Create entity in Tab 1 → Tab 2 receives sync update
- [ ] 1000 forms on one page → each validates correctly
- [ ] Disconnect/reconnect → pending validations handled gracefully

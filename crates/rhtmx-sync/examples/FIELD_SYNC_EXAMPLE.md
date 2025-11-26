# Field-Level Sync Example

This example demonstrates how to use field-level synchronization in rhtmx-sync.

## Overview

Field-level sync allows you to track and synchronize individual field changes instead of entire entities. This is similar to CRDTs like Yjs or Automerge, enabling fine-grained conflict resolution.

## Server Setup

```rust
use rhtmx_sync::{SyncEngine, SyncConfig, FieldMergeStrategy};
use sqlx::SqlitePool;
use axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize database
    let db_pool = SqlitePool::connect("sqlite://app.db").await?;

    // Create sync engine with field-level sync enabled
    let sync_engine = SyncEngine::new(
        SyncConfig::new(db_pool.clone(), vec!["users".to_string()])
            .with_field_sync(FieldMergeStrategy::LastWriteWins)
    ).await?;

    // Get field tracker for manual operations
    let field_tracker = sync_engine.field_tracker().unwrap();

    // Record field changes manually
    field_tracker.record_field_change(
        "users",
        "1",
        "name",
        Some(serde_json::json!("Alice")),
        rhtmx_sync::FieldAction::Update,
        None,
    ).await?;

    // Create router with sync routes
    let app = Router::new()
        .merge(your_routes())
        .merge(sync_engine.routes());

    // ... serve app

    Ok(())
}
```

## Client Setup

### HTML

```html
<!DOCTYPE html>
<html>
<head>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>

    <!-- Field-level sync client -->
    <script src="/api/sync/field-client.js"
            data-sync-entities="users"
            data-field-strategy="last-write-wins"
            data-debug="true">
    </script>
</head>
<body>
    <div id="user-editor">
        <input type="text"
               id="user-name"
               placeholder="Name"
               onchange="updateField('name', this.value)">

        <input type="email"
               id="user-email"
               placeholder="Email"
               onchange="updateField('email', this.value)">
    </div>

    <script>
        function updateField(field, value) {
            // Record field change for user with ID '1'
            window.RHTMXFieldSync.recordFieldChange('users', '1', field, value);
        }

        // Listen for field conflicts
        window.addEventListener('rhtmx:field:conflict', (e) => {
            const conflict = e.detail;
            console.warn('Conflict detected:', conflict);

            // You can implement custom resolution UI here
            alert(`Conflict on ${conflict.field}:
                   Server: ${conflict.server_value}
                   Client: ${conflict.client_value}`);
        });
    </script>
</body>
</html>
```

## API Examples

### Get Field Changes

```bash
curl http://localhost:3000/api/field-sync/users?since=0
```

Response:
```json
{
  "entity": "users",
  "version": 5,
  "changes": [
    {
      "id": 1,
      "entity": "users",
      "entity_id": "1",
      "field": "name",
      "value": "Alice",
      "action": "update",
      "version": 1,
      "client_id": null,
      "timestamp": "2024-01-01T12:00:00Z"
    },
    {
      "id": 2,
      "entity": "users",
      "entity_id": "1",
      "field": "email",
      "value": "alice@example.com",
      "action": "update",
      "version": 2,
      "client_id": null,
      "timestamp": "2024-01-01T12:01:00Z"
    }
  ]
}
```

### Push Field Changes

```bash
curl -X POST http://localhost:3000/api/field-sync/users \
  -H "Content-Type: application/json" \
  -d '{
    "changes": [
      {
        "entity_id": "1",
        "field": "name",
        "value": "Bob",
        "action": "update",
        "timestamp": "2024-01-01T12:05:00Z"
      }
    ]
  }'
```

Response:
```json
{
  "applied": [
    {
      "id": 6,
      "entity": "users",
      "entity_id": "1",
      "field": "name",
      "value": "Bob",
      "action": "update",
      "version": 6,
      "client_id": null,
      "timestamp": "2024-01-01T12:05:00Z"
    }
  ],
  "conflicts": []
}
```

### Get Latest Field Values

```bash
curl http://localhost:3000/api/field-sync/users/1/latest
```

Response:
```json
{
  "name": "Bob",
  "email": "alice@example.com",
  "age": 30
}
```

## Conflict Resolution Example

When two clients modify the same field simultaneously:

**Client A** (timestamp: 12:00:00):
```javascript
window.RHTMXFieldSync.recordFieldChange('users', '1', 'name', 'Alice');
```

**Client B** (timestamp: 12:00:01):
```javascript
window.RHTMXFieldSync.recordFieldChange('users', '1', 'name', 'Bob');
```

With `LastWriteWins` strategy, the server will:
1. Accept both changes
2. Compare timestamps
3. Keep the newer value ("Bob" at 12:00:01)
4. Notify Client A of the conflict via the conflict event

## Merge Strategies

### LastWriteWins (Default)
```rust
SyncConfig::new(db_pool, vec!["users".to_string()])
    .with_field_sync(FieldMergeStrategy::LastWriteWins)
```
- Newest timestamp wins
- Automatic resolution
- Good for most use cases

### KeepBoth
```rust
SyncConfig::new(db_pool, vec!["users".to_string()])
    .with_field_sync(FieldMergeStrategy::KeepBoth)
```
- Reports conflict to client
- Application decides resolution
- Good for critical data

### ServerWins
```rust
SyncConfig::new(db_pool, vec!["users".to_string()])
    .with_field_sync(FieldMergeStrategy::ServerWins)
```
- Server value always preferred
- No client overwrites
- Good for admin-controlled data

### ClientWins
```rust
SyncConfig::new(db_pool, vec!["users".to_string()])
    .with_field_sync(FieldMergeStrategy::ClientWins)
```
- Client changes always win
- Optimistic updates
- Good for user-owned data

## Benefits

1. **Concurrent Editing**: Multiple users can edit different fields without conflicts
2. **Bandwidth Efficient**: Only changed fields are synced
3. **Better UX**: Users don't lose their changes as often
4. **CRDT-like**: Similar behavior to collaborative editing tools

## Use Cases

- **Collaborative Forms**: Multiple people editing different parts of a form
- **Real-time Dashboards**: Different users updating different metrics
- **Document Editing**: Field-by-field document updates
- **Profile Management**: Users editing their own profiles concurrently

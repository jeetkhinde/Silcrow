# CRDT Libraries vs rhtmx-sync

## Comparison: Automerge, Yjs, and rhtmx-sync

### What They Are

**Automerge** - CRDT library with full history and time travel
- Used by: Actual (budgeting app), Pushpin (collaborative workspace)
- Language: Rust (core) + JavaScript bindings
- Size: ~150KB (Wasm + JS)

**Yjs** - CRDT with focus on performance and rich text
- Used by: Jupyter, BlockSuite, Tiptap, many collaborative editors
- Language: JavaScript/TypeScript
- Size: ~35KB minified

**rhtmx-sync** - Timestamp-based sync with field-level granularity
- Used by: RHTMX applications
- Language: Rust + JavaScript
- Size: ~12KB

---

## Feature Comparison

| Feature | rhtmx-sync | Yjs | Automerge |
|---------|------------|-----|-----------|
| **Conflict Resolution** | Strategy-based (LWW, etc.) | Automatic (CRDT) | Automatic (CRDT) |
| **Text Editing** | ❌ Field-level only | ✅ Character-level OT | ✅ Character-level CRDT |
| **Time Travel/Undo** | ❌ | ✅ Full history | ✅ Full history |
| **SQL Backend** | ✅ PostgreSQL/SQLite | ❌ Binary format | ❌ Binary format |
| **Offline Support** | ✅ Queue mutations | ✅ Full offline | ✅ Full offline |
| **Real-time Sync** | ✅ WebSocket/SSE | ✅ WebSocket/WebRTC | ✅ WebSocket/WebRTC |
| **Bundle Size** | ~12KB | ~35KB | ~150KB |
| **Learning Curve** | Low | Medium | High |
| **HTMX Integration** | ✅ Built-in | ❌ Manual | ❌ Manual |
| **IndexedDB** | ✅ Automatic | ⚠️ Manual | ⚠️ Via adapters |
| **Multi-tab Sync** | ✅ BroadcastChannel | ✅ BroadcastChannel | ✅ BroadcastChannel |
| **Compression** | ✅ Gzip | ✅ Delta compression | ✅ Columnar encoding |
| **Peer-to-Peer** | ❌ Server required | ✅ Optional server | ✅ Optional server |
| **Branching/Merging** | ❌ | ⚠️ Limited | ✅ Git-like |
| **Provenance** | ⚠️ client_id only | ✅ Actor-based | ✅ Full history |
| **Query Support** | ✅ SQL queries | ❌ In-memory only | ❌ In-memory only |

---

## Use Case Analysis

### When rhtmx-sync is Better

#### 1. **Traditional CRUD Apps**
```rust
// rhtmx-sync: Simple, works great
#[derive(Syncable)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// SQL queries work normally
let users = sqlx::query!("SELECT * FROM users WHERE active = true")
    .fetch_all(&pool)
    .await?;
```

**With Yjs/Automerge:** You'd need a separate SQL sync layer, losing CRDT benefits for queries.

#### 2. **SQL-First Applications**
```sql
-- Complex joins work naturally
SELECT u.name, COUNT(p.id) as post_count
FROM users u
LEFT JOIN posts p ON u.id = p.user_id
GROUP BY u.id
HAVING post_count > 10;
```

**With Yjs/Automerge:** CRDTs store data in binary format, making SQL queries difficult/impossible.

#### 3. **Simple Conflict Resolution**
```rust
// rhtmx-sync: Timestamp-based (simple, predictable)
SyncStrategy::LastWriteWins  // Newest wins
```

**When you DON'T need:** Complex merging of concurrent text edits.

#### 4. **Lightweight Bundle Size**
- rhtmx-sync: ~12KB
- Yjs: ~35KB (3x larger)
- Automerge: ~150KB (12x larger)

For simple sync, rhtmx-sync is more efficient.

---

### When Yjs is Better

#### 1. **Collaborative Text Editing**
```javascript
// Yjs: Character-level conflict resolution
const ytext = ydoc.getText('content');

// User A types: "Hello"
ytext.insert(0, 'Hello');

// User B concurrently types at position 2: "XX"
ytext.insert(2, 'XX');

// Result: "HeXXllo" (both edits preserved!)
```

**rhtmx-sync:** Would have a conflict - last write wins, one edit lost.

#### 2. **Rich Text Editors**
Yjs integrates with:
- ProseMirror
- Quill
- Monaco (VS Code editor)
- CodeMirror
- TipTap

```javascript
import * as Y from 'yjs';
import { yCollab } from 'y-codemirror.next';

const ydoc = new Y.Doc();
const ytext = ydoc.getText('codemirror');
const provider = new WebsocketProvider('ws://localhost:1234', 'my-doc', ydoc);

// Collaborative code editing just works
```

**rhtmx-sync:** Not designed for this.

#### 3. **Real-time Collaboration (Google Docs-like)**
```javascript
// Multiple users editing simultaneously
// Yjs handles:
// - Concurrent character insertions
// - Formatting conflicts
// - Cursor positions
// - Selection highlighting
```

#### 4. **Performance-Critical Apps**
- Yjs uses delta compression (only sends changes)
- Structural sharing (efficient memory)
- Optimized for 60fps collaboration

---

### When Automerge is Better

#### 1. **Time Travel / Full History**
```rust
use automerge::AutoCommit;

let mut doc = AutoCommit::new();
doc.put(ROOT, "name", "Alice")?;
doc.commit();

doc.put(ROOT, "name", "Bob")?;
doc.commit();

// Travel back in time
let history = doc.get_all_changes();
for change in history {
    println!("At {}: {:?}", change.timestamp(), change);
}
```

**rhtmx-sync:** Only stores current state + recent change log.

#### 2. **Git-like Branching/Merging**
```rust
// Fork document
let mut branch1 = doc.fork();
let mut branch2 = doc.fork();

// Make changes in parallel
branch1.put(ROOT, "field1", "value1")?;
branch2.put(ROOT, "field2", "value2")?;

// Merge automatically
doc.merge(&mut branch1)?;
doc.merge(&mut branch2)?;
// No conflicts!
```

**rhtmx-sync:** No branching support.

#### 3. **Offline-First with Complex Merging**
```rust
// User works offline for days
// Makes hundreds of changes
// Reconnects and syncs
// All changes merge automatically (conflict-free)
```

**rhtmx-sync:** Works offline but uses timestamp-based conflict resolution.

#### 4. **Audit Trail / Compliance**
```rust
// Full provenance - who changed what, when
let changes = doc.get_changes(&[]);
for change in changes {
    println!("Actor: {}, Time: {}, Ops: {:?}",
        change.actor_id(),
        change.timestamp(),
        change.operations()
    );
}
```

---

## Hybrid Approach: Best of Both Worlds

### Architecture: Use Both!

```
┌─────────────────────────────────────────────────────┐
│                    Application                       │
├─────────────────────────────────────────────────────┤
│  rhtmx-sync                  Yjs/Automerge          │
│  ├─ User CRUD               ├─ Rich text editor     │
│  ├─ Post CRUD               ├─ Canvas/whiteboard    │
│  ├─ Comments CRUD           ├─ Spreadsheet cells    │
│  ├─ Form validation         └─ Real-time collab     │
│  └─ SQL queries                                      │
└─────────────────────────────────────────────────────┘
```

### Example: Blogging Platform

```rust
// Use rhtmx-sync for structured data
#[derive(Syncable)]
struct Post {
    id: i32,
    title: String,
    author_id: i32,
    created_at: DateTime<Utc>,
    published: bool,
}

// Use Yjs for collaborative content editing
// <div id="editor"></div>
// <script>
//   const ydoc = new Y.Doc();
//   const ytext = ydoc.getText('content');
//   // ... TipTap editor with Yjs binding
// </script>

// When user saves post:
// 1. Save Yjs document to binary blob
// 2. Sync post metadata via rhtmx-sync
async fn save_post(post: Post, content: Vec<u8>) {
    // Store Yjs binary in database
    sqlx::query!(
        "UPDATE posts SET title = ?, yjs_content = ? WHERE id = ?",
        post.title, content, post.id
    ).execute(&pool).await?;

    // Sync metadata via rhtmx-sync
    sync_engine.tracker().record_change(
        "posts", &post.id.to_string(),
        ChangeAction::Update,
        serde_json::to_value(&post)?
    ).await?;
}
```

---

## Technical Deep Dive

### How They Handle Conflicts

#### rhtmx-sync: Timestamp + Strategy
```rust
// Two users edit same field
User A: { field: "name", value: "Alice", timestamp: 12:00:00 }
User B: { field: "name", value: "Bob",   timestamp: 12:00:01 }

// Strategy: LastWriteWins
Result: "Bob" (newer timestamp)

// Simple, predictable, but one edit is lost
```

#### Yjs: Operational Transformation
```javascript
// Two users edit same text
User A: Insert "Hello" at position 0
User B: Insert "World" at position 0 (concurrent)

// Yjs transforms operations
// Considers operation order, client IDs, lamport timestamps
Result: Both edits preserved in consistent order across all clients
```

#### Automerge: CRDT with History
```rust
// Two users edit same field
User A: Set name = "Alice" (change 1)
User B: Set name = "Bob"   (change 2, concurrent)

// Automerge keeps both in history
// Deterministic merge rule (e.g., actor ID sort)
Result: "Bob" (deterministic, everyone agrees)
// But history contains both changes
```

### Data Storage

#### rhtmx-sync
```sql
-- SQL table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT,
    email TEXT,
    updated_at TIMESTAMP
);

-- Change log
CREATE TABLE rusty_sync_changelog (
    id INTEGER PRIMARY KEY,
    entity TEXT,
    entity_id TEXT,
    action TEXT,
    data JSON,
    version INTEGER,
    timestamp TIMESTAMP
);
```

**Pros:** SQL queries work, familiar, easy to inspect
**Cons:** No true merge, conflicts possible

#### Yjs
```javascript
// Binary encoding (lib0)
// Stores operations + structure sharing
const ydoc = new Y.Doc();
const ymap = ydoc.getMap('users');
ymap.set('name', 'Alice');

// Encode to binary
const update = Y.encodeStateAsUpdate(ydoc);
// Store in database as BLOB
await db.query('INSERT INTO yjs_docs (id, data) VALUES (?, ?)', [docId, update]);
```

**Pros:** Efficient, conflict-free, full history
**Cons:** Can't query with SQL, must load into memory

#### Automerge
```rust
// Similar to Yjs but Rust-native
let mut doc = AutoCommit::new();
doc.put(ROOT, "name", "Alice")?;

// Save to binary
let bytes = doc.save();
// Store as BLOB
sqlx::query!("INSERT INTO automerge_docs VALUES (?, ?)", id, bytes)
    .execute(&pool).await?;
```

**Pros:** Full history, branching, Rust-native
**Cons:** Large binary size, no SQL queries

---

## Performance Comparison

### Bundle Size (Gzipped)
- **rhtmx-sync:** 12 KB
- **Yjs:** 35 KB
- **Automerge:** 150 KB (with Wasm)

### Memory Usage (1000 documents)
- **rhtmx-sync:** Low (only current state)
- **Yjs:** Medium (operations log + structure)
- **Automerge:** High (full history)

### Sync Speed (1000 concurrent edits)
- **rhtmx-sync:** Fast (simple timestamp comparison)
- **Yjs:** Very fast (delta compression, optimized)
- **Automerge:** Medium (columnar encoding)

---

## Migration Path

### Option 1: Replace rhtmx-sync with Yjs
**Don't do this** unless you need:
- Collaborative rich text editing
- Character-level conflict resolution
- Real-time multi-user editing

You'd lose:
- SQL queries
- Simple timestamp-based sync
- HTMX integration
- Smaller bundle size

### Option 2: Hybrid (Recommended)
**Use both** for different purposes:
```
rhtmx-sync: CRUD operations, forms, validation, SQL queries
Yjs/Automerge: Collaborative editors, canvases, real-time co-editing
```

### Option 3: Extend rhtmx-sync with CRDT Features
**Add CRDT support** to rhtmx-sync for specific fields:
```rust
#[derive(Syncable)]
struct Document {
    id: i32,
    title: String,              // rhtmx-sync (LWW)
    #[crdt(yjs)]
    content: YjsDocument,       // Yjs for rich text
    author_id: i32,             // rhtmx-sync (LWW)
}
```

This is complex but gives best of both worlds.

---

## Recommendation

### For Your Use Case (Forms + Validation)

**Stick with rhtmx-sync** because:

1. ✅ **Form validation doesn't need CRDTs**
   - Validation is per-field, not concurrent editing
   - Timestamp-based conflict resolution is fine

2. ✅ **SQL queries are valuable**
   - Filter forms by status
   - Aggregate form submissions
   - Complex reports

3. ✅ **Smaller bundle size**
   - 12KB vs 35KB (Yjs) or 150KB (Automerge)
   - Faster page loads

4. ✅ **Simpler mental model**
   - Timestamps + conflict strategies
   - No CRDT algorithms to understand

5. ✅ **HTMX integration**
   - Already built for your stack

### When to Add Yjs/Automerge

Consider adding them **only if** you need:
- ✅ Rich text collaborative editing (Google Docs-like)
- ✅ Collaborative drawing/whiteboard
- ✅ Real-time code editing
- ✅ Spreadsheet-like concurrent cell editing
- ✅ Time travel / undo across sessions

For most CRUD apps with forms: **rhtmx-sync is better**.

---

## Code Example: Hybrid Approach

```html
<!DOCTYPE html>
<html>
<head>
    <!-- rhtmx-sync for CRUD -->
    <script src="/api/sync/client.js"
            data-sync-entities="posts,users">
    </script>

    <!-- Yjs for collaborative editor (optional) -->
    <script src="https://cdn.jsdelivr.net/npm/yjs@13/dist/yjs.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/y-websocket@1/dist/y-websocket.min.js"></script>
</head>
<body>
    <!-- Form: Use rhtmx-sync -->
    <form id="create-post" data-form-type="post" data-validate>
        <input name="title" type="text">
        <!-- Validated via rhtmx-sync WebSocket -->

        <input name="author_id" type="number">
        <button type="submit">Create Post</button>
    </form>

    <!-- Collaborative editor: Use Yjs -->
    <div id="collaborative-editor"></div>

    <script>
        // rhtmx-sync handles form submission
        document.getElementById('create-post').addEventListener('submit', async (e) => {
            e.preventDefault();

            const title = e.target.title.value;
            const content = ydoc.getText('content').toString(); // Get Yjs content

            // Create post via rhtmx-sync
            await fetch('/api/posts', {
                method: 'POST',
                body: JSON.stringify({ title, content })
            });
        });

        // Yjs handles collaborative editing
        const ydoc = new Y.Doc();
        const ytext = ydoc.getText('content');
        const provider = new WebsocketProvider(
            'ws://localhost:1234',
            'my-document',
            ydoc
        );

        // Bind Yjs to editor (TipTap, ProseMirror, etc.)
        // ...
    </script>
</body>
</html>
```

---

## Summary Table

| Aspect | rhtmx-sync | Yjs | Automerge |
|--------|------------|-----|-----------|
| **Best For** | CRUD apps, forms | Collaborative text | Offline-first, history |
| **Complexity** | Low | Medium | High |
| **Bundle Size** | 12KB | 35KB | 150KB |
| **SQL Support** | ✅ Native | ❌ | ❌ |
| **Conflict-Free** | ❌ Strategy-based | ✅ | ✅ |
| **HTMX Ready** | ✅ | ❌ | ❌ |
| **When to Use** | Always (for CRUD) | Add for rich text | Add for time travel |

**Bottom line:** rhtmx-sync is perfect for your use case. Add Yjs/Automerge **only if** you need collaborative editing features.

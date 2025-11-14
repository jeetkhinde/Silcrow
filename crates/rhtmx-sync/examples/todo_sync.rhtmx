// File: rhtmx-sync/examples/todo_sync.rs
// Purpose: Example showing rhtmx-sync in action

use axum::{
    routing::get,
    Router,
};
use rhtmx_macro::Syncable;
use rhtmx_sync::{Syncable as SyncableTrait, SyncEngine, SyncConfig};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;

/// Todo item model with Syncable derive
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Syncable)]
#[sync(table = "todos")]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Initialize database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await?;

    // Create todos table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Insert sample data
    sqlx::query("INSERT INTO todos (title, completed) VALUES ('Buy milk', 0)")
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO todos (title, completed) VALUES ('Write code', 1)")
        .execute(&pool)
        .await?;

    // Initialize sync engine
    let sync_engine = SyncEngine::new(SyncConfig::new(
        pool.clone(),
        vec!["todos".to_string()],
    ))
    .await?;

    // Build app with sync routes
    let app = Router::new()
        .route("/", get(index))
        .merge(sync_engine.routes());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running at http://{}", addr);
    println!("📦 Sync enabled for: todos");
    println!("📡 SSE endpoint: http://{}/api/sync/events", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Index page with HTMX and rhtmx-sync
async fn index() -> &'static str {
    r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RHTMX Sync - Todo Example</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <script src="/api/sync/client.js"
            data-sync-entities="todos"
            data-debug="true">
    </script>
    <style>
        body {
            font-family: system-ui, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
        }
        .todo {
            padding: 10px;
            margin: 5px 0;
            border: 1px solid #ddd;
            border-radius: 4px;
        }
        .todo.completed {
            text-decoration: line-through;
            opacity: 0.6;
        }
        .status {
            position: fixed;
            top: 10px;
            right: 10px;
            padding: 10px 20px;
            background: #4CAF50;
            color: white;
            border-radius: 4px;
        }
        .status.offline {
            background: #f44336;
        }
        button {
            padding: 8px 16px;
            margin: 5px;
            cursor: pointer;
        }
    </style>
</head>
<body>
    <h1>📝 RHTMX Sync - Todo Example</h1>

    <div id="status" class="status">
        <span id="online-status">Online</span>
    </div>

    <div>
        <h2>Features Enabled:</h2>
        <ul>
            <li>✅ IndexedDB caching</li>
            <li>✅ Real-time updates (SSE)</li>
            <li>✅ Offline support</li>
            <li>✅ Auto-sync on reconnect</li>
        </ul>
    </div>

    <div>
        <h2>Try These:</h2>
        <ol>
            <li>Open this page in multiple tabs - see real-time sync</li>
            <li>Open DevTools → Network → Offline to test offline mode</li>
            <li>Add/modify todos while offline</li>
            <li>Go back online - watch automatic sync!</li>
        </ol>
    </div>

    <div>
        <h2>Todos</h2>
        <div id="todos"
             hx-get="/api/todos"
             hx-trigger="load, rhtmx:todos:changed from:body">
            Loading...
        </div>
    </div>

    <div>
        <h2>Actions</h2>
        <button onclick="addTodo()">Add Random Todo</button>
        <button onclick="showDB()">Show IndexedDB</button>
        <button onclick="clearDB()">Clear IndexedDB</button>
    </div>

    <div id="debug" style="margin-top: 20px; padding: 10px; background: #f5f5f5; border-radius: 4px;">
        <h3>Debug Info</h3>
        <pre id="debug-output"></pre>
    </div>

    <script>
        // Update online/offline status
        function updateStatus() {
            const status = document.getElementById('status');
            const text = document.getElementById('online-status');
            if (navigator.onLine) {
                status.classList.remove('offline');
                text.textContent = 'Online';
            } else {
                status.classList.add('offline');
                text.textContent = 'Offline';
            }
        }

        window.addEventListener('online', updateStatus);
        window.addEventListener('offline', updateStatus);
        updateStatus();

        // Debug: show when sync is ready
        document.addEventListener('rhtmx:sync:ready', () => {
            debug('✅ RHTMX Sync initialized!');
        });

        // Debug: show entity changes
        document.addEventListener('rhtmx:todos:changed', (e) => {
            debug(`📡 Todo changed: ${e.detail.id}`);
        });

        function debug(msg) {
            const output = document.getElementById('debug-output');
            const time = new Date().toLocaleTimeString();
            output.textContent = `[${time}] ${msg}\n` + output.textContent;
        }

        function addTodo() {
            const titles = [
                'Learn Rust',
                'Build awesome app',
                'Test offline sync',
                'Deploy to production',
                'Celebrate success'
            ];
            const title = titles[Math.floor(Math.random() * titles.length)];

            // This would normally be an HTMX post
            fetch('/api/todos', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ title, completed: false })
            }).then(() => {
                debug(`➕ Added: ${title}`);
                htmx.trigger('#todos', 'rhtmx:todos:changed');
            });
        }

        async function showDB() {
            if (!window.indexedDB) {
                alert('IndexedDB not supported');
                return;
            }

            const db = await openDB();
            const tx = db.transaction('todos', 'readonly');
            const store = tx.objectStore('todos');
            const todos = await getAll(store);

            debug(`📦 IndexedDB has ${todos.length} todos`);
            console.log('IndexedDB todos:', todos);
        }

        async function clearDB() {
            if (confirm('Clear IndexedDB cache?')) {
                const db = await openDB();
                const tx = db.transaction('todos', 'readwrite');
                const store = tx.objectStore('todos');
                await clear(store);
                debug('🗑️ IndexedDB cleared');
            }
        }

        function openDB() {
            return new Promise((resolve, reject) => {
                const request = indexedDB.open('rhtmx-cache', 1);
                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            });
        }

        function getAll(store) {
            return new Promise((resolve, reject) => {
                const request = store.getAll();
                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            });
        }

        function clear(store) {
            return new Promise((resolve, reject) => {
                const request = store.clear();
                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error);
            });
        }
    </script>
</body>
</html>
    "#
}

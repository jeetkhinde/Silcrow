# PostgreSQL Setup Guide

rusty-sync now supports PostgreSQL as the **primary database** with Diesel ORM, while maintaining optional SQLite support for development.

## Why PostgreSQL?

- **Real-time notifications**: Built-in LISTEN/NOTIFY for instant updates
- **Scalability**: Production-ready for high-traffic applications
- **Concurrent writes**: Better handling of simultaneous clients
- **Cloud-ready**: Perfect for platforms like Supabase, Railway, Render

## Quick Start with PostgreSQL

### 1. Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features postgres
```

### 2. Set Database URL

```bash
export DATABASE_URL="postgresql://user:password@localhost/mydb"
```

### 3. Run Migrations

```bash
cd crates/rusty-sync
diesel migration run
```

This creates the necessary tables:
- `_rusty_sync_log` - Entity-level change tracking
- `_rhtmx_field_sync_log` - Field-level change tracking

### 4. Update Your Code

```rust
use rusty_sync::{DbPool, SyncEngine, SyncConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create PostgreSQL connection pool
    let database_url = std::env::var("DATABASE_URL")?;
    let db_pool = DbPool::from_url(&database_url).await?;

    // Initialize sync engine with PostgreSQL
    let sync_config = SyncConfig::new(
        db_pool,
        vec!["users".to_string(), "posts".to_string()]
    );

    let sync_engine = SyncEngine::new(sync_config).await?;

    // Add routes to your Axum app
    let app = axum::Router::new()
        .merge(sync_engine.routes());

    // ...
    Ok(())
}
```

## Database Configuration

### Development (SQLite)

```rust
// SQLite for local development
let db_pool = DbPool::from_url("sqlite://./dev.db").await?;
```

### Production (PostgreSQL)

```rust
// PostgreSQL for production
let db_pool = DbPool::from_url(&database_url).await?;
```

### Automatic Detection

`DbPool::from_url()` automatically detects the database type based on the URL scheme:
- `postgresql://` or `postgres://` → PostgreSQL
- `sqlite://` or `*.db` → SQLite

## Migration Management

### Creating New Migrations

```bash
diesel migration generate my_migration_name
```

### Running Migrations

```bash
# PostgreSQL
diesel migration run

# SQLite (development)
diesel migration run --migration-dir migrations_sqlite
```

### Reverting Migrations

```bash
diesel migration revert
```

## PostgreSQL-Specific Features

### LISTEN/NOTIFY for Real-time Updates

PostgreSQL migrations automatically create triggers for real-time notifications:

```sql
-- Automatically created by migrations
CREATE TRIGGER sync_change_notify
AFTER INSERT ON _rusty_sync_log
FOR EACH ROW
EXECUTE FUNCTION notify_sync_change();
```

### Using LISTEN/NOTIFY in Rust

```rust
use rusty_sync::postgres_notify::{start_entity_sync_listener, start_field_sync_listener};

// Start listening for entity-level changes
let entity_listener = start_entity_sync_listener(
    &database_url,
    &["users", "posts"]
).await?;

// Start listening for field-level changes
let field_listener = start_field_sync_listener(
    &database_url,
    &["users", "posts"]
).await?;

// Subscribe to notifications
let mut rx = entity_listener.subscribe();

// Process notifications
tokio::spawn(async move {
    while let Ok(notification) = rx.recv().await {
        println!("Received notification: {:?}", notification);
        // Forward to WebSocket clients, etc.
    }
});
```

## Connection Pooling

PostgreSQL uses `bb8` connection pooling via `diesel-async`:

```rust
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::AsyncPgConnection;

// Configure pool (optional - defaults are good for most cases)
let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
let pool = Pool::builder()
    .max_size(15)  // Max connections
    .min_idle(Some(5))  // Min idle connections
    .build(config)
    .await?;

let db_pool = DbPool::Postgres(pool);
```

## Performance Tips

### 1. Use Indexes

The migrations create necessary indexes, but you may want to add custom ones:

```sql
CREATE INDEX idx_custom_query ON _rusty_sync_log(entity, created_at)
WHERE action = 'create';
```

### 2. Connection Pooling

For high-traffic applications:

```rust
// Increase pool size for high concurrency
let pool = Pool::builder()
    .max_size(50)  // Adjust based on your needs
    .connection_timeout(Duration::from_secs(10))
    .build(config)
    .await?;
```

### 3. Cleanup Old Entries

Periodically clean up old sync log entries:

```rust
// Clean up entries older than 30 days
let deleted = change_tracker.cleanup_old_entries(30).await?;
println!("Cleaned up {} old entries", deleted);
```

## Docker Compose for Development

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: rhtmx
      POSTGRES_PASSWORD: rhtmx_dev
      POSTGRES_DB: rusty_sync
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

Run with:

```bash
docker-compose up -d
export DATABASE_URL="postgresql://rhtmx:rhtmx_dev@localhost/rusty_sync"
diesel migration run
```

## Testing with PostgreSQL

Run integration tests:

```bash
# Start PostgreSQL
docker-compose up -d

# Run tests
DATABASE_URL="postgresql://rhtmx:rhtmx_dev@localhost/rusty_sync" \
  cargo test --test postgres_integration
```

## Troubleshooting

### Connection Errors

```
Error: Failed to create PostgreSQL connection pool
```

**Solutions:**
1. Verify PostgreSQL is running: `pg_isready -h localhost`
2. Check credentials in DATABASE_URL
3. Ensure database exists: `createdb mydb`

### Migration Errors

```
Error: Migrations not found
```

**Solutions:**
1. Ensure you're in the correct directory: `cd crates/rusty-sync`
2. Check `diesel.toml` exists
3. Run: `diesel setup`

### Permission Errors

```
Error: permission denied for table _rusty_sync_log
```

**Solutions:**
1. Grant necessary permissions:
```sql
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO your_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO your_user;
```

## Next Steps

- [Supabase Integration Guide](./supabase-integration.md) - Deploy to Supabase
- [Field-Level Sync](../README.md#field-level-sync) - Enable CRDT-like field sync
- [WebSocket Configuration](../README.md#websocket-sync) - Configure real-time sync

## Resources

- [Diesel Documentation](https://diesel.rs/)
- [diesel-async](https://docs.rs/diesel-async/)
- [PostgreSQL LISTEN/NOTIFY](https://www.postgresql.org/docs/current/sql-notify.html)
- [POSTGRESQL_STATUS.md](../../POSTGRESQL_STATUS.md) - Implementation status

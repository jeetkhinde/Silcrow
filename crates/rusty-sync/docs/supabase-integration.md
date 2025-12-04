# Supabase Integration Guide

Deploy rusty-sync with Supabase for a fully managed PostgreSQL database with real-time capabilities.

## Why Supabase?

- **Managed PostgreSQL**: No server management required
- **Built-in real-time**: Native support for database changes
- **Connection pooling**: PgBouncer included
- **Auto-scaling**: Handles traffic spikes
- **Free tier**: Perfect for development and small projects

## Quick Setup

### 1. Create Supabase Project

1. Go to [supabase.com](https://supabase.com)
2. Create a new project
3. Wait for database provisioning (~2 minutes)
4. Note your database credentials

### 2. Get Connection String

In your Supabase dashboard:
1. Go to **Settings** → **Database**
2. Copy the **Connection string** (URI format)
3. Replace `[YOUR-PASSWORD]` with your actual password

Example:
```
postgresql://postgres:your_password@db.abcdefghij.supabase.co:5432/postgres
```

### 3. Set Environment Variable

```bash
export DATABASE_URL="postgresql://postgres:your_password@db.abcdefghij.supabase.co:5432/postgres"
```

### 4. Run Migrations

```bash
cd crates/rusty-sync
diesel migration run
```

This will create the necessary tables in your Supabase database:
- `_rusty_sync_log`
- `_rhtmx_field_sync_log`

### 5. Deploy Your Application

```rust
use rusty_sync::{DbPool, SyncEngine, SyncConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get Supabase connection string from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Create connection pool
    let db_pool = DbPool::from_url(&database_url).await?;

    // Initialize sync engine
    let sync_config = SyncConfig::new(
        db_pool,
        vec!["users".to_string(), "posts".to_string(), "comments".to_string()]
    );

    let sync_engine = SyncEngine::new(sync_config).await?;

    // Add routes
    let app = axum::Router::new()
        .merge(sync_engine.routes());

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## Configuration Options

### Connection Pooling

Supabase includes PgBouncer, but you can also configure client-side pooling:

```rust
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);

let pool = Pool::builder()
    .max_size(20)  // Adjust based on your plan
    .connection_timeout(Duration::from_secs(30))
    .build(config)
    .await?;

let db_pool = DbPool::Postgres(pool);
```

### Connection Limits by Plan

| Plan | Max Connections | Recommended Pool Size |
|------|----------------|----------------------|
| Free | 60 | 15 |
| Pro | 200 | 50 |
| Team | 400 | 100 |
| Enterprise | Custom | Custom |

## Supabase-Specific Features

### 1. Row Level Security (RLS)

Enable RLS for sync tables:

```sql
-- Enable RLS
ALTER TABLE _rusty_sync_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE _rhtmx_field_sync_log ENABLE ROW LEVEL SECURITY;

-- Allow authenticated users to read/write
CREATE POLICY "Allow authenticated access"
ON _rusty_sync_log
FOR ALL
TO authenticated
USING (true)
WITH CHECK (true);

CREATE POLICY "Allow authenticated access"
ON _rhtmx_field_sync_log
FOR ALL
TO authenticated
USING (true)
WITH CHECK (true);
```

### 2. Real-time Subscriptions

Supabase can notify your app of changes:

```rust
use rusty_sync::postgres_notify::start_entity_sync_listener;

// Subscribe to Supabase real-time events
let listener = start_entity_sync_listener(
    &database_url,
    &["users", "posts"]
).await?;

// Handle notifications
let mut rx = listener.subscribe();
tokio::spawn(async move {
    while let Ok(notification) = rx.recv().await {
        // Forward to WebSocket clients
        println!("Change detected: {:?}", notification);
    }
});
```

### 3. Database Functions

Create custom functions for complex operations:

```sql
CREATE OR REPLACE FUNCTION get_entity_changes(
    p_entity VARCHAR,
    p_since_version BIGINT
)
RETURNS SETOF _rusty_sync_log AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM _rusty_sync_log
    WHERE entity = p_entity
      AND version > p_since_version
    ORDER BY version ASC;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
```

## Deployment Patterns

### Pattern 1: Single Region

```rust
// Simple deployment - closest to your users
let database_url = env::var("DATABASE_URL")?;
let db_pool = DbPool::from_url(&database_url).await?;
```

### Pattern 2: Read Replicas

```rust
// Use Supabase read replicas for scaling reads
let primary_url = env::var("PRIMARY_DATABASE_URL")?;
let replica_url = env::var("REPLICA_DATABASE_URL")?;

// Write to primary
let write_pool = DbPool::from_url(&primary_url).await?;

// Read from replica
let read_pool = DbPool::from_url(&replica_url).await?;
```

### Pattern 3: Multi-Region

For global applications, deploy to multiple regions with Supabase:

```rust
// EU region
let eu_pool = DbPool::from_url(&env::var("EU_DATABASE_URL")?).await?;

// US region
let us_pool = DbPool::from_url(&env::var("US_DATABASE_URL")?).await?;

// Route based on user location
let db_pool = if user_in_europe() { eu_pool } else { us_pool };
```

## Platform-Specific Deployment

### Railway

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and link project
railway login
railway link

# Set Supabase URL
railway variables set DATABASE_URL="postgresql://..."

# Deploy
railway up
```

### Render

Create `render.yaml`:

```yaml
services:
  - type: web
    name: rusty-sync-api
    env: rust
    buildCommand: cargo build --release
    startCommand: ./target/release/your-app
    envVars:
      - key: DATABASE_URL
        value: postgresql://postgres:password@db.supabase.co:5432/postgres
```

### Fly.io

Create `fly.toml`:

```toml
[env]
  DATABASE_URL = "postgresql://postgres:password@db.supabase.co:5432/postgres"

[[services]]
  internal_port = 3000
  protocol = "tcp"

  [[services.ports]]
    handlers = ["http"]
    port = 80

  [[services.ports]]
    handlers = ["tls", "http"]
    port = 443
```

Deploy:

```bash
fly launch
fly deploy
```

## Monitoring & Debugging

### Enable Logging

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();
```

### Monitor Queries

Check slow queries in Supabase dashboard:
1. Go to **Logs** → **Query Performance**
2. Identify slow queries
3. Add indexes as needed

### Database Statistics

```sql
-- Check table sizes
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Check index usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC;
```

## Performance Optimization

### 1. Use Connection Pooling

```rust
// Configure for your workload
let pool = Pool::builder()
    .max_size(30)  // Match Supabase plan limits
    .min_idle(Some(5))
    .connection_timeout(Duration::from_secs(10))
    .build(config)
    .await?;
```

### 2. Batch Operations

```rust
// Record multiple changes in a transaction
for entity in entities {
    tracker.record_change(
        &entity.name,
        &entity.id,
        ChangeAction::Create,
        Some(entity.data),
        client_id.clone()
    ).await?;
}
```

### 3. Cleanup Strategy

```rust
// Schedule cleanup job
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_hours(24));
    loop {
        interval.tick().await;
        // Keep 30 days of history
        let _ = tracker.cleanup_old_entries(30).await;
    }
});
```

## Cost Optimization

### Free Tier Limits

- Database: 500 MB
- Bandwidth: 2 GB
- Max connections: 60

### Tips to Stay Within Limits

1. **Clean up old data regularly**:
   ```rust
   // Keep only 7 days for free tier
   tracker.cleanup_old_entries(7).await?;
   ```

2. **Use compression**:
   ```rust
   let sync_config = SyncConfig::new(db_pool, entities)
       .with_compression(CompressionConfig::default());
   ```

3. **Limit sync scope**:
   ```rust
   // Only sync recently changed entities
   tracker.get_changes_since(entity, recent_version).await?;
   ```

## Security Best Practices

### 1. Use Environment Variables

```rust
// Never hardcode credentials
let database_url = env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```

### 2. Enable SSL

Supabase enforces SSL by default. Ensure your connection string includes SSL:

```
postgresql://user:pass@host:5432/db?sslmode=require
```

### 3. Rotate Credentials

Regularly rotate your database password in Supabase dashboard:
1. **Settings** → **Database** → **Reset database password**
2. Update `DATABASE_URL` in your deployment

### 4. Use Service Keys Securely

```rust
// Different keys for different environments
let api_key = match env::var("ENVIRONMENT")?.as_str() {
    "production" => env::var("SUPABASE_PROD_KEY")?,
    "staging" => env::var("SUPABASE_STAGING_KEY")?,
    _ => env::var("SUPABASE_DEV_KEY")?,
};
```

## Troubleshooting

### Too Many Connections

```
Error: remaining connection slots are reserved
```

**Solutions:**
1. Reduce `max_size` in connection pool
2. Upgrade Supabase plan
3. Enable PgBouncer in Supabase settings

### Slow Queries

```
Error: query timeout
```

**Solutions:**
1. Add indexes for frequently queried columns
2. Use `EXPLAIN ANALYZE` to debug queries
3. Enable Supabase query performance insights

### Migration Failures

```
Error: relation "_rusty_sync_log" already exists
```

**Solutions:**
1. Check if tables already exist
2. Reset migrations: `diesel migration revert`
3. Drop tables manually if needed (⚠️ data loss)

## Example Project

Complete example with Supabase integration:

```bash
git clone https://github.com/jeetkhinde/RHTMX
cd RHTMX/examples/supabase-sync
cp .env.example .env
# Edit .env with your Supabase credentials
diesel migration run
cargo run
```

## Next Steps

- [PostgreSQL Setup](./postgresql-setup.md) - Detailed PostgreSQL guide
- [Field-Level Sync](../README.md#field-level-sync) - Enable granular sync
- [Production Deployment](../README.md#production) - Best practices

## Resources

- [Supabase Documentation](https://supabase.com/docs)
- [Supabase Pricing](https://supabase.com/pricing)
- [Connection Pooling](https://supabase.com/docs/guides/database/connecting-to-postgres#connection-pooler)
- [Row Level Security](https://supabase.com/docs/guides/auth/row-level-security)

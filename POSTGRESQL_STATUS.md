# PostgreSQL Support Status

**Branch:** `claude/add-compression-postgres-support-01R5jYBnuEUQmpe1xcCfZ5u4`
**Last Updated:** 2025-12-04
**Status:** 🟡 Partial Implementation (70% Complete)

## Overview

This document tracks the progress of adding PostgreSQL support to RHTMX using Diesel ORM. PostgreSQL is the **PRIMARY** database, while SQLite remains **OPTIONAL** for development and backward compatibility.

## Architecture

### Database Abstraction (`DbPool`)

```rust
pub enum DbPool {
    Postgres(Pool<AsyncPgConnection>),  // PRIMARY - diesel-async
    Sqlite(Arc<SqlitePool>),             // OPTIONAL - sqlx
}
```

The `DbPool` enum provides:
- Automatic database detection from URL schemes
- Type-safe connection pooling for both backends
- Methods to check database type and retrieve connections

### Migration Strategy

- **PostgreSQL**: Diesel migrations in `migrations/`
- **SQLite**: Legacy migrations in `migrations_sqlite/`
- Both maintain the same `_rhtmx_sync_log` schema

---

## ✅ Completed Work

### 1. Dependencies & Configuration
- [x] Added `diesel` v2.1 with PostgreSQL support
- [x] Added `diesel-async` v0.4 with async PostgreSQL support
- [x] Added `diesel_migrations` v2.1
- [x] Created `diesel.toml` configuration

**Files:**
- `crates/rhtmx-sync/Cargo.toml:37-40`
- `crates/rhtmx-sync/diesel.toml`

### 2. Database Migrations
- [x] Created PostgreSQL migration: `2024-11-28-000001_create_sync_log`
  - Creates `_rhtmx_sync_log` table with proper PostgreSQL types
  - Adds indexes for efficient querying
  - **PostgreSQL-specific:** LISTEN/NOTIFY trigger function
- [x] Created corresponding SQLite migration for backward compatibility

**Files:**
- `crates/rhtmx-sync/migrations/2024-11-28-000001_create_sync_log/up.sql`
- `crates/rhtmx-sync/migrations/2024-11-28-000001_create_sync_log/down.sql`
- `crates/rhtmx-sync/migrations_sqlite/2024-11-28-000001_create_sync_log/`

**PostgreSQL Features:**
```sql
-- Real-time notifications
CREATE OR REPLACE FUNCTION notify_sync_change() RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('_rhtmx_sync_' || NEW.entity, ...);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

### 3. Diesel Schema & Models
- [x] Generated Diesel schema definitions
- [x] Created `SyncLog` model (Queryable)
- [x] Created `NewSyncLog` model (Insertable)
- [x] Conversion methods between Diesel models and public API types

**Files:**
- `crates/rhtmx-sync/src/schema.rs`
- `crates/rhtmx-sync/src/models.rs`

### 4. Database Pool Abstraction
- [x] Created `DbPool` enum supporting both PostgreSQL and SQLite
- [x] Implemented `new_postgres()` for PostgreSQL pools
- [x] Implemented `new_sqlite()` for SQLite pools
- [x] Implemented `from_url()` for automatic detection
- [x] Type-safe connection retrieval methods

**Files:**
- `crates/rhtmx-sync/src/db.rs`

### 5. ChangeTracker Migration
- [x] Updated `ChangeTracker` to accept `DbPool`
- [x] Implemented all operations for PostgreSQL with Diesel:
  - `record_change_postgres()`
  - `get_changes_since_postgres()`
  - `latest_version_postgres()`
  - `cleanup_old_entries_postgres()`
- [x] Maintained SQLite implementations for backward compatibility
- [x] Added tests for both database backends

**Files:**
- `crates/rhtmx-sync/src/change_tracker.rs:45-416`

### 6. SyncEngine Updates
- [x] Updated `SyncConfig` to use `DbPool` instead of `SqlitePool`
- [x] `ChangeTracker` initialization uses `DbPool`

**Files:**
- `crates/rhtmx-sync/src/engine.rs:24-58`

### 7. Public API Exports
- [x] Exported `DbPool` from main library
- [x] All schema and model types available

**Files:**
- `crates/rhtmx-sync/src/lib.rs:45,54`

---

## 🔴 Pending Work

### Critical (Blocking)

#### 1. Update FieldTracker to Use DbPool
**Current Issue:** `FieldTracker` still uses `SqlitePool` directly, causing compilation error with `SyncEngine`.

**Location:** `crates/rhtmx-sync/src/field_tracker.rs:88-98`

**Required Changes:**
```rust
// Current (BROKEN)
pub struct FieldTracker {
    db_pool: Arc<SqlitePool>,
    // ...
}

impl FieldTracker {
    pub async fn new(
        db_pool: Arc<SqlitePool>,
        merge_strategy: FieldMergeStrategy,
    ) -> anyhow::Result<Self> { ... }
}

// Needed (FIXED)
pub struct FieldTracker {
    db_pool: Arc<DbPool>,
    // ...
}

impl FieldTracker {
    pub async fn new(
        db_pool: Arc<DbPool>,
        merge_strategy: FieldMergeStrategy,
    ) -> anyhow::Result<Self> { ... }
}
```

**Implementation Required:**
- [ ] Change `db_pool` type from `Arc<SqlitePool>` to `Arc<DbPool>`
- [ ] Add PostgreSQL migration for `_rhtmx_field_sync_log` table
- [ ] Implement Diesel models for field sync table
- [ ] Implement `record_field_change_postgres()`
- [ ] Implement `get_field_changes_postgres()`
- [ ] Implement `get_latest_fields_postgres()`
- [ ] Implement `apply_field_merge_postgres()`
- [ ] Maintain SQLite implementations for backward compatibility
- [ ] Update all method signatures and implementations

**Files to Modify:**
- `crates/rhtmx-sync/src/field_tracker.rs`
- `crates/rhtmx-sync/migrations/` (new migration for field sync table)
- `crates/rhtmx-sync/src/schema.rs` (add field sync table schema)
- `crates/rhtmx-sync/src/models.rs` (add field sync models)

#### 2. Fix SyncEngine Compilation
**Current Issue:** `SyncEngine` tries to pass `DbPool` to `FieldTracker::new()`, but it expects `SqlitePool`.

**Location:** `crates/rhtmx-sync/src/engine.rs:96-98`

**Status:** Will be fixed once FieldTracker is updated.

### High Priority

#### 3. Implement PostgreSQL LISTEN/NOTIFY
**Purpose:** Enable real-time notifications for sync changes without polling.

**Required Implementation:**
```rust
// New module: crates/rhtmx-sync/src/postgres_notify.rs
use diesel_async::AsyncPgConnection;
use tokio::sync::broadcast;

pub struct PostgresNotifyListener {
    // Listen to PostgreSQL notifications
    // Broadcast to WebSocket clients
}

impl PostgresNotifyListener {
    pub async fn start(pool: &DbPool) -> anyhow::Result<Self> {
        // Subscribe to pg_notify channels
        // Forward to broadcast channel
    }
}
```

**Integration Points:**
- `ChangeTracker`: Use LISTEN/NOTIFY instead of polling
- `WebSocketState`: Subscribe to PostgreSQL notifications
- `FieldWebSocketState`: Subscribe to field-level notifications

**Files to Create/Modify:**
- Create: `crates/rhtmx-sync/src/postgres_notify.rs`
- Modify: `crates/rhtmx-sync/src/change_tracker.rs`
- Modify: `crates/rhtmx-sync/src/websocket.rs`
- Modify: `crates/rhtmx-sync/src/field_websocket.rs`

#### 4. Update WebSocket Handlers
**Current State:** WebSocket handlers may still have SQLite-specific code.

**Files to Review:**
- `crates/rhtmx-sync/src/websocket.rs`
- `crates/rhtmx-sync/src/field_websocket.rs`

**Required Changes:**
- [ ] Verify all database operations use `DbPool`
- [ ] Add PostgreSQL LISTEN/NOTIFY support for real-time updates
- [ ] Test with both PostgreSQL and SQLite

#### 5. Update API Handlers
**Files to Review:**
- `crates/rhtmx-sync/src/sync_api.rs`
- `crates/rhtmx-sync/src/field_sync_api.rs`

**Required Changes:**
- [ ] Verify all handlers work with `DbPool`
- [ ] Test endpoint behavior with PostgreSQL
- [ ] Ensure proper error handling for both backends

### Medium Priority

#### 6. Comprehensive Testing
**Required Test Coverage:**
- [ ] Unit tests for all ChangeTracker PostgreSQL methods
- [ ] Unit tests for all FieldTracker PostgreSQL methods
- [ ] Integration tests with actual PostgreSQL database
- [ ] Integration tests with SQLite for backward compatibility
- [ ] End-to-end tests for WebSocket sync with PostgreSQL
- [ ] Performance comparison tests (PostgreSQL vs SQLite)

**Files:**
- Enhance: `crates/rhtmx-sync/src/change_tracker.rs:418-486` (existing tests)
- Add: `crates/rhtmx-sync/tests/integration_postgres.rs`
- Add: `crates/rhtmx-sync/tests/field_sync_postgres.rs`

#### 7. Documentation & Examples
**Required Documentation:**
- [ ] Update README with PostgreSQL setup instructions
- [ ] Add Supabase integration guide
- [ ] Document environment variables for PostgreSQL
- [ ] Add migration running instructions
- [ ] Create example with PostgreSQL connection
- [ ] Document LISTEN/NOTIFY usage

**Files to Create/Update:**
- `crates/rhtmx-sync/README.md`
- `docs/postgresql-setup.md`
- `docs/supabase-integration.md`
- `examples/postgres_sync/`

---

## Running Migrations

### PostgreSQL
```bash
# Set database URL
export DATABASE_URL="postgresql://user:password@localhost/rhtmx_db"

# Install diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# Run migrations
cd crates/rhtmx-sync
diesel migration run
```

### SQLite (Development)
```bash
# Set database URL
export DATABASE_URL="sqlite://./dev.db"

# Install diesel CLI
cargo install diesel_cli --no-default-features --features sqlite

# Run migrations (from migrations_sqlite directory)
cd crates/rhtmx-sync
diesel migration run --migration-dir migrations_sqlite
```

---

## Testing

### With PostgreSQL
```bash
# Start PostgreSQL (Docker)
docker run -d \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  postgres:16

# Run tests
export DATABASE_URL="postgresql://postgres:postgres@localhost/postgres"
cargo test --package rhtmx-sync
```

### With SQLite
```bash
# Run tests
cargo test --package rhtmx-sync -- test_change_tracker_sqlite
```

---

## Usage Example

### PostgreSQL (Primary)
```rust
use rhtmx_sync::{DbPool, SyncEngine, SyncConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create PostgreSQL connection pool
    let db_pool = DbPool::from_url("postgresql://localhost/mydb").await?;

    // Initialize sync engine
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

### SQLite (Development)
```rust
let db_pool = DbPool::from_url("sqlite://./dev.db").await?;
// Rest is the same...
```

---

## Performance Considerations

### PostgreSQL Advantages
- **LISTEN/NOTIFY**: Real-time notifications without polling
- **Concurrent writes**: Better handling of simultaneous clients
- **Scalability**: Production-ready for multiple servers
- **ACID guarantees**: Stronger consistency guarantees

### SQLite Use Cases
- **Development**: Quick local testing
- **Single-user apps**: Desktop applications
- **Embedded scenarios**: No server required

---

## Supabase Integration

Supabase uses PostgreSQL, making it a perfect fit for RHTMX sync:

```rust
// Supabase connection URL format
let database_url = format!(
    "postgresql://postgres:{}@db.{}.supabase.co:5432/postgres",
    env::var("SUPABASE_DB_PASSWORD")?,
    env::var("SUPABASE_PROJECT_ID")?
);

let db_pool = DbPool::from_url(&database_url).await?;
```

**Benefits:**
- Automatic backups
- Built-in real-time subscriptions
- Connection pooling (PgBouncer)
- Auto-scaling

---

## Next Steps

1. **Complete FieldTracker Migration** (Critical)
   - Create PostgreSQL migration for field sync table
   - Implement all PostgreSQL methods
   - Update tests

2. **Implement LISTEN/NOTIFY** (High Priority)
   - Create `postgres_notify.rs` module
   - Integrate with WebSocket handlers
   - Test real-time sync

3. **Comprehensive Testing** (High Priority)
   - Integration tests with PostgreSQL
   - Performance benchmarks
   - Multi-client scenarios

4. **Documentation** (Medium Priority)
   - PostgreSQL setup guide
   - Supabase integration guide
   - Migration from SQLite

---

## Progress Tracking

- **Overall Completion:** 70%
- **Core Infrastructure:** 90% ✅
- **FieldTracker Migration:** 0% 🔴
- **LISTEN/NOTIFY:** 25% (trigger created, Rust code pending) 🟡
- **Testing:** 40% (basic tests exist, comprehensive tests pending) 🟡
- **Documentation:** 20% 🔴

---

## References

- [Diesel Documentation](https://diesel.rs/)
- [diesel-async](https://docs.rs/diesel-async/)
- [PostgreSQL LISTEN/NOTIFY](https://www.postgresql.org/docs/current/sql-notify.html)
- [Supabase Database](https://supabase.com/docs/guides/database)

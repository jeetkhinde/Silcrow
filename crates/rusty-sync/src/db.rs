// File: rusty-sync/src/db.rs
// Purpose: Database pool abstraction supporting PostgreSQL (diesel-async) and SQLite (sqlx)

use anyhow::{Context, Result};
use diesel_async::pooled_connection::bb8::{self, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Database pool that supports both PostgreSQL (diesel-async) and SQLite (sqlx)
/// PostgreSQL is the PRIMARY database, SQLite is OPTIONAL for development
#[derive(Clone)]
pub enum DbPool {
    /// PostgreSQL connection pool (PRIMARY) using diesel-async
    Postgres(Pool<AsyncPgConnection>),
    /// SQLite connection pool (OPTIONAL) using sqlx for backward compatibility
    Sqlite(Arc<SqlitePool>),
}

impl DbPool {
    /// Create a new PostgreSQL connection pool (PRIMARY)
    pub async fn new_postgres(database_url: &str) -> Result<Self> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(config)
            .await
            .context("Failed to create PostgreSQL connection pool")?;

        Ok(DbPool::Postgres(pool))
    }

    /// Create a new SQLite connection pool (OPTIONAL) using sqlx
    pub async fn new_sqlite(database_url: &str) -> Result<Self> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(database_url)
            .await
            .context("Failed to create SQLite connection pool")?;

        Ok(DbPool::Sqlite(Arc::new(pool)))
    }

    /// Create a connection pool from a database URL
    /// Automatically detects PostgreSQL or SQLite based on URL scheme
    pub async fn from_url(database_url: &str) -> Result<Self> {
        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            Self::new_postgres(database_url).await
        } else if database_url.starts_with("sqlite://") || database_url.ends_with(".db") || database_url.contains(":memory:") {
            Self::new_sqlite(database_url).await
        } else {
            anyhow::bail!("Unsupported database URL format: {}", database_url)
        }
    }

    /// Get a PostgreSQL connection from the pool
    pub async fn get_postgres(&self) -> Result<bb8::PooledConnection<'_, AsyncPgConnection>> {
        match self {
            DbPool::Postgres(pool) => {
                pool.get().await
                    .context("Failed to get PostgreSQL connection from pool")
            }
            _ => anyhow::bail!("Expected PostgreSQL pool, got SQLite"),
        }
    }

    /// Get a SQLite pool reference
    pub fn get_sqlite(&self) -> Result<Arc<SqlitePool>> {
        match self {
            DbPool::Sqlite(pool) => Ok(pool.clone()),
            _ => anyhow::bail!("Expected SQLite pool, got PostgreSQL"),
        }
    }

    /// Check if this is a PostgreSQL pool
    pub fn is_postgres(&self) -> bool {
        matches!(self, DbPool::Postgres(_))
    }

    /// Check if this is a SQLite pool
    pub fn is_sqlite(&self) -> bool {
        matches!(self, DbPool::Sqlite(_))
    }
}

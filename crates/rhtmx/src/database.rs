// File: src/database.rs
// Purpose: Multi-database SQLx layer with support for SQLite, PostgreSQL, and other databases
// Supports environment-based configuration for seamless database switching
//
// Architecture: Separates pure business logic from I/O operations following functional programming principles

use sqlx::AnyPool;
use sqlx::Row;
use serde::{Deserialize, Serialize};

// ============================================================================
// DOMAIN MODELS
// ============================================================================

/// User model representing a row in the users table
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, PartialEq)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub bio: Option<String>,
    pub username: String,
}

/// User update request - represents partial updates
#[derive(Debug, Clone, Default)]
pub struct UserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub age: Option<i32>,
    pub bio: Option<String>,
}

/// Database type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    PostgreSQL,
    SQLite,
}

// ============================================================================
// PURE FUNCTIONS - No I/O, fully testable
// ============================================================================

/// Detect database type from connection URL (Pure function)
///
/// # Examples
/// ```
/// use rhtmx::database::detect_database_type;
/// use rhtmx::database::DatabaseType;
///
/// assert_eq!(detect_database_type("postgres://localhost/db"), DatabaseType::PostgreSQL);
/// assert_eq!(detect_database_type("postgresql://localhost/db"), DatabaseType::PostgreSQL);
/// assert_eq!(detect_database_type("sqlite:rhtmx.db"), DatabaseType::SQLite);
/// assert_eq!(detect_database_type("sqlite::memory:"), DatabaseType::SQLite);
/// ```
pub fn detect_database_type(database_url: &str) -> DatabaseType {
    if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        DatabaseType::PostgreSQL
    } else {
        DatabaseType::SQLite
    }
}

/// Get CREATE TABLE schema for the given database type (Pure function)
///
/// Returns the SQL schema appropriate for the database type
fn get_create_table_schema(db_type: DatabaseType) -> &'static str {
    match db_type {
        DatabaseType::PostgreSQL => r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                age INTEGER NOT NULL,
                bio TEXT,
                username TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        DatabaseType::SQLite => r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                age INTEGER NOT NULL,
                bio TEXT,
                username TEXT NOT NULL UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
    }
}

/// Apply updates to a user (Pure function - no mutation)
///
/// Takes an existing user and an update request, returns a new user with updates applied.
/// Fields that are None in the update request are preserved from the original user.
///
/// # Examples
/// ```
/// use rhtmx::database::{User, UserUpdate, apply_user_updates};
///
/// let user = User {
///     id: 1,
///     name: "John".to_string(),
///     email: "john@example.com".to_string(),
///     age: 30,
///     bio: Some("Developer".to_string()),
///     username: "john".to_string(),
/// };
///
/// let updates = UserUpdate {
///     name: Some("Johnny".to_string()),
///     age: Some(31),
///     ..Default::default()
/// };
///
/// let updated = apply_user_updates(user.clone(), updates);
/// assert_eq!(updated.name, "Johnny");
/// assert_eq!(updated.age, 31);
/// assert_eq!(updated.email, user.email); // Unchanged
/// ```
pub fn apply_user_updates(user: User, updates: UserUpdate) -> User {
    User {
        name: updates.name.unwrap_or(user.name),
        email: updates.email.unwrap_or(user.email),
        age: updates.age.unwrap_or(user.age),
        bio: updates.bio.or(user.bio),
        ..user
    }
}

/// Build SQL LIKE pattern for search (Pure function)
///
/// Returns a tuple of (has_filter, pattern) where:
/// - has_filter: true if a filter was provided
/// - pattern: the SQL LIKE pattern with wildcards
///
/// # Examples
/// ```
/// use rhtmx::database::build_search_pattern;
///
/// let (has_filter, pattern) = build_search_pattern(Some("John"));
/// assert!(has_filter);
/// assert_eq!(pattern, "%John%");
///
/// let (has_filter, pattern) = build_search_pattern(None);
/// assert!(!has_filter);
/// assert_eq!(pattern, "");
/// ```
pub fn build_search_pattern(filter: Option<&str>) -> (bool, String) {
    match filter {
        Some(f) if !f.is_empty() => (true, format!("%{}%", f)),
        _ => (false, String::new()),
    }
}

/// Construct a User from individual fields (Pure function)
///
/// Helper to construct a User in a functional way
fn construct_user(
    id: i32,
    name: String,
    email: String,
    age: i32,
    username: String,
    bio: Option<String>,
) -> User {
    User {
        id,
        name,
        email,
        age,
        bio,
        username,
    }
}

// ============================================================================
// I/O OPERATIONS - Async functions with side effects
// ============================================================================

/// Initialize database with support for SQLite, PostgreSQL, MySQL, etc.
///
/// # Database URLs Format:
/// - SQLite: `sqlite:rhtmx.db` or `sqlite::memory:`
/// - PostgreSQL: `postgres://user:password@localhost:5432/dbname`
/// - PostgreSQL (Supabase): `postgres://postgres:password@db.xxxxx.supabase.co:5432/postgres`
pub async fn init_db(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    // Create connection pool with any supported database
    let pool = AnyPool::connect(database_url).await?;

    // Detect database type (pure function!)
    let db_type = detect_database_type(database_url);

    // Get appropriate schema (pure function!)
    let schema = get_create_table_schema(db_type);

    // Run migrations (I/O operation)
    sqlx::query(schema).execute(&pool).await?;

    Ok(pool)
}

/// Get all users from the database
pub async fn get_users(pool: &AnyPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, name, email, age, bio, username FROM users ORDER BY id")
        .fetch_all(pool)
        .await
}

/// Get a user by ID
pub async fn get_user(pool: &AnyPool, id: i32) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, age, bio, username FROM users WHERE id = ? LIMIT 1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Create a new user
pub async fn create_user(
    pool: &AnyPool,
    name: String,
    email: String,
    age: i32,
    username: String,
    bio: Option<String>,
) -> Result<User, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO users (name, email, age, username, bio) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&name)
    .bind(&email)
    .bind(age)
    .bind(&username)
    .bind(&bio)
    .execute(pool)
    .await?;

    // Get the last inserted ID (works across SQLite, PostgreSQL, MySQL, etc.)
    let last_id: i32 = result.last_insert_id().unwrap_or(0) as i32;

    // Construct user using pure function
    Ok(construct_user(last_id, name, email, age, username, bio))
}

/// Update an existing user
///
/// This function separates I/O (fetching, saving) from pure logic (applying updates)
pub async fn update_user(
    pool: &AnyPool,
    id: i32,
    name: Option<String>,
    email: Option<String>,
    age: Option<i32>,
    bio: Option<String>,
) -> Result<Option<User>, sqlx::Error> {
    // I/O: Fetch current user
    let current = get_user(pool, id).await?;

    match current {
        None => Ok(None),
        Some(user) => {
            // Pure: Apply updates functionally
            let updates = UserUpdate { name, email, age, bio };
            let updated_user = apply_user_updates(user, updates);

            // I/O: Save to database
            sqlx::query(
                "UPDATE users SET name = ?, email = ?, age = ?, bio = ? WHERE id = ?"
            )
            .bind(&updated_user.name)
            .bind(&updated_user.email)
            .bind(updated_user.age)
            .bind(&updated_user.bio)
            .bind(id)
            .execute(pool)
            .await?;

            Ok(Some(updated_user))
        }
    }
}

/// Delete a user by ID
pub async fn delete_user(pool: &AnyPool, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Count total users
pub async fn count_users(pool: &AnyPool) -> Result<i32, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?;

    Ok(row.get::<i32, _>("count"))
}

/// Search users by filter (name or email)
///
/// This function separates pure logic (building search pattern) from I/O (querying)
pub async fn search_users(
    pool: &AnyPool,
    filter: Option<String>,
) -> Result<Vec<User>, sqlx::Error> {
    // Pure: Build search pattern
    let (has_filter, pattern) = build_search_pattern(filter.as_deref());

    // I/O: Execute appropriate query based on pattern
    if has_filter {
        sqlx::query_as::<_, User>(
            "SELECT id, name, email, age, bio, username FROM users WHERE name LIKE ? OR email LIKE ? ORDER BY id"
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(pool)
        .await
    } else {
        get_users(pool).await
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Tests for PURE functions (no I/O, fast, reliable)
    // ========================================================================

    #[test]
    fn test_detect_database_type_postgres() {
        assert_eq!(
            detect_database_type("postgres://localhost/db"),
            DatabaseType::PostgreSQL
        );
        assert_eq!(
            detect_database_type("postgresql://user:pass@host:5432/db"),
            DatabaseType::PostgreSQL
        );
    }

    #[test]
    fn test_detect_database_type_sqlite() {
        assert_eq!(
            detect_database_type("sqlite:rhtmx.db"),
            DatabaseType::SQLite
        );
        assert_eq!(
            detect_database_type("sqlite::memory:"),
            DatabaseType::SQLite
        );
    }

    #[test]
    fn test_get_create_table_schema_postgres() {
        let schema = get_create_table_schema(DatabaseType::PostgreSQL);
        assert!(schema.contains("SERIAL PRIMARY KEY"));
        assert!(schema.contains("TIMESTAMP"));
    }

    #[test]
    fn test_get_create_table_schema_sqlite() {
        let schema = get_create_table_schema(DatabaseType::SQLite);
        assert!(schema.contains("AUTOINCREMENT"));
        assert!(schema.contains("DATETIME"));
    }

    #[test]
    fn test_apply_user_updates_full() {
        let user = User {
            id: 1,
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            bio: Some("Developer".to_string()),
            username: "john".to_string(),
        };

        let updates = UserUpdate {
            name: Some("Johnny".to_string()),
            email: Some("johnny@example.com".to_string()),
            age: Some(31),
            bio: Some("Senior Developer".to_string()),
        };

        let updated = apply_user_updates(user, updates);

        assert_eq!(updated.name, "Johnny");
        assert_eq!(updated.email, "johnny@example.com");
        assert_eq!(updated.age, 31);
        assert_eq!(updated.bio, Some("Senior Developer".to_string()));
        assert_eq!(updated.id, 1); // ID should not change
        assert_eq!(updated.username, "john"); // Username should not change
    }

    #[test]
    fn test_apply_user_updates_partial() {
        let user = User {
            id: 1,
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            bio: Some("Developer".to_string()),
            username: "john".to_string(),
        };

        let updates = UserUpdate {
            name: Some("Johnny".to_string()),
            age: Some(31),
            ..Default::default()
        };

        let updated = apply_user_updates(user.clone(), updates);

        assert_eq!(updated.name, "Johnny");
        assert_eq!(updated.age, 31);
        assert_eq!(updated.email, user.email); // Unchanged
        assert_eq!(updated.bio, user.bio); // Unchanged
    }

    #[test]
    fn test_apply_user_updates_no_changes() {
        let user = User {
            id: 1,
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            bio: Some("Developer".to_string()),
            username: "john".to_string(),
        };

        let updates = UserUpdate::default();
        let updated = apply_user_updates(user.clone(), updates);

        assert_eq!(updated, user); // Should be unchanged
    }

    #[test]
    fn test_apply_user_updates_bio_override() {
        let user = User {
            id: 1,
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            bio: Some("Developer".to_string()),
            username: "john".to_string(),
        };

        let updates = UserUpdate {
            bio: Some("New Bio".to_string()),
            ..Default::default()
        };

        let updated = apply_user_updates(user, updates);
        assert_eq!(updated.bio, Some("New Bio".to_string()));
    }

    #[test]
    fn test_build_search_pattern_with_filter() {
        let (has_filter, pattern) = build_search_pattern(Some("John"));
        assert!(has_filter);
        assert_eq!(pattern, "%John%");

        let (has_filter, pattern) = build_search_pattern(Some("test@example.com"));
        assert!(has_filter);
        assert_eq!(pattern, "%test@example.com%");
    }

    #[test]
    fn test_build_search_pattern_without_filter() {
        let (has_filter, pattern) = build_search_pattern(None);
        assert!(!has_filter);
        assert_eq!(pattern, "");
    }

    #[test]
    fn test_build_search_pattern_empty_string() {
        let (has_filter, pattern) = build_search_pattern(Some(""));
        assert!(!has_filter);
        assert_eq!(pattern, "");
    }

    #[test]
    fn test_construct_user() {
        let user = construct_user(
            1,
            "John".to_string(),
            "john@example.com".to_string(),
            30,
            "john".to_string(),
            Some("Developer".to_string()),
        );

        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.age, 30);
        assert_eq!(user.username, "john");
        assert_eq!(user.bio, Some("Developer".to_string()));
    }

    #[test]
    fn test_construct_user_no_bio() {
        let user = construct_user(
            2,
            "Jane".to_string(),
            "jane@example.com".to_string(),
            25,
            "jane".to_string(),
            None,
        );

        assert_eq!(user.id, 2);
        assert_eq!(user.bio, None);
    }

    // ========================================================================
    // Tests for I/O functions (async, with database)
    // ========================================================================

    #[tokio::test]
    async fn test_init_db() {
        let pool = init_db("sqlite::memory:").await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let pool = init_db("sqlite::memory:").await.unwrap();

        let user = create_user(
            &pool,
            "John".to_string(),
            "john@example.com".to_string(),
            30,
            "john".to_string(),
            Some("Developer".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(user.name, "John");
        assert_eq!(user.email, "john@example.com");

        let fetched = get_user(&pool, user.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "John");
    }

    #[tokio::test]
    async fn test_count_users() {
        let pool = init_db("sqlite::memory:").await.unwrap();

        create_user(
            &pool,
            "Alice".to_string(),
            "alice@example.com".to_string(),
            25,
            "alice".to_string(),
            None,
        )
        .await
        .unwrap();

        create_user(
            &pool,
            "Bob".to_string(),
            "bob@example.com".to_string(),
            35,
            "bob".to_string(),
            None,
        )
        .await
        .unwrap();

        let count = count_users(&pool).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_search_users() {
        let pool = init_db("sqlite::memory:").await.unwrap();

        create_user(
            &pool,
            "Alice".to_string(),
            "alice@example.com".to_string(),
            25,
            "alice".to_string(),
            None,
        )
        .await
        .unwrap();

        create_user(
            &pool,
            "Bob".to_string(),
            "bob@example.com".to_string(),
            35,
            "bob".to_string(),
            None,
        )
        .await
        .unwrap();

        let results = search_users(&pool, Some("Alice".to_string()))
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alice");
    }

    #[tokio::test]
    async fn test_update_user() {
        let pool = init_db("sqlite::memory:").await.unwrap();

        let user = create_user(
            &pool,
            "Charlie".to_string(),
            "charlie@example.com".to_string(),
            28,
            "charlie".to_string(),
            None,
        )
        .await
        .unwrap();

        let updated = update_user(
            &pool,
            user.id,
            Some("Charles".to_string()),
            None,
            Some(29),
            None,
        )
        .await
        .unwrap();

        assert!(updated.is_some());
        let u = updated.unwrap();
        assert_eq!(u.name, "Charles");
        assert_eq!(u.age, 29);
        assert_eq!(u.email, "charlie@example.com"); // Unchanged
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = init_db("sqlite::memory:").await.unwrap();

        let user = create_user(
            &pool,
            "Dave".to_string(),
            "dave@example.com".to_string(),
            40,
            "dave".to_string(),
            None,
        )
        .await
        .unwrap();

        let deleted = delete_user(&pool, user.id).await.unwrap();
        assert!(deleted);

        let fetched = get_user(&pool, user.id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_search_users_no_filter() {
        let pool = init_db("sqlite::memory:").await.unwrap();

        create_user(
            &pool,
            "Alice".to_string(),
            "alice@example.com".to_string(),
            25,
            "alice".to_string(),
            None,
        )
        .await
        .unwrap();

        create_user(
            &pool,
            "Bob".to_string(),
            "bob@example.com".to_string(),
            35,
            "bob".to_string(),
            None,
        )
        .await
        .unwrap();

        // No filter should return all users
        let results = search_users(&pool, None).await.unwrap();
        assert_eq!(results.len(), 2);
    }
}

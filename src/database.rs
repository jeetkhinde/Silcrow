// File: src/database.rs
// Purpose: SQLx database layer with connection pooling and schema

use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

/// User model representing a row in the users table
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub bio: Option<String>,
    pub username: String,
}

/// Initialize SQLite database with schema
pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create connection options
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    // Create pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Run migrations (create tables if they don't exist)
    sqlx::query(
        r#"
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
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

/// Get all users from the database
pub async fn get_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, name, email, age, bio, username FROM users ORDER BY id")
        .fetch_all(pool)
        .await
}

/// Get a user by ID
pub async fn get_user(pool: &SqlitePool, id: i32) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, name, email, age, bio, username FROM users WHERE id = ? LIMIT 1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Create a new user
pub async fn create_user(
    pool: &SqlitePool,
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

    let id = result.last_insert_rowid() as i32;

    Ok(User {
        id,
        name,
        email,
        age,
        username,
        bio,
    })
}

/// Update an existing user
pub async fn update_user(
    pool: &SqlitePool,
    id: i32,
    name: Option<String>,
    email: Option<String>,
    age: Option<i32>,
    bio: Option<String>,
) -> Result<Option<User>, sqlx::Error> {
    // Get current user
    let current = get_user(pool, id).await?;
    let mut user = match current {
        Some(u) => u,
        None => return Ok(None),
    };

    // Update fields
    if let Some(new_name) = name {
        user.name = new_name;
    }
    if let Some(new_email) = email {
        user.email = new_email;
    }
    if let Some(new_age) = age {
        user.age = new_age;
    }
    if let Some(new_bio) = bio {
        user.bio = Some(new_bio);
    }

    // Execute update
    sqlx::query(
        "UPDATE users SET name = ?, email = ?, age = ?, bio = ? WHERE id = ?"
    )
    .bind(&user.name)
    .bind(&user.email)
    .bind(user.age)
    .bind(&user.bio)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(Some(user))
}

/// Delete a user by ID
pub async fn delete_user(pool: &SqlitePool, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Count total users
pub async fn count_users(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?;

    Ok(row.get::<i32, _>("count"))
}

/// Search users by filter (name or email)
pub async fn search_users(
    pool: &SqlitePool,
    filter: Option<String>,
) -> Result<Vec<User>, sqlx::Error> {
    if let Some(f) = filter {
        let pattern = format!("%{}%", f);
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

#[cfg(test)]
mod tests {
    use super::*;

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
}

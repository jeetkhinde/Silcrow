// Integration tests for PostgreSQL support
// Run with: DATABASE_URL=postgresql://user:pass@localhost/test cargo test --test postgres_integration

use rusty_sync::{
    ChangeAction, ChangeTracker, DbPool, FieldAction, FieldMergeStrategy, FieldTracker,
};
use std::sync::Arc;

/// Helper to check if PostgreSQL is available
fn postgres_available() -> Option<String> {
    std::env::var("DATABASE_URL")
        .ok()
        .filter(|url| url.starts_with("postgres"))
}

#[tokio::test]
async fn test_postgres_change_tracker() {
    let Some(db_url) = postgres_available() else {
        eprintln!("Skipping PostgreSQL test: DATABASE_URL not set");
        return;
    };

    let db_pool = DbPool::from_url(&db_url).await.unwrap();
    let tracker = Arc::new(ChangeTracker::new(Arc::new(db_pool)).await.unwrap());

    // Record a change
    let change = tracker
        .record_change(
            "test_users",
            "1",
            ChangeAction::Create,
            Some(serde_json::json!({"name": "Alice"})),
            Some("client-1".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(change.entity, "test_users");
    assert_eq!(change.entity_id, "1");
    assert_eq!(change.action, ChangeAction::Create);
    assert!(change.version >= 1);

    // Get latest version
    let version = tracker.latest_version("test_users").await.unwrap();
    assert!(version >= 1);

    // Get changes since version 0
    let changes = tracker
        .get_changes_since("test_users", 0)
        .await
        .unwrap();
    assert!(!changes.is_empty());

    // Cleanup
    let deleted = tracker.cleanup_old_entries(365).await.unwrap();
    println!("Cleaned up {} old entries", deleted);
}

#[tokio::test]
async fn test_postgres_field_tracker() {
    let Some(db_url) = postgres_available() else {
        eprintln!("Skipping PostgreSQL test: DATABASE_URL not set");
        return;
    };

    let db_pool = DbPool::from_url(&db_url).await.unwrap();
    let tracker = Arc::new(
        FieldTracker::new(Arc::new(db_pool), FieldMergeStrategy::LastWriteWins)
            .await
            .unwrap(),
    );

    // Record field changes
    let change1 = tracker
        .record_field_change(
            "test_products",
            "p1",
            "name",
            Some(serde_json::json!("Widget")),
            FieldAction::Update,
            None,
        )
        .await
        .unwrap();

    assert_eq!(change1.entity, "test_products");
    assert_eq!(change1.entity_id, "p1");
    assert_eq!(change1.field, "name");

    let change2 = tracker
        .record_field_change(
            "test_products",
            "p1",
            "price",
            Some(serde_json::json!(19.99)),
            FieldAction::Update,
            None,
        )
        .await
        .unwrap();

    assert_eq!(change2.field, "price");

    // Get latest fields
    let fields = tracker
        .get_latest_fields("test_products", "p1")
        .await
        .unwrap();

    assert_eq!(fields.len(), 2);
    assert_eq!(fields.get("name").unwrap(), &serde_json::json!("Widget"));
    assert_eq!(fields.get("price").unwrap(), &serde_json::json!(19.99));

    // Get changes since version 0
    let changes = tracker
        .get_field_changes_since("test_products", 0)
        .await
        .unwrap();

    assert_eq!(changes.len(), 2);
}

#[tokio::test]
async fn test_postgres_concurrent_writes() {
    let Some(db_url) = postgres_available() else {
        eprintln!("Skipping PostgreSQL test: DATABASE_URL not set");
        return;
    };

    let db_pool = DbPool::from_url(&db_url).await.unwrap();
    let tracker = Arc::new(ChangeTracker::new(Arc::new(db_pool)).await.unwrap());

    // Spawn multiple concurrent writes
    let mut handles = vec![];

    for i in 0..10 {
        let tracker = tracker.clone();
        let handle = tokio::spawn(async move {
            tracker
                .record_change(
                    "concurrent_test",
                    &format!("id-{}", i),
                    ChangeAction::Create,
                    Some(serde_json::json!({"value": i})),
                    None,
                )
                .await
        });
        handles.push(handle);
    }

    // Wait for all writes to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Verify all changes were recorded
    let changes = tracker
        .get_changes_since("concurrent_test", 0)
        .await
        .unwrap();

    assert_eq!(changes.len(), 10);
}

#[tokio::test]
async fn test_postgres_field_merge_strategies() {
    let Some(db_url) = postgres_available() else {
        eprintln!("Skipping PostgreSQL test: DATABASE_URL not set");
        return;
    };

    let db_pool = DbPool::from_url(&db_url).await.unwrap();

    // Test LastWriteWins strategy
    let tracker = Arc::new(
        FieldTracker::new(Arc::new(db_pool.clone()), FieldMergeStrategy::LastWriteWins)
            .await
            .unwrap(),
    );

    // Record initial server value
    tracker
        .record_field_change(
            "merge_test",
            "m1",
            "status",
            Some(serde_json::json!("pending")),
            FieldAction::Update,
            None,
        )
        .await
        .unwrap();

    // Simulate client change with future timestamp
    let client_timestamp = chrono::Utc::now() + chrono::Duration::seconds(10);
    let client_changes = vec![(
        "status".to_string(),
        serde_json::json!("completed"),
        client_timestamp,
    )];

    let (applied, conflicts) = tracker
        .merge_field_changes("merge_test", "m1", client_changes)
        .await
        .unwrap();

    // With LastWriteWins, newer client change should be applied
    assert_eq!(applied.len(), 1);
    assert_eq!(conflicts.len(), 0);

    // Verify the merged value
    let fields = tracker
        .get_latest_fields("merge_test", "m1")
        .await
        .unwrap();

    assert_eq!(
        fields.get("status").unwrap(),
        &serde_json::json!("completed")
    );
}

#[tokio::test]
async fn test_postgres_version_tracking() {
    let Some(db_url) = postgres_available() else {
        eprintln!("Skipping PostgreSQL test: DATABASE_URL not set");
        return;
    };

    let db_pool = DbPool::from_url(&db_url).await.unwrap();
    let tracker = Arc::new(ChangeTracker::new(Arc::new(db_pool)).await.unwrap());

    let entity = "version_test";

    // Initial version should be 0
    let v0 = tracker.latest_version(entity).await.unwrap();

    // Record multiple changes
    for i in 1..=5 {
        tracker
            .record_change(
                entity,
                &format!("id-{}", i),
                ChangeAction::Create,
                Some(serde_json::json!({"count": i})),
                None,
            )
            .await
            .unwrap();
    }

    // Version should be incremented
    let v5 = tracker.latest_version(entity).await.unwrap();
    assert_eq!(v5, v0 + 5);

    // Get changes since v2
    let changes = tracker.get_changes_since(entity, v0 + 2).await.unwrap();
    assert_eq!(changes.len(), 3);

    // Verify versions are sequential
    assert_eq!(changes[0].version, v0 + 3);
    assert_eq!(changes[1].version, v0 + 4);
    assert_eq!(changes[2].version, v0 + 5);
}

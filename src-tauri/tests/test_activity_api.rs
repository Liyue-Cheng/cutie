use app_lib::commands::activity_commands::*;
use app_lib::core::db::DbPool;
use chrono::{Duration, Utc};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

async fn setup_test_db() -> DbPool {
    let db_url = "sqlite::memory:";
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        Sqlite::create_database(db_url).await.unwrap();
    }
    let pool = SqlitePool::connect(db_url)
        .await
        .expect("Failed to connect to in-memory db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migrations failed");
    pool
}

#[tokio::test]
async fn test_create_activity_core() {
    // Arrange
    let pool = setup_test_db().await;
    let payload = CreateActivityPayload {
        title: Some("Test Activity".to_string()),
        start_time: Utc::now(),
        end_time: Utc::now(),
        ..Default::default()
    };

    // Act
    let result = create_activity_core(&pool, payload).await;

    // Assert
    assert!(result.is_ok());
    let activity = result.unwrap();
    assert_eq!(activity.title, Some("Test Activity".to_string()));
}

#[tokio::test]
async fn test_create_activity_fails_if_overlapping() {
    // Arrange
    let pool = setup_test_db().await;
    let now = Utc::now();
    let existing_payload = CreateActivityPayload {
        title: Some("Existing Activity".to_string()),
        start_time: now,
        end_time: now + Duration::hours(2),
        ..Default::default()
    };
    create_activity_core(&pool, existing_payload).await.unwrap();

    // Overlaps at the start
    let overlapping_payload = CreateActivityPayload {
        title: Some("Overlapping Activity".to_string()),
        start_time: now + Duration::hours(1),
        end_time: now + Duration::hours(3),
        ..Default::default()
    };

    // Act
    let result = create_activity_core(&pool, overlapping_payload).await;

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("An overlapping activity already exists."));
}

#[tokio::test]
async fn test_get_activity_core() {
    // Arrange
    let pool = setup_test_db().await;
    let created = create_activity_core(
        &pool,
        CreateActivityPayload {
            title: Some("Get Me".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act
    let result = get_activity_core(&pool, created.id).await;

    // Assert
    assert!(result.is_ok());
    let fetched = result.unwrap();
    assert_eq!(fetched.id, created.id);
}

#[tokio::test]
async fn test_list_activities_core() {
    // Arrange
    let pool = setup_test_db().await;
    create_activity_core(
        &pool,
        CreateActivityPayload {
            title: Some("A1".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    create_activity_core(
        &pool,
        CreateActivityPayload {
            title: Some("A2".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act
    let result = list_activities_core(&pool).await;

    // Assert
    assert!(result.is_ok());
    let activities = result.unwrap();
    assert_eq!(activities.len(), 2);
}

#[tokio::test]
async fn test_update_activity_core() {
    // Arrange
    let pool = setup_test_db().await;
    let activity = create_activity_core(
        &pool,
        CreateActivityPayload {
            title: Some("Original".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act
    let payload = UpdateActivityPayload {
        title: Some("Updated".to_string()),
        ..Default::default()
    };
    let result = update_activity_core(&pool, activity.id, payload).await;

    // Assert
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.title, Some("Updated".to_string()));
}

#[tokio::test]
async fn test_update_activity_succeeds_with_same_times() {
    // Arrange
    let pool = setup_test_db().await;
    let now = Utc::now();
    let activity = create_activity_core(
        &pool,
        CreateActivityPayload {
            start_time: now,
            end_time: now + Duration::hours(2),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let payload = UpdateActivityPayload {
        title: Some("Updated Title".to_string()),
        start_time: Some(activity.start_time),
        end_time: Some(activity.end_time),
        ..Default::default()
    };

    // Act
    let result = update_activity_core(&pool, activity.id, payload).await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_activity_fails_if_overlapping() {
    // Arrange
    let pool = setup_test_db().await;
    let now = Utc::now();
    // Create first activity
    create_activity_core(
        &pool,
        CreateActivityPayload {
            start_time: now,
            end_time: now + Duration::hours(2),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    // Create the activity we are going to update
    let activity_to_update = create_activity_core(
        &pool,
        CreateActivityPayload {
            start_time: now + Duration::hours(3),
            end_time: now + Duration::hours(4),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act: Try to update the second activity to overlap with the first
    let payload = UpdateActivityPayload {
        start_time: Some(now + Duration::hours(1)),
        end_time: Some(now + Duration::hours(3)),
        ..Default::default()
    };
    let result = update_activity_core(&pool, activity_to_update.id, payload).await;

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("An overlapping activity already exists."));
}

#[tokio::test]
async fn test_delete_activity_core() {
    // Arrange
    let pool = setup_test_db().await;
    let activity = create_activity_core(
        &pool,
        CreateActivityPayload {
            title: Some("To Delete".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act
    let delete_result = delete_activity_core(&pool, activity.id).await;
    let get_result = get_activity_core(&pool, activity.id).await;

    // Assert
    assert!(delete_result.is_ok());
    assert!(get_result.is_err());
}

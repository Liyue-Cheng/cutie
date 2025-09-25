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

// --- Tests for All-Day Event Overlap Logic ---

#[tokio::test]
async fn test_all_day_events_can_overlap_with_non_all_day_events() {
    // Arrange
    let pool = setup_test_db().await;
    let now = Utc::now();

    // Create a non-all-day event
    let non_all_day_payload = CreateActivityPayload {
        title: Some("Non All-Day Meeting".to_string()),
        start_time: now,
        end_time: now + Duration::hours(2),
        is_all_day: Some(false),
        ..Default::default()
    };
    create_activity_core(&pool, non_all_day_payload)
        .await
        .unwrap();

    // Create an all-day event on the same day
    let all_day_payload = CreateActivityPayload {
        title: Some("All-Day Event".to_string()),
        start_time: now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
        end_time: now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc(),
        is_all_day: Some(true),
        ..Default::default()
    };

    // Act
    let result = create_activity_core(&pool, all_day_payload).await;

    // Assert
    assert!(
        result.is_ok(),
        "All-day events should be able to coexist with non-all-day events on the same day"
    );
}

#[tokio::test]
async fn test_all_day_events_conflict_with_other_all_day_events_same_day() {
    // Arrange
    let pool = setup_test_db().await;
    let now = Utc::now();

    // Create first all-day event
    let first_all_day_payload = CreateActivityPayload {
        title: Some("First All-Day Event".to_string()),
        start_time: now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
        end_time: now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc(),
        is_all_day: Some(true),
        ..Default::default()
    };
    create_activity_core(&pool, first_all_day_payload)
        .await
        .unwrap();

    // Try to create second all-day event on the same day
    let second_all_day_payload = CreateActivityPayload {
        title: Some("Second All-Day Event".to_string()),
        start_time: now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
        end_time: now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc(),
        is_all_day: Some(true),
        ..Default::default()
    };

    // Act
    let result = create_activity_core(&pool, second_all_day_payload).await;

    // Assert
    assert!(
        result.is_err(),
        "All-day events should conflict with other all-day events on the same day"
    );
    let err = result.unwrap_err().to_string();
    assert!(err.contains("An overlapping activity already exists."));
}

#[tokio::test]
async fn test_all_day_events_can_coexist_on_different_days() {
    // Arrange
    let pool = setup_test_db().await;
    let today = Utc::now();
    let tomorrow = today + Duration::days(1);

    // Create first all-day event for today
    let today_payload = CreateActivityPayload {
        title: Some("Today All-Day Event".to_string()),
        start_time: today.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
        end_time: today
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc(),
        is_all_day: Some(true),
        ..Default::default()
    };
    create_activity_core(&pool, today_payload).await.unwrap();

    // Create second all-day event for tomorrow
    let tomorrow_payload = CreateActivityPayload {
        title: Some("Tomorrow All-Day Event".to_string()),
        start_time: tomorrow
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
        end_time: tomorrow
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc(),
        is_all_day: Some(true),
        ..Default::default()
    };

    // Act
    let result = create_activity_core(&pool, tomorrow_payload).await;

    // Assert
    assert!(
        result.is_ok(),
        "All-day events on different days should not conflict"
    );
}

#[tokio::test]
async fn test_non_all_day_events_still_conflict_with_time_overlap() {
    // Arrange
    let pool = setup_test_db().await;
    let now = Utc::now();

    // Create first non-all-day event
    let first_event_payload = CreateActivityPayload {
        title: Some("First Meeting".to_string()),
        start_time: now,
        end_time: now + Duration::hours(2),
        is_all_day: Some(false),
        ..Default::default()
    };
    create_activity_core(&pool, first_event_payload)
        .await
        .unwrap();

    // Try to create overlapping non-all-day event
    let overlapping_event_payload = CreateActivityPayload {
        title: Some("Overlapping Meeting".to_string()),
        start_time: now + Duration::hours(1),
        end_time: now + Duration::hours(3),
        is_all_day: Some(false),
        ..Default::default()
    };

    // Act
    let result = create_activity_core(&pool, overlapping_event_payload).await;

    // Assert
    assert!(
        result.is_err(),
        "Non-all-day events should still conflict when they have time overlap"
    );
    let err = result.unwrap_err().to_string();
    assert!(err.contains("An overlapping activity already exists."));
}

#[tokio::test]
async fn test_update_activity_all_day_status_respects_new_overlap_rules() {
    // Arrange
    let pool = setup_test_db().await;
    let now = Utc::now();

    // Create a non-all-day event
    let non_all_day_event = create_activity_core(
        &pool,
        CreateActivityPayload {
            title: Some("Meeting".to_string()),
            start_time: now,
            end_time: now + Duration::hours(2),
            is_all_day: Some(false),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Create an all-day event on the same day
    let all_day_event = create_activity_core(
        &pool,
        CreateActivityPayload {
            title: Some("All-Day Event".to_string()),
            start_time: now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
            end_time: now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc(),
            is_all_day: Some(true),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act: Try to change the non-all-day event to all-day (should fail due to conflict)
    let update_payload = UpdateActivityPayload {
        is_all_day: Some(true),
        ..Default::default()
    };
    let result = update_activity_core(&pool, non_all_day_event.id, update_payload).await;

    // Assert
    assert!(
        result.is_err(),
        "Converting to all-day should fail if there's already an all-day event on the same day"
    );
    let err = result.unwrap_err().to_string();
    assert!(err.contains("An overlapping activity already exists."));
}

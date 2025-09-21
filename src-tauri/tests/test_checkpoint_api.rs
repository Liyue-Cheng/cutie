use app_lib::commands::checkpoint_commands::*;
use app_lib::commands::task_commands::{create_task_core, CreateTaskPayload};
use app_lib::core::db::DbPool;
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
async fn test_create_checkpoint_core() {
    // Arrange
    let pool = setup_test_db().await;
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            title: "Task for Checkpoint".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let payload = CreateCheckpointPayload {
        task_id: task.id,
        title: "Test Checkpoint".to_string(),
        sort_key: "a1".to_string(),
    };

    // Act
    let result = create_checkpoint_core(&pool, payload).await;

    // Assert
    assert!(result.is_ok());
    let checkpoint = result.unwrap();
    assert_eq!(checkpoint.title, "Test Checkpoint");
    assert_eq!(checkpoint.task_id, task.id);
}

#[tokio::test]
async fn test_get_checkpoint_core() {
    // Arrange
    let pool = setup_test_db().await;
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            title: "Task".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let created_checkpoint = create_checkpoint_core(
        &pool,
        CreateCheckpointPayload {
            task_id: task.id,
            title: "Get Me".to_string(),
            sort_key: "a1".to_string(),
        },
    )
    .await
    .unwrap();

    // Act
    let result = get_checkpoint_core(&pool, created_checkpoint.id).await;

    // Assert
    assert!(result.is_ok());
    let fetched = result.unwrap();
    assert_eq!(fetched.id, created_checkpoint.id);
}

#[tokio::test]
async fn test_list_checkpoints_for_task_core() {
    // Arrange
    let pool = setup_test_db().await;
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            title: "Task".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    create_checkpoint_core(
        &pool,
        CreateCheckpointPayload {
            task_id: task.id,
            title: "CP1".to_string(),
            sort_key: "a1".to_string(),
        },
    )
    .await
    .unwrap();
    create_checkpoint_core(
        &pool,
        CreateCheckpointPayload {
            task_id: task.id,
            title: "CP2".to_string(),
            sort_key: "a2".to_string(),
        },
    )
    .await
    .unwrap();

    // Act
    let result = list_checkpoints_for_task_core(&pool, task.id).await;

    // Assert
    assert!(result.is_ok());
    let checkpoints = result.unwrap();
    assert_eq!(checkpoints.len(), 2);
}

#[tokio::test]
async fn test_update_checkpoint_core() {
    // Arrange
    let pool = setup_test_db().await;
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            title: "Task".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let checkpoint = create_checkpoint_core(
        &pool,
        CreateCheckpointPayload {
            task_id: task.id,
            title: "Original".to_string(),
            sort_key: "a1".to_string(),
        },
    )
    .await
    .unwrap();

    // Act
    let payload = UpdateCheckpointPayload {
        title: Some("Updated".to_string()),
        is_completed: Some(true),
        sort_key: Some("z9".to_string()),
    };
    let result = update_checkpoint_core(&pool, checkpoint.id, payload).await;

    // Assert
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.is_completed, true);
    assert_eq!(updated.sort_key, "z9");
}

#[tokio::test]
async fn test_delete_checkpoint_core() {
    // Arrange
    let pool = setup_test_db().await;
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            title: "Task".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let checkpoint = create_checkpoint_core(
        &pool,
        CreateCheckpointPayload {
            task_id: task.id,
            title: "To Delete".to_string(),
            sort_key: "a1".to_string(),
        },
    )
    .await
    .unwrap();

    // Act
    let delete_result = delete_checkpoint_core(&pool, checkpoint.id).await;
    let get_result = get_checkpoint_core(&pool, checkpoint.id).await;

    // Assert
    assert!(delete_result.is_ok());
    assert!(get_result.is_err());
}

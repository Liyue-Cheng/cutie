use app_lib::commands::project_commands::{create_project_core, CreateProjectPayload};
use app_lib::commands::task_commands::*;
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
async fn test_create_task_core() {
    // Arrange
    let pool = setup_test_db().await;
    let payload = CreateTaskPayload {
        project_id: None,
        title: "Test Task".to_string(),
        status: Some("in_progress".to_string()),
        due_date: None,
        sort_key: "a1".to_string(),
        metadata: None,
    };

    // Act
    let result = create_task_core(&pool, payload).await;

    // Assert
    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.title, "Test Task");
    assert_eq!(task.status, "in_progress");
}

#[tokio::test]
async fn test_get_task_core() {
    // Arrange
    let pool = setup_test_db().await;
    let created_task = create_task_core(
        &pool,
        CreateTaskPayload {
            project_id: None,
            title: "Get Me".to_string(),
            status: None,
            due_date: None,
            sort_key: "a1".to_string(),
            metadata: None,
        },
    )
    .await
    .unwrap();

    // Act
    let result = get_task_core(&pool, created_task.id).await;

    // Assert
    assert!(result.is_ok());
    let fetched_task = result.unwrap();
    assert_eq!(fetched_task.id, created_task.id);
}

#[tokio::test]
async fn test_list_tasks_core() {
    // Arrange
    let pool = setup_test_db().await;
    create_task_core(
        &pool,
        CreateTaskPayload {
            project_id: None,
            title: "Task 1".to_string(),
            status: None,
            due_date: None,
            sort_key: "a1".to_string(),
            metadata: None,
        },
    )
    .await
    .unwrap();
    create_task_core(
        &pool,
        CreateTaskPayload {
            project_id: None,
            title: "Task 2".to_string(),
            status: None,
            due_date: None,
            sort_key: "a2".to_string(),
            metadata: None,
        },
    )
    .await
    .unwrap();

    // Act
    let result = list_tasks_core(&pool).await;

    // Assert
    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 2);
}

#[tokio::test]
async fn test_update_task_core() {
    // Arrange
    let pool = setup_test_db().await;
    let project = create_project_core(
        &pool,
        CreateProjectPayload {
            title: "Parent Project".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            project_id: None,
            title: "Original".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act
    let payload = UpdateTaskPayload {
        project_id: Some(project.id),
        title: Some("Updated".to_string()),
        status: Some("done".to_string()),
        ..Default::default()
    };
    let result = update_task_core(&pool, task.id, payload).await;

    // Assert
    assert!(result.is_ok());
    let updated_task = result.unwrap();
    assert_eq!(updated_task.title, "Updated");
    assert_eq!(updated_task.status, "done");
    assert_eq!(updated_task.project_id, Some(project.id));
}

#[tokio::test]
async fn test_delete_task_core() {
    // Arrange
    let pool = setup_test_db().await;
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            project_id: None,
            title: "To Delete".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act
    let delete_result = delete_task_core(&pool, task.id).await;
    let get_result = get_task_core(&pool, task.id).await;

    // Assert
    assert!(delete_result.is_ok());
    assert!(get_result.is_err());
}

#[tokio::test]
async fn test_list_inbox_tasks_core() {
    // Arrange
    let pool = setup_test_db().await;
    let project = create_project_core(
        &pool,
        CreateProjectPayload {
            title: "Project".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Inbox task
    create_task_core(
        &pool,
        CreateTaskPayload {
            project_id: None,
            title: "Inbox Task".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    // Project task
    create_task_core(
        &pool,
        CreateTaskPayload {
            project_id: Some(project.id),
            title: "Project Task".to_string(),
            sort_key: "a2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Act
    let result = list_inbox_tasks_core(&pool).await;

    // Assert
    assert!(result.is_ok());
    let inbox_tasks = result.unwrap();
    assert_eq!(inbox_tasks.len(), 1);
    assert_eq!(inbox_tasks[0].title, "Inbox Task");
}

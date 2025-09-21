use app_lib::commands::project_commands::*;
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
async fn test_create_project_core() {
    // Arrange
    let pool = setup_test_db().await;

    let payload = CreateProjectPayload {
        title: "Test Project".to_string(),
        description: Some("Test Description".to_string()),
        icon: None,
        color: None,
        status: None,
        metadata: None,
    };

    // Act
    let result = create_project_core(&pool, payload).await;

    // Assert
    assert!(result.is_ok());
    let project = result.unwrap();
    assert_eq!(project.title, "Test Project");
    assert_eq!(project.description, Some("Test Description".to_string()));
}

#[tokio::test]
async fn test_get_project_core() {
    // Arrange
    let pool = setup_test_db().await;

    let created_project = create_project_core(
        &pool,
        CreateProjectPayload {
            title: "Get Me".to_string(),
            description: None,
            icon: None,
            color: None,
            status: None,
            metadata: None,
        },
    )
    .await
    .unwrap();

    // Act
    let result = get_project_core(&pool, created_project.id).await;

    // Assert
    assert!(result.is_ok());
    let fetched_project = result.unwrap();
    assert_eq!(fetched_project.id, created_project.id);
    assert_eq!(fetched_project.title, "Get Me");
}

#[tokio::test]
async fn test_list_projects_core() {
    // Arrange
    let pool = setup_test_db().await;

    create_project_core(
        &pool,
        CreateProjectPayload {
            title: "Project 1".to_string(),
            description: None,
            icon: None,
            color: None,
            status: None,
            metadata: None,
        },
    )
    .await
    .unwrap();
    create_project_core(
        &pool,
        CreateProjectPayload {
            title: "Project 2".to_string(),
            description: None,
            icon: None,
            color: None,
            status: None,
            metadata: None,
        },
    )
    .await
    .unwrap();

    // Act
    let result = list_projects_core(&pool).await;

    // Assert
    assert!(result.is_ok());
    let projects = result.unwrap();
    assert_eq!(projects.len(), 2);
}

#[tokio::test]
async fn test_update_project_core() {
    // Arrange
    let pool = setup_test_db().await;

    let project = create_project_core(
        &pool,
        CreateProjectPayload {
            title: "Original Title".to_string(),
            description: None,
            icon: None,
            color: None,
            status: None,
            metadata: None,
        },
    )
    .await
    .unwrap();

    // Act
    let update_payload = UpdateProjectPayload {
        title: Some("Updated Title".to_string()),
        description: Some("Updated Description".to_string()),
        icon: None,
        color: None,
        status: None,
        metadata: None,
    };
    let result = update_project_core(&pool, project.id, update_payload).await;

    // Assert
    assert!(result.is_ok());
    let updated_project = result.unwrap();
    assert_eq!(updated_project.title, "Updated Title");
    assert_eq!(
        updated_project.description,
        Some("Updated Description".to_string())
    );
}

#[tokio::test]
async fn test_delete_project_core() {
    // Arrange
    let pool = setup_test_db().await;

    let project = create_project_core(
        &pool,
        CreateProjectPayload {
            title: "To Be Deleted".to_string(),
            description: None,
            icon: None,
            color: None,
            status: None,
            metadata: None,
        },
    )
    .await
    .unwrap();

    // Act
    let delete_result = delete_project_core(&pool, project.id).await;
    let get_result = get_project_core(&pool, project.id).await;

    // Assert
    assert!(delete_result.is_ok());
    assert!(get_result.is_err());
}

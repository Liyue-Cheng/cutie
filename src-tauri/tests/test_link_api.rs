use app_lib::commands::activity_commands::{create_activity_core, CreateActivityPayload};
use app_lib::commands::link_commands::*;
use app_lib::commands::project_commands::{create_project_core, CreateProjectPayload};
use app_lib::commands::tag_commands::{create_tag_core, CreateTagPayload};
use app_lib::commands::task_commands::{create_task_core, CreateTaskPayload};
use app_lib::core::db::DbPool;
use chrono::Utc;
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
async fn test_link_and_unlink_task_to_activity() {
    let pool = setup_test_db().await;
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            title: "T".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let activity = create_activity_core(
        &pool,
        CreateActivityPayload {
            start_time: Utc::now(),
            end_time: Utc::now(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Link
    let link_result = link_task_to_activity_core(&pool, task.id, activity.id).await;
    assert!(link_result.is_ok());

    // Verify link exists
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM task_activity_links WHERE task_id = ? AND activity_id = ?",
    )
    .bind(task.id)
    .bind(activity.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count.0, 1);

    // Unlink
    let unlink_result = unlink_task_from_activity_core(&pool, task.id, activity.id).await;
    assert!(unlink_result.is_ok());

    // Verify link is gone
    let count_after: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM task_activity_links WHERE task_id = ? AND activity_id = ?",
    )
    .bind(task.id)
    .bind(activity.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count_after.0, 0);
}

#[tokio::test]
async fn test_add_and_remove_tag_from_project() {
    let pool = setup_test_db().await;
    let project = create_project_core(
        &pool,
        CreateProjectPayload {
            title: "P".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let tag = create_tag_core(
        &pool,
        CreateTagPayload {
            title: "T".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Add
    let add_result = add_tag_to_project_core(&pool, project.id, tag.id).await;
    assert!(add_result.is_ok());

    // Verify
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM project_tags WHERE project_id = ? AND tag_id = ?")
            .bind(project.id)
            .bind(tag.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count.0, 1);

    // Remove
    let remove_result = remove_tag_from_project_core(&pool, project.id, tag.id).await;
    assert!(remove_result.is_ok());

    // Verify
    let count_after: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM project_tags WHERE project_id = ? AND tag_id = ?")
            .bind(project.id)
            .bind(tag.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count_after.0, 0);
}

#[tokio::test]
async fn test_add_and_remove_tag_from_task() {
    let pool = setup_test_db().await;
    let task = create_task_core(
        &pool,
        CreateTaskPayload {
            title: "T".to_string(),
            sort_key: "a1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let tag = create_tag_core(
        &pool,
        CreateTagPayload {
            title: "T".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Add
    let add_result = add_tag_to_task_core(&pool, task.id, tag.id).await;
    assert!(add_result.is_ok());

    // Verify
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM task_tags WHERE task_id = ? AND tag_id = ?")
            .bind(task.id)
            .bind(tag.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count.0, 1);

    // Remove
    let remove_result = remove_tag_from_task_core(&pool, task.id, tag.id).await;
    assert!(remove_result.is_ok());

    // Verify
    let count_after: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM task_tags WHERE task_id = ? AND tag_id = ?")
            .bind(task.id)
            .bind(tag.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count_after.0, 0);
}

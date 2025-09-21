use app_lib::commands::tag_commands::*;
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
async fn test_create_tag_core() {
    let pool = setup_test_db().await;
    let payload = CreateTagPayload {
        title: "Test Tag".to_string(),
        ..Default::default()
    };
    let result = create_tag_core(&pool, payload).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().title, "Test Tag");
}

#[tokio::test]
async fn test_get_tag_core() {
    let pool = setup_test_db().await;
    let created = create_tag_core(
        &pool,
        CreateTagPayload {
            title: "Get Me".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let result = get_tag_core(&pool, created.id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, created.id);
}

#[tokio::test]
async fn test_list_tags_core() {
    let pool = setup_test_db().await;
    create_tag_core(
        &pool,
        CreateTagPayload {
            title: "T1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    create_tag_core(
        &pool,
        CreateTagPayload {
            title: "T2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let result = list_tags_core(&pool).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn test_update_tag_core() {
    let pool = setup_test_db().await;
    let tag = create_tag_core(
        &pool,
        CreateTagPayload {
            title: "Original".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let payload = UpdateTagPayload {
        title: Some("Updated".to_string()),
        ..Default::default()
    };
    let result = update_tag_core(&pool, tag.id, payload).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().title, "Updated");
}

#[tokio::test]
async fn test_delete_tag_core() {
    let pool = setup_test_db().await;
    let tag = create_tag_core(
        &pool,
        CreateTagPayload {
            title: "To Delete".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let delete_result = delete_tag_core(&pool, tag.id).await;
    let get_result = get_tag_core(&pool, tag.id).await;
    assert!(delete_result.is_ok());
    assert!(get_result.is_err());
}

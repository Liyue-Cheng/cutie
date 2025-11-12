use explore_lib::features::shared::repositories::TaskRepository;
/// 单元测试：TaskRepository
///
/// 测试 Repository 层的数据访问逻辑
use explore_lib::features::shared::TransactionHelper;

mod infrastructure {
    pub use crate::infrastructure::*;
}
use infrastructure::{create_test_db, TaskFixture};

#[tokio::test]
async fn test_insert_and_find_by_id() {
    // Arrange: 准备测试数据库和数据
    let test_db = create_test_db().await.unwrap();
    let task = TaskFixture::new()
        .title("Test Task for Repository")
        .duration(30)
        .build();

    // Act: 插入任务（使用事务）
    let mut tx = TransactionHelper::begin(test_db.pool()).await.unwrap();
    TaskRepository::insert_in_tx(&mut tx, &task).await.unwrap();
    TransactionHelper::commit(tx).await.unwrap();

    // Assert: 验证可以查询到任务
    let found_task = TaskRepository::find_by_id(test_db.pool(), task.id)
        .await
        .unwrap()
        .expect("Task should exist");

    assert_eq!(found_task.id, task.id);
    assert_eq!(found_task.title, "Test Task for Repository");
    assert_eq!(found_task.estimated_duration, Some(30));
}

#[tokio::test]
async fn test_update_task() {
    // Arrange
    let test_db = create_test_db().await.unwrap();
    let task = TaskFixture::new().title("Original Title").build();

    let mut tx = TransactionHelper::begin(test_db.pool()).await.unwrap();
    TaskRepository::insert_in_tx(&mut tx, &task).await.unwrap();
    TransactionHelper::commit(tx).await.unwrap();

    // Act: 更新任务
    let update_request = explore_lib::entities::UpdateTaskRequest {
        title: Some("Updated Title".to_string()),
        glance_note: None,
        detail_note: None,
        estimated_duration: None,
        area_id: None,
        project_id: None,
        due_date: None,
        due_date_type: None,
        subtasks: None,
    };

    let mut tx = TransactionHelper::begin(test_db.pool()).await.unwrap();
    TaskRepository::update_in_tx(&mut tx, task.id, &update_request)
        .await
        .unwrap();
    TransactionHelper::commit(tx).await.unwrap();

    // Assert: 验证更新生效
    let updated_task = TaskRepository::find_by_id(test_db.pool(), task.id)
        .await
        .unwrap()
        .expect("Task should exist");

    assert_eq!(updated_task.title, "Updated Title");
}

#[tokio::test]
async fn test_delete_task() {
    // Arrange
    let test_db = create_test_db().await.unwrap();
    let task = TaskFixture::new().build();

    let mut tx = TransactionHelper::begin(test_db.pool()).await.unwrap();
    TaskRepository::insert_in_tx(&mut tx, &task).await.unwrap();
    TransactionHelper::commit(tx).await.unwrap();

    // Act: 软删除任务
    let deleted_at = chrono::Utc::now();
    let mut tx = TransactionHelper::begin(test_db.pool()).await.unwrap();
    TaskRepository::soft_delete_in_tx(&mut tx, task.id, deleted_at)
        .await
        .unwrap();
    TransactionHelper::commit(tx).await.unwrap();

    // Assert: 验证任务被标记为删除
    let deleted_task = TaskRepository::find_by_id(test_db.pool(), task.id)
        .await
        .unwrap();

    assert!(
        deleted_task.is_none(),
        "Soft-deleted task should not be found"
    );
}

#[tokio::test]
async fn test_list_non_deleted_tasks() {
    // Arrange
    let test_db = create_test_db().await.unwrap();

    let task1 = TaskFixture::new().title("Task 1").build();
    let task2 = TaskFixture::new().title("Task 2").build();
    let task3 = TaskFixture::new().title("Task 3").build();

    let mut tx = TransactionHelper::begin(test_db.pool()).await.unwrap();
    TaskRepository::insert_in_tx(&mut tx, &task1).await.unwrap();
    TaskRepository::insert_in_tx(&mut tx, &task2).await.unwrap();
    TaskRepository::insert_in_tx(&mut tx, &task3).await.unwrap();
    TransactionHelper::commit(tx).await.unwrap();

    // 删除其中一个任务
    let deleted_at = chrono::Utc::now();
    let mut tx = TransactionHelper::begin(test_db.pool()).await.unwrap();
    TaskRepository::soft_delete_in_tx(&mut tx, task2.id, deleted_at)
        .await
        .unwrap();
    TransactionHelper::commit(tx).await.unwrap();

    // Act: 验证未删除的任务可以查到
    let found1 = TaskRepository::find_by_id(test_db.pool(), task1.id)
        .await
        .unwrap();
    let found2 = TaskRepository::find_by_id(test_db.pool(), task2.id)
        .await
        .unwrap();
    let found3 = TaskRepository::find_by_id(test_db.pool(), task3.id)
        .await
        .unwrap();

    // Assert: 只有未删除的任务可以被找到
    assert!(found1.is_some(), "Task 1 should exist");
    assert!(found2.is_none(), "Task 2 should be deleted");
    assert!(found3.is_some(), "Task 3 should exist");
}

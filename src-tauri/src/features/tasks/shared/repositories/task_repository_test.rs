/// 单元测试：TaskRepository
///
/// 测试 Repository 层的数据访问逻辑
#[cfg(test)]
mod tests {
    use crate::features::tasks::shared::repositories::TaskRepository;
    use crate::shared::testing::{create_test_db, fixtures::TaskFixture};

    #[tokio::test]
    async fn test_insert_and_find_by_id() {
        // Arrange: 准备测试数据库和数据
        let test_db = create_test_db().await.unwrap();
        let task = TaskFixture::new()
            .title("Test Task for Repository")
            .duration(30)
            .build();

        // Act: 插入任务
        TaskRepository::insert(test_db.pool(), &task).await.unwrap();

        // Assert: 验证可以查询到任务
        let found_task = TaskRepository::find_by_id(test_db.pool(), &task.id)
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
        let mut task = TaskFixture::new().title("Original Title").build();

        TaskRepository::insert(test_db.pool(), &task).await.unwrap();

        // Act: 更新任务
        task.title = "Updated Title".to_string();
        TaskRepository::update(test_db.pool(), &task).await.unwrap();

        // Assert: 验证更新生效
        let updated_task = TaskRepository::find_by_id(test_db.pool(), &task.id)
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

        TaskRepository::insert(test_db.pool(), &task).await.unwrap();

        // Act: 软删除任务
        TaskRepository::soft_delete(test_db.pool(), &task.id)
            .await
            .unwrap();

        // Assert: 验证任务被标记为删除
        let deleted_task = TaskRepository::find_by_id(test_db.pool(), &task.id)
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

        TaskRepository::insert(test_db.pool(), &task1)
            .await
            .unwrap();
        TaskRepository::insert(test_db.pool(), &task2)
            .await
            .unwrap();
        TaskRepository::insert(test_db.pool(), &task3)
            .await
            .unwrap();

        // 删除其中一个任务
        TaskRepository::soft_delete(test_db.pool(), &task2.id)
            .await
            .unwrap();

        // Act: 查询所有未删除的任务
        let tasks = TaskRepository::list_non_deleted(test_db.pool())
            .await
            .unwrap();

        // Assert: 只返回未删除的任务
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().any(|t| t.id == task1.id));
        assert!(tasks.iter().any(|t| t.id == task3.id));
        assert!(!tasks.iter().any(|t| t.id == task2.id));
    }
}

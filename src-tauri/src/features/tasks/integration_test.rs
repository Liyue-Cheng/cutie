/// 业务集成测试：任务完整生命周期
///
/// 测试多个端点协同工作，验证业务逻辑正确性
#[cfg(test)]
mod tests {
    use crate::{
        entities::{CreateTaskRequest, UpdateTaskRequest},
        features::tasks,
        shared::testing::{create_test_db, http_client::TestClient},
        startup::AppState,
    };
    use axum::{http::StatusCode, Router};

    async fn setup_test_app() -> Router {
        let test_db = create_test_db().await.unwrap();
        let app_state = AppState::new_test(test_db.pool().clone());

        Router::new()
            .nest("/tasks", tasks::create_routes())
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_task_lifecycle_create_update_complete() {
        // Arrange
        let router = setup_test_app().await;
        let client = TestClient::new(router);

        // Step 1: 创建任务
        let create_request = CreateTaskRequest {
            title: "Lifecycle Test Task".to_string(),
            glance_note: Some("Original note".to_string()),
            detail_note: None,
            estimated_duration: Some(30),
            area_id: None,
            due_date: None,
            due_date_type: None,
            subtasks: None,
        };

        let create_response = client.post("/tasks", &create_request).await;
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let created_task: serde_json::Value = create_response.json().await;
        let task_id = created_task["id"].as_str().unwrap();
        assert_eq!(created_task["title"], "Lifecycle Test Task");
        assert_eq!(created_task["is_completed"], false);

        // Step 2: 读取任务，验证创建成功
        let get_response = client.get(&format!("/tasks/{}", task_id)).await;
        assert_eq!(get_response.status(), StatusCode::OK);

        let fetched_task: serde_json::Value = get_response.json().await;
        assert_eq!(fetched_task["id"], task_id);
        assert_eq!(fetched_task["title"], "Lifecycle Test Task");

        // Step 3: 更新任务
        let update_request = UpdateTaskRequest {
            title: Some("Updated Lifecycle Task".to_string()),
            glance_note: Some(Some("Updated note".to_string())),
            detail_note: None,
            estimated_duration: Some(Some(45)),
            area_id: None,
            due_date: None,
            due_date_type: None,
            subtasks: None,
        };

        let update_response = client
            .patch(&format!("/tasks/{}", task_id), &update_request)
            .await;
        assert_eq!(update_response.status(), StatusCode::OK);

        let updated_task: serde_json::Value = update_response.json().await;
        assert_eq!(updated_task["title"], "Updated Lifecycle Task");
        assert_eq!(updated_task["glance_note"], "Updated note");
        assert_eq!(updated_task["estimated_duration"], 45);

        // Step 4: 完成任务
        let complete_response = client
            .post::<serde_json::Value>(&format!("/tasks/{}/completion", task_id), &serde_json::json!({}))
            .await;
        assert_eq!(complete_response.status(), StatusCode::OK);

        let completed_task: serde_json::Value = complete_response.json().await;
        assert_eq!(completed_task["is_completed"], true);
        assert!(completed_task["completed_at"].is_string());

        // Step 5: 验证完成状态持久化
        let final_get_response = client.get(&format!("/tasks/{}", task_id)).await;
        let final_task: serde_json::Value = final_get_response.json().await;
        assert_eq!(final_task["is_completed"], true);
    }

    #[tokio::test]
    async fn test_task_deletion_workflow() {
        // Arrange
        let router = setup_test_app().await;
        let client = TestClient::new(router);

        // Step 1: 创建任务
        let create_request = CreateTaskRequest {
            title: "Task to Delete".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            subtasks: None,
        };

        let create_response = client.post("/tasks", &create_request).await;
        let created_task: serde_json::Value = create_response.json().await;
        let task_id = created_task["id"].as_str().unwrap();

        // Step 2: 软删除任务（移到回收站）
        let delete_response = client.delete(&format!("/tasks/{}", task_id)).await;
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        // Step 3: 验证任务不再出现在正常列表中
        // （这里需要有 list_tasks 端点，暂时跳过）

        // Step 4: 可以从回收站恢复
        // （需要 restore 端点）
    }

    #[tokio::test]
    async fn test_multiple_tasks_creation_and_retrieval() {
        // Arrange
        let router = setup_test_app().await;
        let client = TestClient::new(router);

        // Step 1: 创建多个任务
        for i in 1..=3 {
            let request = CreateTaskRequest {
                title: format!("Task {}", i),
                glance_note: None,
                detail_note: None,
                estimated_duration: Some(i * 15),
                area_id: None,
                due_date: None,
                due_date_type: None,
                subtasks: None,
            };

            let response = client.post("/tasks", &request).await;
            assert_eq!(response.status(), StatusCode::CREATED);
        }

        // Step 2: 验证每个任务都可以独立读取
        // （需要记录 task_id 或使用 list 端点）
    }
}


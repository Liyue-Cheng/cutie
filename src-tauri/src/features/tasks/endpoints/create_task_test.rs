/// 端点测试：POST /api/tasks
///
/// 测试单个 HTTP 端点的行为
#[cfg(test)]
mod tests {
    use crate::{
        entities::CreateTaskRequest,
        features::tasks,
        shared::testing::{create_test_db, http_client::TestClient},
        startup::AppState,
    };
    use axum::{http::StatusCode, Router};

    /// 创建测试用的 AppState 和 Router
    async fn setup_test_app() -> (AppState, Router) {
        let test_db = create_test_db().await.unwrap();
        let app_state = AppState::new_test(test_db.pool().clone());

        let router = Router::new()
            .nest("/tasks", tasks::create_routes())
            .with_state(app_state.clone());

        (app_state, router)
    }

    #[tokio::test]
    async fn test_create_task_success() {
        // Arrange
        let (_app_state, router) = setup_test_app().await;
        let client = TestClient::new(router);

        let request = CreateTaskRequest {
            title: "New Test Task".to_string(),
            glance_note: Some("Quick note".to_string()),
            detail_note: None,
            estimated_duration: Some(60),
            area_id: None,
            due_date: None,
            due_date_type: None,
            subtasks: None,
        };

        // Act: 发送 POST 请求
        let response = client.post("/tasks", &request).await;

        // Assert: 验证响应
        let status = response.status();
        assert_eq!(status, StatusCode::CREATED, "Should return 201 Created");

        let body: serde_json::Value = response.json().await;
        assert_eq!(body["title"], "New Test Task");
        assert_eq!(body["glance_note"], "Quick note");
        assert_eq!(body["estimated_duration"], 60);
        assert_eq!(body["schedule_status"], "staging");
        assert!(body["id"].is_string());
    }

    #[tokio::test]
    async fn test_create_task_validation_error() {
        // Arrange
        let (_app_state, router) = setup_test_app().await;
        let client = TestClient::new(router);

        let request = CreateTaskRequest {
            title: "".to_string(), // 空标题，应该失败
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            subtasks: None,
        };

        // Act
        let response = client.post("/tasks", &request).await;

        // Assert: 应该返回 422
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Empty title should return validation error"
        );
    }

    #[tokio::test]
    async fn test_create_task_with_long_title_error() {
        // Arrange
        let (_app_state, router) = setup_test_app().await;
        let client = TestClient::new(router);

        let long_title = "a".repeat(300); // 超过 255 字符限制

        let request = CreateTaskRequest {
            title: long_title,
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            subtasks: None,
        };

        // Act
        let response = client.post("/tasks", &request).await;

        // Assert
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Title > 255 chars should fail"
        );
    }
}

/// 任务功能模块 - 重构为单文件组件模式
///
/// 新的组织方式：
/// - shared/: 共享的基础设施（repository, dtos, validation）
/// - endpoints/: 每个API的单文件组件实现
///
/// 这种设计遵循Vue单文件组件的思想，每个API都是独立的、完整的实现
use axum::{
    routing::{get, post},
    Router,
};

use crate::startup::AppState;

pub mod endpoints;
pub mod shared;

// 重新导出共享组件
pub use shared::*;

// 重新导出端点处理器
pub use endpoints::*;

/// 创建任务功能模块的路由
///
/// 使用新的单文件组件端点
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 基本CRUD操作
        .route("/", post(endpoints::create_task_handler))
        .route("/:id", get(endpoints::get_task_handler))
        // TODO: 添加其他端点
        // .route("/:id", put(endpoints::update_task_handler))
        // .route("/:id", delete(endpoints::delete_task_handler))
        // 任务状态操作
        .route("/:id/completion", post(endpoints::complete_task_handler))
    // .route("/:id/reopen", post(endpoints::reopen_task_handler))

    // 查询操作
    // .route("/search", get(endpoints::search_tasks_handler))
    // .route("/unscheduled", get(endpoints::get_unscheduled_tasks_handler))
    // .route("/stats", get(endpoints::get_task_stats_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::startup::database::initialize_database;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn create_test_app_state() -> AppState {
        let config = AppConfig::default();
        let db_pool = initialize_database(&config).await.unwrap();
        AppState::new(config, db_pool)
    }

    #[tokio::test]
    async fn test_task_routes_creation() {
        let app_state = create_test_app_state().await;
        let app = create_routes().with_state(app_state);

        let request = Request::builder()
            .method("GET")
            .uri("/nonexistent-id")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // 这应该返回404或其他错误，但不应该panic
        assert!(response.status().is_client_error() || response.status().is_server_error());
    }
}

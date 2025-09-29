/// 模板相关路由定义
use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::template_handlers::*;
use crate::startup::AppState;

/// 创建模板相关的路由
pub fn create_template_routes() -> Router<AppState> {
    Router::new()
        // 模板CRUD操作
        .route("/templates", post(create_template_handler))
        .route("/templates", get(get_templates_handler))
        .route("/templates/stats", get(get_template_stats_handler))
        .route("/templates/:id", get(get_template_handler))
        .route("/templates/:id", put(update_template_handler))
        .route("/templates/:id", delete(delete_template_handler))
        // 模板特殊操作
        .route("/templates/:id/clone", post(clone_template_handler))
        .route(
            "/templates/:id/tasks",
            post(create_task_from_template_handler),
        )
}

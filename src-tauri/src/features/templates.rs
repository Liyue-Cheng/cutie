/// Templates功能模块
/// 
/// 处理模板相关的所有业务逻辑
use axum::{routing::{get, post, put, delete}, Router};
use crate::startup::AppState;

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::templates::*;
}

/// 创建templates相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_templates))
        .route("/", post(endpoints::create_template))
        .route("/:id", put(endpoints::update_template))
        .route("/:id", delete(endpoints::delete_template))
        .route("/:id/create-task", post(endpoints::create_task_from_template))
}

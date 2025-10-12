use crate::startup::AppState;
/// Recurrences功能模块
///
/// 处理循环任务相关的所有业务逻辑
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::recurrences::*;
}

/// 创建recurrences相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_recurrences))
        .route("/", post(endpoints::create_recurrence))
        .route("/:id", patch(endpoints::update_recurrence)) // ✅ 修正：PUT -> PATCH
        .route("/:id", delete(endpoints::delete_recurrence))
        .route(
            "/batch-update-instances",
            patch(endpoints::batch_update_instances),
        )
        .route(
            "/batch-update-template-and-instances",
            patch(endpoints::batch_update_template_and_instances),
        )
}

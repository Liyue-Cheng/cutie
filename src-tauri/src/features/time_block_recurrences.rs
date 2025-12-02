use crate::startup::AppState;
/// TimeBlockRecurrences功能模块
///
/// 处理时间块循环相关的所有业务逻辑
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::time_block_recurrences::*;
}

/// 创建time_block_recurrences相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_time_block_recurrences))
        .route("/", post(endpoints::create_time_block_recurrence))
        .route("/:id", get(endpoints::get_time_block_recurrence))
        .route("/:id", patch(endpoints::update_time_block_recurrence))
        .route("/:id", delete(endpoints::delete_time_block_recurrence))
        .route("/:id/stop", post(endpoints::stop_time_block_recurrence))
        .route("/:id/resume", post(endpoints::resume_time_block_recurrence))
        .route("/:id/edit", post(endpoints::edit_time_block_recurrence))
}

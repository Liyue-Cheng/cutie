use crate::startup::AppState;
/// Time Blocks功能模块
///
/// 处理时间块相关的所有业务逻辑
use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::time_blocks::*;
}

/// 创建time_blocks相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_time_blocks))
        .route("/", post(endpoints::create_time_block))
        .route("/from-task", post(endpoints::create_from_task))
        .route("/:id", patch(endpoints::update_time_block)) // ✅ 修正：PUT -> PATCH
        .route("/:id", delete(endpoints::delete_time_block))
        .route("/:id/link-task", patch(endpoints::link_task))
}

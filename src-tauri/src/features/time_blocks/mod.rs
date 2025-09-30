/// TimeBlocks (时间块) 功能模块
///
/// 提供时间块的完整生命周期管理

use axum::{
    routing::{delete, patch, post},
    Router,
};

use crate::startup::AppState;

pub mod endpoints;

/// 创建 TimeBlocks 路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(endpoints::create_time_block::handle))
        .route("/:id", patch(endpoints::update_time_block::handle))
        .route("/:id", delete(endpoints::delete_time_block::handle))
        .route("/:id/links", post(endpoints::link_task::handle))
        .route("/:id/links", delete(endpoints::unlink_task::handle))
}
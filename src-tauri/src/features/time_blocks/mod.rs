/// TimeBlocks 功能模块 - 单文件组件架构
///
/// 架构原则：
/// - endpoints/ 目录存放纯粹的 SFC 文件（无需 mod.rs）
/// - 每个 SFC 文件导出 `pub async fn handle(...)`
/// - 本文件直接声明 endpoints 子模块并在路由中使用
use axum::{
    routing::{get, post},
    Router,
};

use crate::startup::AppState;

// 直接声明 endpoints 子模块（无需 pub，只内部使用）
mod endpoints {
    pub mod create_time_block; // POST /time-blocks
    pub mod list_time_blocks;  // GET /time-blocks
    // 待实现的其他端点：
    // pub mod update_time_block;
    // pub mod delete_time_block;
    // pub mod link_task;
    // pub mod unlink_task;
}

/// 创建时间块功能模块的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_time_blocks::handle).post(endpoints::create_time_block::handle))
    // .route("/:id", patch(endpoints::update_time_block::handle))
    // .route("/:id", delete(endpoints::delete_time_block::handle))
    // .route("/:id/tasks", post(endpoints::link_task::handle))
    // .route("/:id/tasks/:task_id", delete(endpoints::unlink_task::handle))
}


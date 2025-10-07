/// TimeBlocks 功能模块 - 单文件组件架构
///
/// 架构原则：
/// - endpoints/ 目录存放纯粹的 SFC 文件（无需 mod.rs）
/// - 每个 SFC 文件导出 `pub async fn handle(...)`
/// - 本文件直接声明 endpoints 子模块并在路由中使用
use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::startup::AppState;

// 共享模块（Repositories、ConflictChecker 等工具）
pub mod shared;

// 直接声明 endpoints 子模块（无需 pub，只内部使用）
mod endpoints {
    pub mod create_from_task; // POST /time-blocks/from-task (拖动任务到日历)
    pub mod create_time_block; // POST /time-blocks (直接创建空时间块)
    pub mod delete_time_block; // DELETE /time-blocks/:id
    pub mod link_task; // POST /time-blocks/:id/link-task (将任务链接到已有时间块)
    pub mod list_time_blocks; // GET /time-blocks
    pub mod update_time_block; // PATCH /time-blocks/:id (更新时间块)
                               // 待实现的其他端点：
                               // pub mod unlink_task;
}

/// 创建时间块功能模块的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(endpoints::list_time_blocks::handle).post(endpoints::create_time_block::handle),
        )
        .route("/from-task", post(endpoints::create_from_task::handle))
        .route(
            "/:id",
            delete(endpoints::delete_time_block::handle)
                .patch(endpoints::update_time_block::handle),
        )
        .route("/:id/link-task", post(endpoints::link_task::handle))
    // 待实现：
    // .route("/:id/unlink-task", delete(endpoints::unlink_task::handle))
}

/// 任务功能模块 - 单文件组件架构
///
/// 架构原则：
/// - endpoints/ 目录存放纯粹的 SFC 文件（无需 mod.rs）
/// - 每个 SFC 文件导出 `pub async fn handle(...)`
/// - 本文件直接声明 endpoints 子模块并在路由中使用
/// - DTOs 在 entities 模块中定义
use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::startup::AppState;

// 共享模块（装配器等工具）
pub mod shared;

// 直接声明 endpoints 子模块（无需 pub，只内部使用）
mod endpoints {
    pub mod add_schedule; // POST /tasks/:id/schedules
    pub mod complete_task; // POST /tasks/:id/completion
    pub mod create_task; // POST /tasks
    pub mod delete_schedule; // DELETE /tasks/:id/schedules/:date
    pub mod delete_task; // DELETE /tasks/:id
    pub mod get_task; // GET /tasks/:id
    pub mod reopen_task; // DELETE /tasks/:id/completion
    pub mod return_to_staging; // POST /tasks/:id/return-to-staging
    pub mod update_schedule; // PATCH /tasks/:id/schedules/:date
    pub mod update_task; // PATCH /tasks/:id
}

/// 创建任务功能模块的路由
///
/// 直接使用 endpoints::文件名::handle 的方式引用处理器
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 基本 CRUD 操作
        .route("/", post(endpoints::create_task::handle))
        .route(
            "/:id",
            get(endpoints::get_task::handle)
                .patch(endpoints::update_task::handle)
                .delete(endpoints::delete_task::handle),
        )
        // 任务状态操作
        .route(
            "/:id/completion",
            post(endpoints::complete_task::handle).delete(endpoints::reopen_task::handle),
        )
        // 日程管理操作
        .route("/:id/schedules", post(endpoints::add_schedule::handle))
        .route(
            "/:id/schedules/:date",
            patch(endpoints::update_schedule::handle).delete(endpoints::delete_schedule::handle),
        )
        .route(
            "/:id/return-to-staging",
            post(endpoints::return_to_staging::handle),
        )
    // 查询操作（待实现）
    // .route("/search", get(endpoints::search_tasks::handle))
}

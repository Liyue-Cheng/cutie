use crate::startup::AppState;
/// Tasks功能模块
///
/// 处理任务相关的所有业务逻辑
use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::tasks::*;
}

/// 创建tasks相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(endpoints::create_task))
        .route("/with-schedule", post(endpoints::create_task_with_schedule))
        .route("/:id", get(endpoints::get_task))
        .route("/:id", patch(endpoints::update_task))  // ✅ 修正：PUT -> PATCH
        .route("/:id", delete(endpoints::delete_task))
        .route(
            "/:id/permanently-delete",
            delete(endpoints::permanently_delete_task),
        )
        .route("/:id/complete", patch(endpoints::complete_task))
        .route("/:id/reopen", patch(endpoints::reopen_task))
        .route("/:id/archive", patch(endpoints::archive_task))
        .route("/:id/unarchive", patch(endpoints::unarchive_task))
        .route("/:id/restore", patch(endpoints::restore_task))
        .route(
            "/:id/return-to-staging",
            patch(endpoints::return_to_staging),
        )
        // Schedule management routes - 使用复数 schedules 和 :date 参数
        .route("/:id/schedules/:date", post(endpoints::add_schedule))
        .route("/:id/schedules/:date", patch(endpoints::update_schedule))
        .route("/:id/schedules/:date", delete(endpoints::delete_schedule))
}

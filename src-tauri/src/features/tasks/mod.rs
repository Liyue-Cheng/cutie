/// 任务功能模块 - 单文件组件架构
///
/// 采用单文件组件模式：
/// - endpoints/: 每个API endpoint包含完整的验证、业务逻辑和数据访问代码
/// - 每个endpoint是独立的、自包含的实现
///
/// 注意：
/// - DTOs在entities模块中定义
/// - 不再使用共享的repository和validation
/// - 每个endpoint直接操作数据库
use axum::{
    routing::{get, post},
    Router,
};

use crate::startup::AppState;

pub mod endpoints;
pub mod shared; // 预留给未来可能的共享组件

// 重新导出端点处理器
pub use endpoints::*;

/// 创建任务功能模块的路由
///
/// 使用新的单文件组件端点
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 基本CRUD操作
        .route("/", post(endpoints::create_task_handler))
        .route("/:id", get(endpoints::get_task_handler))
        .route("/:id", axum::routing::patch(endpoints::update_task_handler))
        .route(
            "/:id",
            axum::routing::delete(endpoints::delete_task_handler),
        )
        // 任务状态操作
        .route("/:id/completion", post(endpoints::complete_task_handler))
        .route(
            "/:id/completion",
            axum::routing::delete(endpoints::reopen_task_handler),
        )
        // 取消任务所有日程
        .route(
            "/:id/schedules",
            axum::routing::delete(
                crate::features::schedules::endpoints::unschedule_task_completely::handle,
            ),
        )
    // .route("/:id/reopen", post(endpoints::reopen_task_handler))

    // 查询操作
    // .route("/search", get(endpoints::search_tasks_handler))
    // .route("/unscheduled", get(endpoints::get_unscheduled_tasks_handler))
    // .route("/stats", get(endpoints::get_task_stats_handler))
}

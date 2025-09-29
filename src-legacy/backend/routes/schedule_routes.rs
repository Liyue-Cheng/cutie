/// 日程相关路由定义
use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::schedule_handlers::*;
use crate::startup::AppState;

/// 创建日程相关的路由
pub fn create_schedule_routes() -> Router<AppState> {
    Router::new()
        // 日程操作
        .route("/schedules", post(schedule_task_handler))
        .route("/schedules", get(get_schedules_handler))
        .route("/schedules/stats", get(get_schedule_stats_handler))
        .route("/schedules/:id", delete(delete_schedule_handler))
        .route("/schedules/:id/presence", post(log_presence_handler))
        // 任务日程管理
        .route(
            "/schedules/tasks/:task_id",
            delete(unschedule_task_completely_handler),
        )
}

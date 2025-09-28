/// 任务相关路由定义

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::startup::AppState;
use crate::handlers::{task_handlers::*, schedule_handlers, ordering_handlers};

/// 创建任务相关的路由
pub fn create_task_routes() -> Router<AppState> {
    Router::new()
        // 任务CRUD操作
        .route("/tasks", post(create_task_handler))
        .route("/tasks/search", get(search_tasks_handler))
        .route("/tasks/unscheduled", get(get_unscheduled_tasks_handler))
        .route("/tasks/stats", get(get_task_stats_handler))
        .route("/tasks/:id", get(get_task_handler))
        .route("/tasks/:id", put(update_task_handler))
        .route("/tasks/:id", delete(delete_task_handler))
        
        // 任务状态操作
        .route("/tasks/:id/completion", post(complete_task_handler))
        .route("/tasks/:id/reopen", post(reopen_task_handler))
        
        // 任务日程关联
        .route("/tasks/:id/schedules", get(schedule_handlers::get_task_schedules_handler))
        
        // 任务排序关联
        .route("/tasks/:id/ordering", get(ordering_handlers::get_task_orderings_handler))
}

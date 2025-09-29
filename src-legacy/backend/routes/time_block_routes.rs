/// 时间块相关路由定义

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::startup::AppState;
use crate::handlers::time_block_handlers::*;

/// 创建时间块相关的路由
pub fn create_time_block_routes() -> Router<AppState> {
    Router::new()
        // 时间块CRUD操作
        .route("/time-blocks", post(create_time_block_handler))
        .route("/time-blocks", get(get_time_blocks_handler))
        .route("/time-blocks/:id", get(get_time_block_handler))
        .route("/time-blocks/:id", put(update_time_block_handler))
        .route("/time-blocks/:id", delete(delete_time_block_handler))
        
        // 任务关联操作
        .route("/time-blocks/:id/tasks", post(link_task_to_block_handler))
        .route("/time-blocks/:id/tasks/:task_id", delete(unlink_task_from_block_handler))
        
        // 时间块特殊操作
        .route("/time-blocks/:id/truncate", post(truncate_time_block_handler))
        .route("/time-blocks/:id/extend", post(extend_time_block_handler))
        .route("/time-blocks/:id/split", post(split_time_block_handler))
        
        // 时间相关查询
        .route("/time-blocks/conflicts", get(check_time_conflict_handler))
        .route("/time-blocks/free-slots", get(find_free_slots_handler))
}

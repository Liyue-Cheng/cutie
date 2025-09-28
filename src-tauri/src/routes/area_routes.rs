/// 领域相关路由定义
use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::area_handlers::*;
use crate::startup::AppState;

/// 创建领域相关的路由
pub fn create_area_routes() -> Router<AppState> {
    Router::new()
        // 领域CRUD操作
        .route("/areas", post(create_area_handler))
        .route("/areas", get(get_areas_handler))
        .route("/areas/stats", get(get_area_stats_handler))
        .route("/areas/:id", get(get_area_handler))
        .route("/areas/:id", put(update_area_handler))
        .route("/areas/:id", delete(delete_area_handler))
        // 领域特殊操作
        .route("/areas/:id/path", get(get_area_path_handler))
        .route("/areas/:id/move", post(move_area_handler))
        .route("/areas/:id/restore", post(restore_area_handler))
        .route("/areas/:id/can-delete", get(check_area_can_delete_handler))
}

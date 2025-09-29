/// 排序相关路由定义
use axum::{
    routing::{delete, get, put},
    Router,
};

use crate::handlers::ordering_handlers::*;
use crate::startup::AppState;

/// 创建排序相关的路由
pub fn create_ordering_routes() -> Router<AppState> {
    Router::new()
        // 排序操作
        .route("/ordering", put(update_order_handler))
        .route("/ordering", get(get_context_ordering_handler))
        .route("/ordering", delete(clear_context_ordering_handler))
        .route("/ordering/batch", put(batch_update_ordering_handler))
        .route("/ordering/calculate", get(calculate_sort_order_handler))
}

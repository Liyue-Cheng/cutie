/// Ordering (排序) 功能模块
///
/// 提供任务排序管理

use axum::{routing::{get, put}, Router};

use crate::startup::AppState;

pub mod endpoints;

/// 创建 Ordering 路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_orderings::handle).put(endpoints::update_order::handle))
        .route("/calculate", get(endpoints::calculate_sort_order::handle))
}



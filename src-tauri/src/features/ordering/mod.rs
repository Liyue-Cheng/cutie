/// Ordering (排序) 功能模块
///
/// 提供任务排序管理

use axum::{routing::put, Router};

use crate::startup::AppState;

pub mod endpoints;

/// 创建 Ordering 路由
pub fn create_routes() -> Router<AppState> {
    Router::new().route("/", put(endpoints::update_order::handle))
}



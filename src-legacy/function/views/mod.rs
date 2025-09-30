/// Views (查询视图) 功能模块
///
/// 提供各种只读查询视图
use axum::{routing::get, Router};

use crate::startup::AppState;

pub mod endpoints;

/// 创建 Views 路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/daily-schedule",
            get(endpoints::get_daily_schedule::handle),
        )
        .route("/staging", get(endpoints::get_staging_view::handle))
        .route("/tasks/:id", get(endpoints::get_task_details::handle))
}

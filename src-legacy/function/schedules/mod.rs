/// 日程功能模块
use axum::{routing::post, Router};

use crate::startup::AppState;

pub mod endpoints;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 创建日程
        .route("/", post(endpoints::link_schedule::handle))
        // 删除单个日程
        .route(
            "/:id",
            axum::routing::delete(endpoints::delete_schedule::handle),
        )
        // 记录努力
        .route("/:id/presence", post(endpoints::log_presence::handle))
        // 移动日程
        .route(
            "/:id/reschedule",
            axum::routing::patch(endpoints::reschedule_task::handle),
        )
}

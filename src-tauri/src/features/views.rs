use crate::startup::AppState;
/// Views功能模块
///
/// 处理视图相关的所有业务逻辑
use axum::{routing::get, Router};

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::views::*;
}

/// 创建views相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/all", get(endpoints::get_all))
        .route("/all-incomplete", get(endpoints::get_all_incomplete))
        .route("/daily/:date", get(endpoints::get_daily_tasks)) // ✅ 修正：/daily-tasks -> /daily/:date
        .route("/planned", get(endpoints::get_planned))
        .route("/staging", get(endpoints::get_staging_view))
}

/// Views 功能模块 - 视图聚合端点
///
/// 职责：提供聚合多个实体数据的视图端点
///
/// 架构原则：
/// - 视图端点从多个表聚合数据
/// - 返回为前端优化的 DTO
use axum::{routing::get, Router};

use crate::startup::AppState;

// 共享模块（装配器等工具）
pub mod shared;

// 视图端点
mod endpoints {
    pub mod get_all; // GET /views/all
    pub mod get_all_incomplete; // GET /views/all-incomplete
    pub mod get_daily_tasks; // GET /views/daily/:date
    pub mod get_planned; // GET /views/planned
    pub mod get_staging_view; // GET /views/staging
}

/// 创建 views 功能模块的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 任务列表视图
        .route("/all", get(endpoints::get_all::handle))
        .route(
            "/all-incomplete",
            get(endpoints::get_all_incomplete::handle),
        )
        .route("/planned", get(endpoints::get_planned::handle))
        .route("/staging", get(endpoints::get_staging_view::handle))
        // 日期视图
        .route("/daily/:date", get(endpoints::get_daily_tasks::handle))
}

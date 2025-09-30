/// Views 功能模块 - 视图聚合端点
///
/// 职责：提供聚合多个实体数据的视图端点
/// 
/// 架构原则：
/// - 视图端点从多个表聚合数据
/// - 返回为前端优化的 DTO
use axum::{routing::get, Router};

use crate::startup::AppState;

// 视图端点
mod endpoints {
    pub mod get_staging_view; // GET /views/staging
    // 其他视图端点（待实现）
    // pub mod get_daily_schedule;
}

/// 创建 views 功能模块的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Staging 视图 - 未排期任务列表
        .route("/staging", get(endpoints::get_staging_view::handle))
    // 其他视图端点（待实现）
    // .route("/daily-schedule", get(endpoints::get_daily_schedule::handle))
}

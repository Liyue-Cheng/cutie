/// View Preferences功能模块
/// 
/// 处理视图偏好设置相关的所有业务逻辑
use axum::{routing::{get, post}, Router};
use crate::startup::AppState;

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::view_preferences::*;
}

/// 创建view_preferences相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/:view_key", get(endpoints::get_view_preference))
        .route("/:view_key", post(endpoints::save_view_preference))
}

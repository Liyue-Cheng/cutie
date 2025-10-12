use crate::startup::AppState;
/// View Preferences功能模块
///
/// 处理视图偏好设置相关的所有业务逻辑
use axum::{
    routing::{get, put},
    Router,
};

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::view_preferences::*;
}

/// 创建view_preferences相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 保存/更新视图偏好（RESTful 风格：context_key 在 URL 中）
        .route("/:context_key", put(endpoints::save_view_preference))
        // 获取视图偏好
        .route("/:context_key", get(endpoints::get_view_preference))
}

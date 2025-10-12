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
        // 保存视图偏好（根路径，从请求体中获取 context_key）
        .route("/", put(endpoints::save_view_preference))
        // 获取视图偏好（通过路径参数指定 context_key）
        .route("/:view_key", get(endpoints::get_view_preference))
}

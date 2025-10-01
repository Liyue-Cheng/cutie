/// ViewPreferences 功能模块 - 单文件组件架构
use axum::{routing::{get, put}, Router};
use crate::startup::AppState;

// 直接声明 endpoints 子模块（SFC 不需要 endpoints/mod.rs）
mod endpoints {
    pub mod get_view_preference;
    pub mod save_view_preference;
}

/// 创建 view_preferences 功能模块的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/:context_key", get(endpoints::get_view_preference::handle))
        .route("/", put(endpoints::save_view_preference::handle))
}


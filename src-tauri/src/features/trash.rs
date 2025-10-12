/// Trash功能模块
/// 
/// 处理回收站相关的所有业务逻辑
use axum::{routing::get, Router};
use crate::startup::AppState;

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::trash::*;
}

/// 创建trash相关的路由
pub fn create_routes() -> Router<AppState> {
    use axum::routing::post;
    
    Router::new()
        .route("/", get(endpoints::list_trash))
        .route("/empty", post(endpoints::empty_trash)) // ✅ 修正：DELETE -> POST
}

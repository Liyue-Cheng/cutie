/// Areas功能模块
/// 
/// 处理区域相关的所有业务逻辑
use axum::{routing::{get, post, put, delete}, Router};
use crate::startup::AppState;

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::area::*;
}

/// 创建areas相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_areas))
        .route("/", post(endpoints::create_area))
        .route("/:id", get(endpoints::get_area))
        .route("/:id", put(endpoints::update_area))
        .route("/:id", delete(endpoints::delete_area))
}

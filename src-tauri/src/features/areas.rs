use crate::startup::AppState;
/// Areas功能模块
///
/// 处理区域相关的所有业务逻辑
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

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
        .route("/:id", patch(endpoints::update_area)) // ✅ 修正：PUT -> PATCH (部分更新)
        .route("/:id", delete(endpoints::delete_area))
}

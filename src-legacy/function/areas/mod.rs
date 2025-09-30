/// Areas (领域) 功能模块
///
/// 提供领域的完整生命周期管理

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::startup::AppState;

pub mod endpoints;

/// 创建 Areas 路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_areas::handle))
        .route("/", post(endpoints::create_area::handle))
        .route("/:id", patch(endpoints::update_area::handle))
        .route("/:id", delete(endpoints::delete_area::handle))
}

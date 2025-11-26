/// Projects功能模块
///
/// 处理项目相关的所有业务逻辑
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::startup::AppState;

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::projects::*;
}

/// 创建projects相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Projects 路由
        .route("/", get(endpoints::list_projects))
        .route("/", post(endpoints::create_project))
        .route("/:id", get(endpoints::get_project))
        .route("/:id", patch(endpoints::update_project))
        .route("/:id", delete(endpoints::delete_project))
        // Project Sections 路由
        .route("/:project_id/sections", get(endpoints::list_sections))
        .route("/:project_id/sections", post(endpoints::create_section))
        .route(
            "/:project_id/sections/:id",
            patch(endpoints::update_section),
        )
        .route(
            "/:project_id/sections/:id",
            delete(endpoints::delete_section),
        )
        .route(
            "/:project_id/sections/:id/reorder",
            post(endpoints::reorder_section),
        )
}

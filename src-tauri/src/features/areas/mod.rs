/// Areas 功能模块 - 单文件组件架构
use axum::{routing::get, Router};

use crate::startup::AppState;

// 直接声明 endpoints 子模块
mod endpoints {
    pub mod create_area; // POST /areas
    pub mod delete_area; // DELETE /areas/:id
    pub mod get_area; // GET /areas/:id
    pub mod list_areas; // GET /areas
    pub mod update_area; // PATCH /areas/:id
}

/// 创建 areas 功能模块的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(endpoints::list_areas::handle).post(endpoints::create_area::handle),
        )
        .route(
            "/:id",
            get(endpoints::get_area::handle)
                .patch(endpoints::update_area::handle)
                .delete(endpoints::delete_area::handle),
        )
}

/// Areas 功能模块 - 单文件组件架构
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::startup::AppState;

// 直接声明 endpoints 子模块
mod endpoints {
    pub mod create_area; // POST /areas
    pub mod list_areas; // GET /areas
                        // 待实现的其他端点：
                        // pub mod get_area;
                        // pub mod update_area;
                        // pub mod delete_area;
}

/// 创建 areas 功能模块的路由
pub fn create_routes() -> Router<AppState> {
    Router::new().route(
        "/",
        get(endpoints::list_areas::handle).post(endpoints::create_area::handle),
    )
    // .route("/:id", get(endpoints::get_area::handle))
    // .route("/:id", patch(endpoints::update_area::handle))
    // .route("/:id", delete(endpoints::delete_area::handle))
}

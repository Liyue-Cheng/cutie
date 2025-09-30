/// Templates (模板) 功能模块
///
/// 提供任务模板的完整生命周期管理

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::startup::AppState;

pub mod endpoints;

/// 创建 Templates 路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_templates::handle))
        .route("/", post(endpoints::create_template::handle))
        .route("/:id", patch(endpoints::update_template::handle))
        .route("/:id", delete(endpoints::delete_template::handle))
        .route(
            "/:id/instantiate",
            post(endpoints::instantiate_template::handle),
        )
}

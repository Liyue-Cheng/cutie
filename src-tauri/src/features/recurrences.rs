use crate::startup::AppState;
/// Recurrences功能模块
///
/// 处理循环任务相关的所有业务逻辑
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

// 引入endpoints
mod endpoints {
    pub use crate::features::endpoints::recurrences::*;
}

/// 创建recurrences相关的路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_recurrences))
        .route("/", post(endpoints::create_recurrence))
        .route("/:id", patch(endpoints::update_recurrence))
        .route("/:id", delete(endpoints::delete_recurrence))
        // ✅ 修正批量更新路由：需要 :id 参数
        .route(
            "/:id/instances/batch",
            patch(endpoints::batch_update_instances),
        )
        .route(
            "/:id/template-and-instances",
            patch(endpoints::batch_update_template_and_instances),
        )
}

/// Trash (回收站) Feature Module
///
/// 提供回收站相关的功能：
/// - 查询回收站列表
/// - 清空回收站
use axum::{routing::{get, post}, Router};
use crate::startup::AppState;

pub mod endpoints {
    pub mod list_trash;
    pub mod empty_trash;
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_trash::handle))
        .route("/empty", post(endpoints::empty_trash::handle))
}

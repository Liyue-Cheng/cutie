use axum::{
    routing::{get, post, put},
    Router,
};

use crate::startup::AppState;

mod get_all_settings;
mod get_setting;
mod reset_settings;
mod update_batch_settings;
mod update_setting;

/// 创建用户设置路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_settings::handle))
        .route("/", put(update_batch_settings::handle))
        .route("/reset", post(reset_settings::handle))
        .route("/:key", get(get_setting::handle))
        .route("/:key", put(update_setting::handle))
}

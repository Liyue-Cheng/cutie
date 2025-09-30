/// 日程功能模块
use axum::{routing::post, Router};

use crate::startup::AppState;

pub mod endpoints;

pub fn create_routes() -> Router<AppState> {
    Router::new().route("/", post(endpoints::link_schedule::handle))
}

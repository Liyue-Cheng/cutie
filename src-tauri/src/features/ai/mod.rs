use axum::{routing::post, Router};

use crate::startup::AppState;

pub mod endpoints {
    pub mod chat;
}

pub mod shared;

/// 创建 AI 功能路由
pub fn create_routes() -> Router<AppState> {
    Router::new().route("/chat", post(endpoints::chat::handle))
}

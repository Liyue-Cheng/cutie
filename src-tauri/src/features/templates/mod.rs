/// Templates 功能模块

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::startup::AppState;

pub mod endpoints {
    pub mod create_task_from_template;
    pub mod create_template;
    pub mod delete_template;
    pub mod list_templates;
    pub mod update_template;
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(endpoints::list_templates::handle).post(endpoints::create_template::handle),
        )
        .route(
            "/:id",
            patch(endpoints::update_template::handle).delete(endpoints::delete_template::handle),
        )
        .route(
            "/:id/create-task",
            post(endpoints::create_task_from_template::handle),
        )
}


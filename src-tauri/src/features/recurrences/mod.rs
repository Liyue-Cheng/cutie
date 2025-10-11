use crate::startup::AppState;
use axum::{
    routing::{delete, get, patch},
    Router,
};

pub mod endpoints {
    pub mod batch_update_instances;
    pub mod batch_update_template_and_instances;
    pub mod create_recurrence;
    pub mod delete_recurrence;
    pub mod list_recurrences;
    pub mod update_recurrence;
}

pub mod shared;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(endpoints::list_recurrences::handle).post(endpoints::create_recurrence::handle),
        )
        .route(
            "/:id",
            patch(endpoints::update_recurrence::handle)
                .delete(endpoints::delete_recurrence::handle),
        )
        .route(
            "/:id/instances/batch",
            patch(endpoints::batch_update_instances::handle),
        )
        .route(
            "/:id/template-and-instances",
            patch(endpoints::batch_update_template_and_instances::handle),
        )
}

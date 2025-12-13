use crate::startup::AppState;
/// Shutdown ritual feature module
///
/// Handles daily shutdown ritual steps & per-day progress.
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

// Import endpoints
mod endpoints {
    pub use crate::features::endpoints::shutdown_ritual::*;
}

/// Create shutdown ritual routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/state", get(endpoints::get_state))
        .route("/settings", patch(endpoints::update_settings))
        .route("/steps", post(endpoints::create_step))
        .route("/steps/:id", patch(endpoints::update_step))
        .route("/steps/:id", delete(endpoints::delete_step))
        .route("/steps/:id/order-rank", patch(endpoints::update_step_sort_rank))
        .route("/progress/toggle", post(endpoints::toggle_progress))
}



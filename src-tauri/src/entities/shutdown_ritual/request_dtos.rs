use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateShutdownRitualStepRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShutdownRitualStepRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShutdownRitualStepSortRequest {
    pub prev_step_id: Option<Uuid>,
    pub next_step_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ToggleShutdownRitualRequest {
    pub step_id: Uuid,
    pub date: String, // YYYY-MM-DD
}



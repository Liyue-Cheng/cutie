use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ShutdownRitualStepDto {
    pub id: Uuid,
    pub title: String,
    pub order_rank: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShutdownRitualProgressDto {
    pub step_id: Uuid,
    pub date: String, // YYYY-MM-DD
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShutdownRitualStateDto {
    pub date: String, // YYYY-MM-DD
    pub steps: Vec<ShutdownRitualStepDto>,
    pub progress: Vec<ShutdownRitualProgressDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateShutdownRitualStepSortResponse {
    pub step_id: Uuid,
    pub new_rank: String,
}



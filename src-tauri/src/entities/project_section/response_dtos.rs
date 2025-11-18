/// ProjectSection 响应 DTOs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ProjectSection 详情 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSectionDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub sort_order: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


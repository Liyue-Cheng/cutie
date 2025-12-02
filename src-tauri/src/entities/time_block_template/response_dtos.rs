/// TimeBlockTemplate 响应 DTOs
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::entities::time_block::TimeType;

/// 时间块模板 DTO
#[derive(Debug, Clone, Serialize)]
pub struct TimeBlockTemplateDto {
    pub id: Uuid,
    pub title: Option<String>,
    pub glance_note_template: Option<String>,
    pub detail_note_template: Option<String>,
    pub duration_minutes: i32,
    pub start_time_local: String,
    pub time_type: TimeType,
    pub is_all_day: bool,
    pub area_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

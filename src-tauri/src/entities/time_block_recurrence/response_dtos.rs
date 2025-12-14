/// TimeBlockRecurrence 响应 DTOs
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::entities::time_block::TimeType;

/// 时间块循环规则 DTO
#[derive(Debug, Clone, Serialize)]
pub struct TimeBlockRecurrenceDto {
    pub id: Uuid,
    pub template_id: Uuid,
    pub rule: String,
    pub time_type: TimeType,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub timezone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 时间块循环规则详情 DTO（包含模板信息）
#[derive(Debug, Clone, Serialize)]
pub struct TimeBlockRecurrenceDetailDto {
    pub id: Uuid,
    pub template_id: Uuid,
    pub rule: String,
    pub time_type: TimeType,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub timezone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 关联的模板信息
    pub template: Option<TimeBlockTemplateInfo>,
}

/// 模板简要信息
#[derive(Debug, Clone, Serialize)]
pub struct TimeBlockTemplateInfo {
    pub id: Uuid,
    pub title: Option<String>,
    pub glance_note_template: Option<String>,
    pub detail_note_template: Option<String>,
    pub duration_minutes: i32,
    pub start_time_local: String,
    pub is_all_day: bool,
    pub area_id: Option<Uuid>,
}

/// 编辑时间块循环规则的结果 DTO
#[derive(Debug, Clone, Serialize)]
pub struct TimeBlockRecurrenceEditResultDto {
    pub recurrence: TimeBlockRecurrenceDetailDto,
    pub deleted_time_block_ids: Vec<Uuid>,
    pub deleted_count: usize,
}

/// Project 响应 DTOs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::ProjectStatus;

/// Project 详情 DTO
///
/// 注意：任务统计由前端基于 task store 实时计算，后端不传输统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub area_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

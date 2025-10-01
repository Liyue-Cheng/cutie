/// Area 响应 DTOs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Area 详情 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaDto {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub parent_area_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Area 树形结构 DTO（包含子节点）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaTreeDto {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub parent_area_id: Option<Uuid>,
    pub children: Vec<AreaTreeDto>,
}


/// Area 请求 DTOs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 创建 Area 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAreaRequest {
    pub name: String,
    pub color: String,
    pub parent_area_id: Option<Uuid>,
}

/// 更新 Area 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAreaRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub parent_area_id: Option<Option<Uuid>>,
}

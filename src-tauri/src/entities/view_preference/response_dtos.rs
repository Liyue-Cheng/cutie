use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 视图排序偏好 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewPreferenceDto {
    pub context_key: String,
    pub sorted_task_ids: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

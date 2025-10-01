use serde::{Deserialize, Serialize};

/// 保存视图排序偏好请求
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveViewPreferenceRequest {
    pub context_key: String,
    pub sorted_task_ids: Vec<String>,
}


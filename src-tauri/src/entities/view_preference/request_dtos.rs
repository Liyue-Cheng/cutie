use serde::{Deserialize, Serialize};

/// 保存视图排序偏好请求
///
/// 注意：context_key 从 URL 路径参数获取，不在请求体中
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveViewPreferenceRequest {
    pub sorted_task_ids: Vec<String>,
}

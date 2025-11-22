/// ProjectSection 请求 DTOs
use serde::{Deserialize, Serialize};

/// 创建 ProjectSection 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectSectionRequest {
    pub title: String,
    pub description: Option<String>,
    pub sort_order: Option<String>,
}

/// 更新 ProjectSection 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectSectionRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub sort_order: Option<String>,
}

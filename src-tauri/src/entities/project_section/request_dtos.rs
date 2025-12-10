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
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub description: Option<Option<String>>,
    pub sort_order: Option<String>,
}

/// 自定义反序列化器，用于正确处理三态字段
/// - 字段缺失 → None (不更新)
/// - 字段为 null → Some(None) (设为 NULL)
/// - 字段有值 → Some(Some(value)) (设为值)
fn deserialize_nullable_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    use serde::Deserialize;
    Ok(Some(Option::deserialize(deserializer)?))
}

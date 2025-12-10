/// Project 请求 DTOs
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::ProjectStatus;

/// 创建 Project 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub area_id: Option<Uuid>,
}

/// 更新 Project 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub description: Option<Option<String>>,
    pub status: Option<ProjectStatus>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub due_date: Option<Option<NaiveDate>>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub area_id: Option<Option<Uuid>>,
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

/// Template Request DTOs
///
/// 用于API请求的数据传输对象
use serde::Deserialize;
use uuid::Uuid;

use super::model::TemplateCategory;
use crate::entities::task::Subtask;

/// 创建模板请求
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    /// 模板标题 (必填)
    pub title: String,

    /// 快览笔记模板 (可选)
    pub glance_note_template: Option<String>,

    /// 详细笔记模板 (可选)
    pub detail_note_template: Option<String>,

    /// 预估时长模板 (可选, 单位: 分钟)
    pub estimated_duration_template: Option<i32>,

    /// 子任务模板 (可选)
    pub subtasks_template: Option<Vec<Subtask>>,

    /// 领域ID (可选)
    pub area_id: Option<Uuid>,

    /// 模板类别 (可选，默认为 GENERAL)
    pub category: Option<TemplateCategory>,
}

/// 更新模板请求
#[derive(Debug, Deserialize, Default)]
pub struct UpdateTemplateRequest {
    /// 模板标题
    pub title: Option<String>,

    /// 快览笔记模板
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub glance_note_template: Option<Option<String>>,

    /// 详细笔记模板
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub detail_note_template: Option<Option<String>>,

    /// 预估时长模板 (单位: 分钟)
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub estimated_duration_template: Option<Option<i32>>,

    /// 子任务模板
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub subtasks_template: Option<Option<Vec<Subtask>>>,

    /// 领域ID (嵌套 Option 用于区分"不更新"和"设置为null")
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub area_id: Option<Option<Uuid>>,

    /// 模板类别
    pub category: Option<TemplateCategory>,
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

/// 从模板创建任务请求
#[derive(Debug, Deserialize)]
pub struct CreateTaskFromTemplateRequest {
    /// 模板ID
    pub template_id: Uuid,

    /// 可选的变量替换映射 (如 {"date": "2025-10-09"})
    #[serde(default)]
    pub variables: std::collections::HashMap<String, String>,
}

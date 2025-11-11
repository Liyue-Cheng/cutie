/// TaskRecurrence Request DTOs
///
/// 用于API请求的数据传输对象
use serde::Deserialize;
use uuid::Uuid;

use super::model::{ExpiryBehavior, TimeType};

/// 创建循环规则请求
#[derive(Debug, Deserialize)]
pub struct CreateTaskRecurrenceRequest {
    /// 关联的模板ID (必填)
    pub template_id: Uuid,

    /// 循环字符串 (必填)
    pub rule: String,

    /// 时间类型 (可选，默认为 FLOATING)
    pub time_type: Option<TimeType>,

    /// 生效起始日期 (可选, YYYY-MM-DD)
    pub start_date: Option<String>,

    /// 生效结束日期 (可选, YYYY-MM-DD)
    pub end_date: Option<String>,

    /// 时区 (可选，仅 FIXED 类型使用)
    pub timezone: Option<String>,

    /// 过期行为 (可选，默认为 CARRYOVER_TO_STAGING)
    pub expiry_behavior: Option<ExpiryBehavior>,

    /// 是否激活 (可选，默认为 true)
    pub is_active: Option<bool>,

    /// 源任务ID (可选) - 如果提供且该任务在start_date有日程，将其作为第一个循环实例
    pub source_task_id: Option<Uuid>,
}

/// 更新循环规则请求
#[derive(Debug, Deserialize, Default)]
pub struct UpdateTaskRecurrenceRequest {
    /// 关联的模板ID
    pub template_id: Option<Uuid>,

    /// 循环字符串
    pub rule: Option<String>,

    /// 时间类型
    pub time_type: Option<TimeType>,

    /// 生效起始日期
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub start_date: Option<Option<String>>,

    /// 生效结束日期
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub end_date: Option<Option<String>>,

    /// 时区
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub timezone: Option<Option<String>>,

    /// 过期行为
    pub expiry_behavior: Option<ExpiryBehavior>,

    /// 是否激活
    pub is_active: Option<bool>,
}

/// 自定义反序列化器，用于正确处理三态字段
fn deserialize_nullable_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    use serde::Deserialize;
    Ok(Some(Option::deserialize(deserializer)?))
}

/// TimeBlock 请求 DTOs
///
/// 只包含 API 请求相关的数据传输对象
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TimeType;

/// 创建时间块的请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeBlockRequest {
    pub title: Option<String>,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    /// 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时使用
    pub start_time_local: Option<String>,
    /// 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时使用
    pub end_time_local: Option<String>,
    /// 时间类型，默认为FLOATING
    #[serde(default)]
    pub time_type: Option<TimeType>,
    /// 创建时的时区（占位字段）
    pub creation_timezone: Option<String>,
    pub is_all_day: Option<bool>,
    pub area_id: Option<Uuid>,
    // 🔧 REMOVED: linked_task_ids
    // 职责分离：创建纯时间块不应关联任务
    // 任务关联应使用专门的 POST /time-blocks/from-task 端点
}

/// 更新时间块的请求载荷
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTimeBlockRequest {
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub title: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub glance_note: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub detail_note: Option<Option<String>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    /// 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时使用
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub start_time_local: Option<Option<String>>,
    /// 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时使用
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub end_time_local: Option<Option<String>>,
    /// 时间类型
    pub time_type: Option<TimeType>,
    /// 创建时的时区（占位字段）
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub creation_timezone: Option<Option<String>>,
    pub is_all_day: Option<bool>,
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

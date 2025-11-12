/// TaskRecurrence Response DTOs
///
/// 用于API响应的数据传输对象
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::model::{ExpiryBehavior, TimeType};

/// 循环规则 DTO
#[derive(Debug, Serialize)]
pub struct TaskRecurrenceDto {
    /// 循环规则ID
    pub id: Uuid,

    /// 关联的模板ID
    pub template_id: Uuid,

    /// 循环字符串
    pub rule: String,

    /// 时间类型
    pub time_type: TimeType,

    /// 生效起始日期 (YYYY-MM-DD)
    pub start_date: Option<String>,

    /// 生效结束日期 (YYYY-MM-DD)
    pub end_date: Option<String>,

    /// 时区
    pub timezone: Option<String>,

    /// 过期行为
    pub expiry_behavior: ExpiryBehavior,

    /// 是否激活
    pub is_active: bool,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

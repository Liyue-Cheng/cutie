/// TimeBlockRecurrence 请求 DTOs
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::time_block::TimeType;

/// 创建时间块循环规则请求
#[derive(Debug, Deserialize)]
pub struct CreateTimeBlockRecurrenceRequest {
    /// 关联的模板ID
    pub template_id: Uuid,

    /// 循环规则字符串（RRULE 标准）
    pub rule: String,

    /// 时间类型 (可选，默认 FLOATING)
    pub time_type: Option<TimeType>,

    /// 生效起始日期 (可选, YYYY-MM-DD)
    pub start_date: Option<String>,

    /// 生效结束日期 (可选, YYYY-MM-DD)
    pub end_date: Option<String>,

    /// 时区 (可选)
    pub timezone: Option<String>,

    /// 是否激活 (可选，默认 true)
    pub is_active: Option<bool>,

    /// 源时间块ID（可选，用于将现有时间块作为第一个实例）
    pub source_time_block_id: Option<Uuid>,
}

/// 更新时间块循环规则请求
#[derive(Debug, Deserialize)]
pub struct UpdateTimeBlockRecurrenceRequest {
    /// 关联的模板ID (可选)
    pub template_id: Option<Uuid>,

    /// 循环规则字符串 (可选)
    pub rule: Option<String>,

    /// 时间类型 (可选)
    pub time_type: Option<TimeType>,

    /// 生效起始日期 (可选，使用 Option<Option<String>> 支持设置为 null)
    pub start_date: Option<Option<String>>,

    /// 生效结束日期 (可选，使用 Option<Option<String>> 支持设置为 null)
    pub end_date: Option<Option<String>>,

    /// 时区 (可选，使用 Option<Option<String>> 支持设置为 null)
    pub timezone: Option<Option<String>>,

    /// 是否激活 (可选)
    pub is_active: Option<bool>,
}

/// 编辑时间块循环规则请求（更新规则 + 模板 + 清理未来实例）
#[derive(Debug, Deserialize)]
pub struct EditTimeBlockRecurrenceRequest {
    /// 循环规则字符串 (可选)
    pub rule: Option<String>,

    /// 生效结束日期 (可选，使用 Option<Option<String>> 支持设置为 null)
    pub end_date: Option<Option<String>>,

    /// 时区 (可选，使用 Option<Option<String>> 支持设置为 null)
    pub timezone: Option<Option<String>>,

    /// 时间类型 (可选)
    pub time_type: Option<TimeType>,

    /// 模板标题 (可选, 允许设置为 null)
    pub title: Option<Option<String>>,

    /// 模板快览笔记 (可选, 允许设置为 null)
    pub glance_note_template: Option<Option<String>>,

    /// 模板详细笔记 (可选, 允许设置为 null)
    pub detail_note_template: Option<Option<String>>,

    /// 模板时长（分钟，可选）
    pub duration_minutes: Option<i32>,

    /// 模板是否全天 (可选)
    pub is_all_day: Option<bool>,

    /// 模板所属领域 (可选, 允许设置为 null)
    pub area_id: Option<Option<Uuid>>,

    /// 当前本地时间（YYYY-MM-DDTHH:mm），用于确定不可删除的历史实例
    pub local_now: String,

    /// 是否删除未来实例（默认 true）
    pub delete_future_instances: Option<bool>,
}

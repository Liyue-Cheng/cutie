/// TimeBlockTemplate 请求 DTOs
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::time_block::TimeType;

/// 创建时间块模板请求
#[derive(Debug, Deserialize)]
pub struct CreateTimeBlockTemplateRequest {
    /// 标题模板 (可选)
    pub title: Option<String>,

    /// 快览笔记模板 (可选)
    pub glance_note_template: Option<String>,

    /// 详细笔记模板 (可选)
    pub detail_note_template: Option<String>,

    /// 时长（分钟）
    pub duration_minutes: i32,

    /// 每天开始时间 (HH:MM:SS，如 "08:00:00")
    pub start_time_local: String,

    /// 时间类型 (可选，默认 FLOATING)
    pub time_type: Option<TimeType>,

    /// 是否为全天事件 (可选，默认 false)
    pub is_all_day: Option<bool>,

    /// 领域ID (可选)
    pub area_id: Option<Uuid>,
}

/// 更新时间块模板请求
#[derive(Debug, Deserialize)]
pub struct UpdateTimeBlockTemplateRequest {
    /// 标题模板 (可选)
    pub title: Option<Option<String>>,

    /// 快览笔记模板 (可选)
    pub glance_note_template: Option<Option<String>>,

    /// 详细笔记模板 (可选)
    pub detail_note_template: Option<Option<String>>,

    /// 时长（分钟）
    pub duration_minutes: Option<i32>,

    /// 每天开始时间 (HH:MM:SS)
    pub start_time_local: Option<String>,

    /// 时间类型
    pub time_type: Option<TimeType>,

    /// 是否为全天事件
    pub is_all_day: Option<bool>,

    /// 领域ID
    pub area_id: Option<Option<Uuid>>,
}

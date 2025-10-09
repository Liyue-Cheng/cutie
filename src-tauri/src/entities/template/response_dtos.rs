/// Template Response DTOs
///
/// 用于API响应的数据传输对象
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::model::TemplateCategory;
use crate::entities::task::Subtask;

/// 模板DTO - 用于列表和详情响应
#[derive(Debug, Serialize)]
pub struct TemplateDto {
    /// 模板ID
    pub id: Uuid,

    /// 模板标题
    pub title: String,

    /// 快览笔记模板
    pub glance_note_template: Option<String>,

    /// 详细笔记模板
    pub detail_note_template: Option<String>,

    /// 预估时长模板 (单位: 分钟)
    pub estimated_duration_template: Option<i32>,

    /// 子任务模板
    pub subtasks_template: Option<Vec<Subtask>>,

    /// 领域ID
    pub area_id: Option<Uuid>,

    /// 模板类别
    pub category: TemplateCategory,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

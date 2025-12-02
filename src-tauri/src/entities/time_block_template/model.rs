/// TimeBlockTemplate 核心模型
///
/// 时间块循环模板实体
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::entities::time_block::TimeType;

/// TimeBlockTemplate (时间块循环模板) 实体定义
///
/// 存储循环时间块的模板信息，用于生成循环实例
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeBlockTemplate {
    /// 模板ID (主键)
    pub id: Uuid,

    /// 标题模板 (可选)
    pub title: Option<String>,

    /// 快览笔记模板 (可选)
    pub glance_note_template: Option<String>,

    /// 详细笔记模板 (可选)
    pub detail_note_template: Option<String>,

    /// 时长（分钟）
    pub duration_minutes: i32,

    /// 每天开始时间 (HH:MM:SS)
    pub start_time_local: String,

    /// 时间类型
    pub time_type: TimeType,

    /// 是否为全天事件
    pub is_all_day: bool,

    /// 领域ID (可选)
    pub area_id: Option<Uuid>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 是否删除
    pub is_deleted: bool,
}

/// TimeBlockTemplateRow - 数据库行映射结构
#[derive(Debug, FromRow)]
pub struct TimeBlockTemplateRow {
    pub id: String,
    pub title: Option<String>,
    pub glance_note_template: Option<String>,
    pub detail_note_template: Option<String>,
    pub duration_minutes: i32,
    pub start_time_local: String,
    pub time_type: String,
    pub is_all_day: bool,
    pub area_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl TryFrom<TimeBlockTemplateRow> for TimeBlockTemplate {
    type Error = String;

    fn try_from(row: TimeBlockTemplateRow) -> Result<Self, Self::Error> {
        let time_type = match row.time_type.as_str() {
            "FLOATING" => TimeType::Floating,
            "FIXED" => TimeType::Fixed,
            _ => return Err(format!("Invalid time_type: {}", row.time_type)),
        };

        Ok(TimeBlockTemplate {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            title: row.title,
            glance_note_template: row.glance_note_template,
            detail_note_template: row.detail_note_template,
            duration_minutes: row.duration_minutes,
            start_time_local: row.start_time_local,
            time_type,
            is_all_day: row.is_all_day,
            area_id: row.area_id.as_ref().and_then(|s| Uuid::parse_str(s).ok()),
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        })
    }
}

impl TimeBlockTemplate {
    /// 创建新的时间块模板
    pub fn new(
        id: Uuid,
        duration_minutes: i32,
        start_time_local: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            title: None,
            glance_note_template: None,
            detail_note_template: None,
            duration_minutes,
            start_time_local,
            time_type: TimeType::Floating,
            is_all_day: false,
            area_id: None,
            created_at,
            updated_at: created_at,
            is_deleted: false,
        }
    }
}

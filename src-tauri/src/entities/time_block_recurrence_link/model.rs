/// TimeBlockRecurrenceLink 核心模型
///
/// 时间块循环实例链接实体，用于记录某个循环规则在某一天生成的时间块实例
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// TimeBlockRecurrenceLink (时间块循环实例链接) 实体定义
///
/// 为每条循环规则在"某一天"的实例与时间块建立一条链接
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeBlockRecurrenceLink {
    /// 循环规则ID (外键)
    pub recurrence_id: Uuid,

    /// 实例日期 (YYYY-MM-DD)
    pub instance_date: String,

    /// 关联的时间块ID (外键)
    pub time_block_id: Uuid,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// TimeBlockRecurrenceLinkRow - 数据库行映射结构
#[derive(Debug, FromRow)]
pub struct TimeBlockRecurrenceLinkRow {
    pub recurrence_id: String,
    pub instance_date: String,
    pub time_block_id: String,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<TimeBlockRecurrenceLinkRow> for TimeBlockRecurrenceLink {
    type Error = String;

    fn try_from(row: TimeBlockRecurrenceLinkRow) -> Result<Self, Self::Error> {
        Ok(TimeBlockRecurrenceLink {
            recurrence_id: Uuid::parse_str(&row.recurrence_id).map_err(|e| e.to_string())?,
            instance_date: row.instance_date,
            time_block_id: Uuid::parse_str(&row.time_block_id).map_err(|e| e.to_string())?,
            created_at: row.created_at,
        })
    }
}

impl TimeBlockRecurrenceLink {
    /// 创建新的循环实例链接
    pub fn new(
        recurrence_id: Uuid,
        instance_date: String,
        time_block_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            recurrence_id,
            instance_date,
            time_block_id,
            created_at,
        }
    }
}

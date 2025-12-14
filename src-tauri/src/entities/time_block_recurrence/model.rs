/// TimeBlockRecurrence 核心模型
///
/// 时间块循环规则实体
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::entities::time_block::TimeType;

/// TimeBlockRecurrence (时间块循环规则) 实体定义
///
/// 存储生效的循环规则，用于自动生成时间块实例
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeBlockRecurrence {
    /// 循环规则ID (主键)
    pub id: Uuid,

    /// 关联的模板ID (外键)
    pub template_id: Uuid,

    /// 循环规则字符串（RRULE 标准，如 "FREQ=DAILY", "FREQ=WEEKLY;BYDAY=MO,WE,FR"）
    pub rule: String,

    /// 时间类型（浮动时间 FLOATING 或固定时间 FIXED）
    pub time_type: TimeType,

    /// 生效起始日期 (可选, YYYY-MM-DD)
    pub start_date: Option<String>,

    /// 生效结束日期 (可选, YYYY-MM-DD)
    pub end_date: Option<String>,

    /// 时区 (可选，仅 FIXED 类型使用)
    pub timezone: Option<String>,

    /// 是否激活
    pub is_active: bool,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// TimeBlockRecurrenceRow - 数据库行映射结构
#[derive(Debug, FromRow)]
pub struct TimeBlockRecurrenceRow {
    pub id: String,
    pub template_id: String,
    pub rule: String,
    pub time_type: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub timezone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<TimeBlockRecurrenceRow> for TimeBlockRecurrence {
    type Error = String;

    fn try_from(row: TimeBlockRecurrenceRow) -> Result<Self, Self::Error> {
        let time_type = match row.time_type.as_str() {
            "FLOATING" => TimeType::Floating,
            "FIXED" => TimeType::Fixed,
            _ => return Err(format!("Invalid time_type: {}", row.time_type)),
        };

        Ok(TimeBlockRecurrence {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            template_id: Uuid::parse_str(&row.template_id).map_err(|e| e.to_string())?,
            rule: row.rule,
            time_type,
            start_date: row.start_date,
            end_date: row.end_date,
            timezone: row.timezone,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TimeBlockRecurrence {
    /// 创建新的循环规则
    pub fn new(
        id: Uuid,
        template_id: Uuid,
        rule: String,
        time_type: TimeType,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            template_id,
            rule,
            time_type,
            start_date: None,
            end_date: None,
            timezone: None,
            is_active: true,
            created_at,
            updated_at: created_at,
        }
    }

    /// 检查规则是否在某个日期生效
    pub fn is_effective_on(&self, date: &str) -> bool {
        if !self.is_active {
            return false;
        }

        // 检查起始日期
        if let Some(ref start) = self.start_date {
            if date < start.as_str() {
                return false;
            }
        }

        // 检查结束日期
        if let Some(ref end) = self.end_date {
            if date > end.as_str() {
                return false;
            }
        }

        true
    }
}

/// TaskRecurrence 核心模型
///
/// 循环任务规则实体
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 时间类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeType {
    Floating,
    Fixed,
}

impl TimeType {
    pub fn as_str(&self) -> &str {
        match self {
            TimeType::Floating => "FLOATING",
            TimeType::Fixed => "FIXED",
        }
    }
}

impl TryFrom<&str> for TimeType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "FLOATING" => Ok(TimeType::Floating),
            "FIXED" => Ok(TimeType::Fixed),
            _ => Err(format!("Invalid time type: {}", s)),
        }
    }
}

/// 过期行为枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpiryBehavior {
    CarryoverToStaging,
    Expire,
}

impl ExpiryBehavior {
    pub fn as_str(&self) -> &str {
        match self {
            ExpiryBehavior::CarryoverToStaging => "CARRYOVER_TO_STAGING",
            ExpiryBehavior::Expire => "EXPIRE",
        }
    }
}

impl TryFrom<&str> for ExpiryBehavior {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "CARRYOVER_TO_STAGING" => Ok(ExpiryBehavior::CarryoverToStaging),
            "EXPIRE" => Ok(ExpiryBehavior::Expire),
            _ => Err(format!("Invalid expiry behavior: {}", s)),
        }
    }
}

/// TaskRecurrence (循环任务规则) 实体定义
///
/// 存储生效的循环规则，用于自动生成任务实例
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskRecurrence {
    /// 循环规则ID (主键)
    pub id: Uuid,

    /// 关联的模板ID (外键)
    pub template_id: Uuid,

    /// 循环字符串（如 "RRULE:FREQ=DAILY" 或自定义简化串）
    pub rule: String,

    /// 时间类型（浮动时间 FLOATING 或固定时间 FIXED）
    pub time_type: TimeType,

    /// 生效起始日期 (可选, YYYY-MM-DD)
    pub start_date: Option<String>,

    /// 生效结束日期 (可选, YYYY-MM-DD)
    pub end_date: Option<String>,

    /// 时区 (可选，仅 FIXED 类型使用)
    pub timezone: Option<String>,

    /// 过期行为（过期后是否结转到暂存区）
    pub expiry_behavior: ExpiryBehavior,

    /// 是否激活
    pub is_active: bool,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// TaskRecurrenceRow - 数据库行映射结构
#[derive(Debug, FromRow)]
pub struct TaskRecurrenceRow {
    pub id: String,
    pub template_id: String,
    pub rule: String,
    pub time_type: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub timezone: Option<String>,
    pub expiry_behavior: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<TaskRecurrenceRow> for TaskRecurrence {
    type Error = String;

    fn try_from(row: TaskRecurrenceRow) -> Result<Self, Self::Error> {
        Ok(TaskRecurrence {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            template_id: Uuid::parse_str(&row.template_id).map_err(|e| e.to_string())?,
            rule: row.rule,
            time_type: TimeType::try_from(row.time_type.as_str())?,
            start_date: row.start_date,
            end_date: row.end_date,
            timezone: row.timezone,
            expiry_behavior: ExpiryBehavior::try_from(row.expiry_behavior.as_str())?,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TaskRecurrence {
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
            expiry_behavior: ExpiryBehavior::CarryoverToStaging, // 默认值
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

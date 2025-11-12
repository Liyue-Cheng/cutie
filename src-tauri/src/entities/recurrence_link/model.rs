/// TaskRecurrenceLink 核心模型
///
/// 循环任务实例链接实体，用于记录某个循环规则在某一天生成的任务实例
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// TaskRecurrenceLink (循环任务实例链接) 实体定义
///
/// 为每条循环规则在"某一天"的实例与任务建立一条链接
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskRecurrenceLink {
    /// 循环规则ID (外键)
    pub recurrence_id: Uuid,

    /// 实例日期 (YYYY-MM-DD)
    pub instance_date: String,

    /// 关联的任务ID (外键)
    pub task_id: Uuid,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// TaskRecurrenceLinkRow - 数据库行映射结构
#[derive(Debug, FromRow)]
pub struct TaskRecurrenceLinkRow {
    pub recurrence_id: String,
    pub instance_date: String,
    pub task_id: String,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<TaskRecurrenceLinkRow> for TaskRecurrenceLink {
    type Error = String;

    fn try_from(row: TaskRecurrenceLinkRow) -> Result<Self, Self::Error> {
        Ok(TaskRecurrenceLink {
            recurrence_id: Uuid::parse_str(&row.recurrence_id).map_err(|e| e.to_string())?,
            instance_date: row.instance_date,
            task_id: Uuid::parse_str(&row.task_id).map_err(|e| e.to_string())?,
            created_at: row.created_at,
        })
    }
}

impl TaskRecurrenceLink {
    /// 创建新的循环实例链接
    pub fn new(
        recurrence_id: Uuid,
        instance_date: String,
        task_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            recurrence_id,
            instance_date,
            task_id,
            created_at,
        }
    }
}

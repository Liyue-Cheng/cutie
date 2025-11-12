/// TaskSchedule核心模型
///
/// 从shared/core/models/task_schedule.rs迁移而来
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::entities::task::Outcome;

/// TaskSchedule (任务日程) 实体定义
///
/// 一条关联记录，精确定义了一个Task在哪一天被安排，以及它在那一天的最终"结局"。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSchedule {
    /// 日程ID (主键)
    pub id: Uuid,

    /// 任务ID (外键)
    ///
    /// **不变量:** 必须永远指向一个有效的、未被物理删除的Task
    pub task_id: Uuid,

    /// 安排日期（YYYY-MM-DD 字符串）
    ///
    /// **语义:** 表示用户本地时区的某一天，无时区信息
    /// **前置条件:** 必须是有效的 YYYY-MM-DD 格式字符串
    pub scheduled_date: String,

    /// 结局
    ///
    /// **后置条件:** 忠实记录当日的结局，不应被未来发生的事件（如任务的全局完成）追溯修改
    pub outcome: Outcome,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// TaskScheduleRow - 数据库行映射结构
///
/// 用于直接从数据库查询结果映射
/// SQLx会自动将数据库的TEXT时间字段转换为DateTime<Utc>
#[derive(Debug, FromRow)]
pub struct TaskScheduleRow {
    pub id: String,
    pub task_id: String,
    pub scheduled_date: String, // YYYY-MM-DD 字符串
    pub outcome: String,
    pub created_at: DateTime<Utc>, // SQLx自动转换
    pub updated_at: DateTime<Utc>, // SQLx自动转换
}

impl TryFrom<TaskScheduleRow> for TaskSchedule {
    type Error = String;

    fn try_from(row: TaskScheduleRow) -> Result<Self, Self::Error> {
        let outcome = match row.outcome.as_str() {
            "PLANNED" => Outcome::Planned,
            "PRESENCE_LOGGED" => Outcome::PresenceLogged,
            "COMPLETED_ON_DAY" => Outcome::CompletedOnDay,
            "CARRIED_OVER" => Outcome::CarriedOver,
            _ => return Err(format!("Invalid outcome: {}", row.outcome)),
        };

        Ok(TaskSchedule {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            task_id: Uuid::parse_str(&row.task_id).map_err(|e| e.to_string())?,
            scheduled_date: row.scheduled_date, // YYYY-MM-DD 字符串
            outcome,
            created_at: row.created_at, // SQLx已经转换
            updated_at: row.updated_at, // SQLx已经转换
        })
    }
}

impl TaskSchedule {
    /// 创建新的任务日程
    pub fn new(id: Uuid, task_id: Uuid, scheduled_date: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            task_id,
            scheduled_date,
            outcome: Outcome::Planned,
            created_at,
            updated_at: created_at,
        }
    }

    /// 更新结局
    pub fn update_outcome(&mut self, outcome: Outcome, updated_at: DateTime<Utc>) {
        self.outcome = outcome;
        self.updated_at = updated_at;
    }

    /// 重新安排到新日期
    pub fn reschedule(&mut self, new_date: String, updated_at: DateTime<Utc>) {
        self.scheduled_date = new_date;
        self.outcome = Outcome::Planned; // 重置为计划状态
        self.updated_at = updated_at;
    }

    /// 记录努力
    pub fn log_presence(&mut self, updated_at: DateTime<Utc>) -> Result<(), String> {
        if self.outcome == Outcome::CompletedOnDay {
            return Err("Cannot log presence for already completed task".to_string());
        }

        self.outcome = Outcome::PresenceLogged;
        self.updated_at = updated_at;
        Ok(())
    }

    /// 标记为当日完成
    pub fn mark_completed_on_day(&mut self, updated_at: DateTime<Utc>) {
        self.outcome = Outcome::CompletedOnDay;
        self.updated_at = updated_at;
    }

    /// 标记为延期
    pub fn mark_carried_over(&mut self, updated_at: DateTime<Utc>) {
        self.outcome = Outcome::CarriedOver;
        self.updated_at = updated_at;
    }

    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        self.outcome == Outcome::CompletedOnDay
    }

    /// 检查是否已记录努力
    pub fn has_logged_presence(&self) -> bool {
        matches!(
            self.outcome,
            Outcome::PresenceLogged | Outcome::CompletedOnDay
        )
    }
}

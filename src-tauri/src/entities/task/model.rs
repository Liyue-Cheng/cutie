/// Task核心模型
///
/// 从shared/core/models/task.rs迁移而来
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;
use uuid::Uuid;

use super::{DueDateType, SourceInfo, Subtask};

/// Task (任务) 实体定义
///
/// 代表一个用户需要完成的具体待办事项。它是系统中进行规划、调度和执行的基本原子单位。
///
/// ## 不变量
/// - 所有ID字段必须使用UUID类型
/// - 所有时间戳字段必须使用带时区的UTC时间
/// - id在Task的整个生命周期中永远不变
/// - completed_at字段的值是判断任务是否完成的唯一依据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct Task {
    /// 任务ID (主键)
    ///
    /// **不变量:** 在Task的整个生命周期中，id永远不变
    pub id: Uuid,

    /// 任务标题
    ///
    /// **前置条件:** 创建时不能为空字符串，长度必须在1到255个字符之间
    /// **后置条件:** 代表任务的核心描述
    pub title: String,

    /// 快览笔记 (可选)
    ///
    /// **前置条件:** 长度（如果存在）建议不超过140个字符
    /// **后置条件:** 用于在卡片视图上提供快速上下文，是纯文本
    pub glance_note: Option<String>,

    /// 详细笔记 (可选)
    ///
    /// **前置条件:** 无特定长度限制
    /// **后置条件:** 用于存储任务的详细信息，支持Markdown格式
    pub detail_note: Option<String>,

    /// 预估时长 (分钟, 可选)
    ///
    /// **前置条件:** 必须是正整数
    /// **后置条件:** 代表预估完成任务所需的分钟数。特殊值1被前端解释为"Tiny"
    pub estimated_duration: Option<i32>,

    /// 子任务列表 (可选)
    ///
    /// **前置条件:** 必须是Vec<Subtask>的有效结构
    pub subtasks: Option<Vec<Subtask>>,

    /// 项目ID (外键, 可选)
    ///
    /// **前置条件:** 如果非NULL，必须指向一个存在的Project.id
    pub project_id: Option<Uuid>,

    /// 领域ID (外键, 可选)
    ///
    /// **前置条件:** 如果非NULL，必须指向一个存在的Area.id
    pub area_id: Option<Uuid>,

    /// 截止日期 (可选)
    ///
    /// **前置条件:** 如果非NULL，due_date_type也必须非NULL
    pub due_date: Option<DateTime<Utc>>,

    /// 截止日期类型 (可选)
    ///
    /// **前置条件:** 只有在due_date非NULL时才能有值
    pub due_date_type: Option<DueDateType>,

    /// 完成时间 (可选)
    ///
    /// **不变量:** 此字段的值是判断任务是否完成的唯一依据。IS NOT NULL意味着已完成
    pub completed_at: Option<DateTime<Utc>>,

    /// 归档时间 (可选)
    ///
    /// **后置条件:** 当为Some时，该任务被归档，不应在常规视图中显示
    pub archived_at: Option<DateTime<Utc>>,

    /// 创建时间
    ///
    /// **不变量:** 创建后不可更改
    pub created_at: DateTime<Utc>,

    /// 更新时间
    ///
    /// **后置条件:** 每当任务记录发生任何修改时，此字段必须被更新为当前时间
    pub updated_at: DateTime<Utc>,

    /// 删除时间 (可选)
    ///
    /// **后置条件:** NULL表示未删除，有值表示已删除（软删除）。删除时间用于回收站排序和清理策略
    pub deleted_at: Option<DateTime<Utc>>,

    /// 来源信息 (可选)
    pub source_info: Option<SourceInfo>,

    /// 外部来源ID (可选)
    pub external_source_id: Option<String>,

    /// 外部来源提供商 (可选)
    pub external_source_provider: Option<String>,

    /// 外部来源元数据 (可选)
    pub external_source_metadata: Option<serde_json::Value>,

    /// 重复规则 (可选)
    pub recurrence_rule: Option<String>,

    /// 重复任务父ID (可选)
    pub recurrence_parent_id: Option<Uuid>,

    /// 重复任务原始日期 (可选)
    pub recurrence_original_date: Option<DateTime<Utc>>,

    /// 重复任务排除日期 (可选)
    pub recurrence_exclusions: Option<Vec<DateTime<Utc>>>,
}

impl Task {
    /// 创建新的任务
    pub fn new(id: Uuid, title: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            title,
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            subtasks: None,
            project_id: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            completed_at: None,
            archived_at: None,
            created_at,
            updated_at: created_at,
            deleted_at: None,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: None,
            recurrence_parent_id: None,
            recurrence_original_date: None,
            recurrence_exclusions: None,
        }
    }

    /// 检查任务是否已删除
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// 软删除任务
    pub fn soft_delete(&mut self, deleted_at: DateTime<Utc>) {
        self.deleted_at = Some(deleted_at);
        self.updated_at = deleted_at;
    }

    /// 恢复任务
    pub fn restore(&mut self, updated_at: DateTime<Utc>) {
        self.deleted_at = None;
        self.updated_at = updated_at;
    }

    /// 检查任务是否已完成
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// 完成任务
    pub fn complete(&mut self, completed_at: DateTime<Utc>) {
        self.completed_at = Some(completed_at);
        self.updated_at = completed_at;
    }

    /// 重新打开任务
    pub fn reopen(&mut self, updated_at: DateTime<Utc>) {
        self.completed_at = None;
        self.updated_at = updated_at;
    }

    /// 归档任务
    pub fn archive(&mut self, archived_at: DateTime<Utc>) {
        self.archived_at = Some(archived_at);
        self.updated_at = archived_at;
    }

    /// 取消归档任务
    pub fn unarchive(&mut self, updated_at: DateTime<Utc>) {
        self.archived_at = None;
        self.updated_at = updated_at;
    }

    /// 检查任务是否已归档
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

/// TaskRow - 数据库行映射结构
///
/// 用于直接从数据库查询结果映射
/// SQLx会自动将数据库的TEXT时间字段转换为DateTime<Utc>
#[derive(Debug, FromRow)]
pub struct TaskRow {
    pub id: String,
    pub title: String,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub estimated_duration: Option<i32>,
    pub subtasks: Option<String>, // JSON
    pub project_id: Option<String>,
    pub area_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,     // SQLx自动转换
    pub due_date_type: Option<String>,       // JSON
    pub completed_at: Option<DateTime<Utc>>, // SQLx自动转换
    pub archived_at: Option<DateTime<Utc>>,  // SQLx自动转换
    pub created_at: DateTime<Utc>,           // SQLx自动转换
    pub updated_at: DateTime<Utc>,           // SQLx自动转换
    pub deleted_at: Option<DateTime<Utc>>,   // SQLx自动转换
    pub source_info: Option<String>,         // JSON
    pub external_source_id: Option<String>,
    pub external_source_provider: Option<String>,
    pub external_source_metadata: Option<String>, // JSON
    pub recurrence_rule: Option<String>,
    pub recurrence_parent_id: Option<String>,
    pub recurrence_original_date: Option<DateTime<Utc>>, // SQLx自动转换
    pub recurrence_exclusions: Option<String>,           // JSON
}

impl TryFrom<TaskRow> for Task {
    type Error = String;

    fn try_from(row: TaskRow) -> Result<Self, Self::Error> {
        Ok(Task {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            title: row.title,
            glance_note: row.glance_note,
            detail_note: row.detail_note,
            estimated_duration: row.estimated_duration,
            subtasks: row
                .subtasks
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            project_id: row
                .project_id
                .as_ref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            area_id: row.area_id.as_ref().and_then(|s| Uuid::parse_str(s).ok()),
            due_date: row.due_date, // SQLx已经转换
            due_date_type: row.due_date_type.as_ref().and_then(|s| match s.as_str() {
                "SOFT" => Some(DueDateType::Soft),
                "HARD" => Some(DueDateType::Hard),
                _ => None,
            }),
            completed_at: row.completed_at, // SQLx已经转换
            archived_at: row.archived_at,   // SQLx已经转换
            created_at: row.created_at,     // SQLx已经转换
            updated_at: row.updated_at,     // SQLx已经转换
            deleted_at: row.deleted_at,     // SQLx已经转换
            source_info: row
                .source_info
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            external_source_id: row.external_source_id,
            external_source_provider: row.external_source_provider,
            external_source_metadata: row
                .external_source_metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            recurrence_rule: row.recurrence_rule,
            recurrence_parent_id: row
                .recurrence_parent_id
                .as_ref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            recurrence_original_date: row.recurrence_original_date, // SQLx已经转换
            recurrence_exclusions: row
                .recurrence_exclusions
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
        })
    }
}

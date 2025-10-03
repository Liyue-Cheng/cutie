/// Task 响应 DTOs - 对应前端的视图模型
///
/// 这些 DTOs 与前端的 dtos.ts 中定义的类型一一对应
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{DueDateType, Subtask};

/// TaskCard (任务卡片视图模型)
///
/// 对应前端: src/types/dtos.ts 中的 TaskCard
/// 用途: 在各种看板（每日看板、Staging区、项目列表等）上显示的任务卡片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCardDto {
    // --- 核心身份 ---
    pub id: Uuid,
    pub title: String,
    pub glance_note: Option<String>,

    // --- 核心状态 (已解耦) ---
    pub is_completed: bool,
    pub schedule_status: ScheduleStatus,

    // --- 详细信息 ---
    pub subtasks: Option<Vec<SubtaskDto>>,

    // --- 上下文与聚合信息 ---
    pub area_id: Option<Uuid>, // ✅ 前端通过 area_id 从 area store 获取完整信息
    pub project_id: Option<Uuid>,
    pub schedule_info: Option<ScheduleInfo>,
    pub due_date: Option<DueDateInfo>,

    // --- UI提示标志 ---
    pub has_detail_note: bool,
}

/// TaskDetail (任务详情视图模型)
///
/// 对应前端: src/types/dtos.ts 中的 TaskDetail
/// 用途: 任务的完整详情，继承 TaskCard 的所有属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetailDto {
    // --- 继承自 TaskCard 的所有属性 ---
    #[serde(flatten)]
    pub card: TaskCardDto,

    // --- 额外增加的深度信息 ---
    /// 完整的详细笔记
    pub detail_note: Option<String>,

    /// 完整的日程安排历史与未来
    pub schedules: Vec<ScheduleRecord>,

    /// 完整的项目信息
    pub project: Option<ProjectSummary>,

    /// 审计与调试信息
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- 辅助结构体 ---

/// 日程状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleStatus {
    Scheduled,
    Staging,
}

/// 子任务DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskDto {
    pub id: Uuid,
    pub title: String,
    pub is_completed: bool,
    pub sort_order: String,
}

impl From<Subtask> for SubtaskDto {
    fn from(subtask: Subtask) -> Self {
        Self {
            id: subtask.id,
            title: subtask.title,
            is_completed: subtask.is_completed,
            sort_order: subtask.sort_order,
        }
    }
}

/// 区域摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaSummary {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

/// 日程信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub outcome_for_today: Option<DailyOutcome>,
    pub is_recurring: bool,
    pub linked_schedule_count: i32,
}

/// 当日结局类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DailyOutcome {
    Planned,
    PresenceLogged,
    Completed,
    CarriedOver,
}

/// 截止日期信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DueDateInfo {
    pub date: DateTime<Utc>,
    #[serde(rename = "type")]
    pub due_date_type: DueDateType,
    pub is_overdue: bool,
}

/// 日程记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRecord {
    pub day: DateTime<Utc>,
    pub outcome: DailyOutcome,
}

/// 项目摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub name: String,
}

// 注意：从 Task 实体转换为 TaskCardDto/TaskDetailDto 的逻辑
// 应该在 features/tasks/shared/assembler.rs 中实现

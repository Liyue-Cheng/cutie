/// HTTP请求载荷定义
/// 
/// 定义所有API端点的请求体结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::models::{DueDateType, Subtask, ContextType};

/// 创建任务请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskPayload {
    /// 任务标题
    pub title: String,
    
    /// 快览笔记
    pub glance_note: Option<String>,
    
    /// 详细笔记
    pub detail_note: Option<String>,
    
    /// 预估时长（分钟）
    pub estimated_duration: Option<i32>,
    
    /// 子任务列表
    pub subtasks: Option<Vec<Subtask>>,
    
    /// 领域ID
    pub area_id: Option<Uuid>,
    
    /// 截止日期
    pub due_date: Option<DateTime<Utc>>,
    
    /// 截止日期类型
    pub due_date_type: Option<DueDateType>,
    
    /// 创建上下文
    pub context: CreationContextPayload,
}

/// 创建上下文载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationContextPayload {
    /// 上下文类型
    pub context_type: ContextType,
    
    /// 上下文ID
    pub context_id: String,
}

/// 更新任务请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskPayload {
    /// 任务标题
    pub title: Option<String>,
    
    /// 快览笔记
    pub glance_note: Option<Option<String>>,
    
    /// 详细笔记
    pub detail_note: Option<Option<String>>,
    
    /// 预估时长（分钟）
    pub estimated_duration: Option<Option<i32>>,
    
    /// 子任务列表
    pub subtasks: Option<Option<Vec<Subtask>>>,
    
    /// 项目ID
    pub project_id: Option<Option<Uuid>>,
    
    /// 领域ID
    pub area_id: Option<Option<Uuid>>,
    
    /// 截止日期
    pub due_date: Option<Option<DateTime<Utc>>>,
    
    /// 截止日期类型
    pub due_date_type: Option<Option<DueDateType>>,
}

/// 安排任务请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTaskPayload {
    /// 任务ID
    pub task_id: Uuid,
    
    /// 目标日期
    pub target_day: DateTime<Utc>,
    
    /// 操作模式：move（移动）或link（链接）
    pub mode: ScheduleMode,
    
    /// 源日程ID（仅在move模式下需要）
    pub source_schedule_id: Option<Uuid>,
}

/// 安排模式枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleMode {
    /// 移动现有日程到新日期
    Move,
    /// 为任务创建额外的日程
    Link,
}

/// 更新排序请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderPayload {
    /// 上下文类型
    pub context_type: ContextType,
    
    /// 上下文ID
    pub context_id: String,
    
    /// 任务ID
    pub task_id: Uuid,
    
    /// 新的排序位置
    pub new_sort_order: String,
}

/// 创建时间块请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeBlockPayload {
    /// 时间块标题
    pub title: Option<String>,
    
    /// 快览笔记
    pub glance_note: Option<String>,
    
    /// 详细笔记
    pub detail_note: Option<String>,
    
    /// 开始时间
    pub start_time: DateTime<Utc>,
    
    /// 结束时间
    pub end_time: DateTime<Utc>,
    
    /// 领域ID
    pub area_id: Option<Uuid>,
    
    /// 关联的任务ID列表
    pub task_ids: Vec<Uuid>,
}

/// 更新时间块请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTimeBlockPayload {
    /// 时间块标题
    pub title: Option<Option<String>>,
    
    /// 快览笔记
    pub glance_note: Option<Option<String>>,
    
    /// 详细笔记
    pub detail_note: Option<Option<String>>,
    
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    
    /// 领域ID
    pub area_id: Option<Option<Uuid>>,
}

/// 链接任务到时间块请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTaskToBlockPayload {
    /// 任务ID
    pub task_id: Uuid,
}

/// 创建模板请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplatePayload {
    /// 模板名称
    pub name: String,
    
    /// 标题模板
    pub title_template: String,
    
    /// 快览笔记模板
    pub glance_note_template: Option<String>,
    
    /// 详细笔记模板
    pub detail_note_template: Option<String>,
    
    /// 预估时长模板
    pub estimated_duration_template: Option<i32>,
    
    /// 子任务模板
    pub subtasks_template: Option<Vec<Subtask>>,
    
    /// 领域ID
    pub area_id: Option<Uuid>,
}

/// 更新模板请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplatePayload {
    /// 模板名称
    pub name: Option<String>,
    
    /// 标题模板
    pub title_template: Option<String>,
    
    /// 快览笔记模板
    pub glance_note_template: Option<Option<String>>,
    
    /// 详细笔记模板
    pub detail_note_template: Option<Option<String>>,
    
    /// 预估时长模板
    pub estimated_duration_template: Option<Option<i32>>,
    
    /// 子任务模板
    pub subtasks_template: Option<Option<Vec<Subtask>>>,
    
    /// 领域ID
    pub area_id: Option<Option<Uuid>>,
}

/// 基于模板创建任务请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskFromTemplatePayload {
    /// 模板ID
    pub template_id: Uuid,
    
    /// 创建上下文
    pub context: CreationContextPayload,
}

/// 创建领域请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAreaPayload {
    /// 领域名称
    pub name: String,
    
    /// 颜色代码
    pub color: String,
    
    /// 父领域ID
    pub parent_area_id: Option<Uuid>,
}

/// 更新领域请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAreaPayload {
    /// 领域名称
    pub name: Option<String>,
    
    /// 颜色代码
    pub color: Option<String>,
    
    /// 父领域ID
    pub parent_area_id: Option<Option<Uuid>>,
}

/// AI任务细化请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineTaskPayload {
    /// 任务ID
    pub task_id: Uuid,
    
    /// 提示键（可选）
    pub prompt_key: Option<String>,
}

/// 记录努力请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPresencePayload {
    /// 日程ID
    pub schedule_id: Uuid,
}

impl From<CreateTaskPayload> for crate::services::CreateTaskData {
    fn from(payload: CreateTaskPayload) -> Self {
        Self {
            title: payload.title,
            glance_note: payload.glance_note,
            detail_note: payload.detail_note,
            estimated_duration: payload.estimated_duration,
            subtasks: payload.subtasks,
            area_id: payload.area_id,
            due_date: payload.due_date,
            due_date_type: payload.due_date_type,
        }
    }
}

impl From<CreationContextPayload> for crate::services::CreationContext {
    fn from(payload: CreationContextPayload) -> Self {
        Self {
            context_type: payload.context_type,
            context_id: payload.context_id,
        }
    }
}

impl From<UpdateTaskPayload> for crate::services::UpdateTaskData {
    fn from(payload: UpdateTaskPayload) -> Self {
        Self {
            title: payload.title,
            glance_note: payload.glance_note,
            detail_note: payload.detail_note,
            estimated_duration: payload.estimated_duration,
            subtasks: payload.subtasks,
            project_id: payload.project_id,
            area_id: payload.area_id,
            due_date: payload.due_date,
            due_date_type: payload.due_date_type,
        }
    }
}

impl From<CreateTimeBlockPayload> for crate::services::CreateTimeBlockData {
    fn from(payload: CreateTimeBlockPayload) -> Self {
        Self {
            title: payload.title,
            glance_note: payload.glance_note,
            detail_note: payload.detail_note,
            start_time: payload.start_time,
            end_time: payload.end_time,
            area_id: payload.area_id,
            task_ids: payload.task_ids,
        }
    }
}

impl From<UpdateTimeBlockPayload> for crate::services::UpdateTimeBlockData {
    fn from(payload: UpdateTimeBlockPayload) -> Self {
        Self {
            title: payload.title,
            glance_note: payload.glance_note,
            detail_note: payload.detail_note,
            start_time: payload.start_time,
            end_time: payload.end_time,
            area_id: payload.area_id,
        }
    }
}

impl From<UpdateOrderPayload> for crate::services::UpdateOrderCommand {
    fn from(payload: UpdateOrderPayload) -> Self {
        Self {
            task_id: payload.task_id,
            context_type: payload.context_type,
            context_id: payload.context_id,
            new_sort_order: payload.new_sort_order,
        }
    }
}

impl From<CreateTemplatePayload> for crate::services::CreateTemplateData {
    fn from(payload: CreateTemplatePayload) -> Self {
        Self {
            name: payload.name,
            title_template: payload.title_template,
            glance_note_template: payload.glance_note_template,
            detail_note_template: payload.detail_note_template,
            estimated_duration_template: payload.estimated_duration_template,
            subtasks_template: payload.subtasks_template,
            area_id: payload.area_id,
        }
    }
}

impl From<UpdateTemplatePayload> for crate::services::UpdateTemplateData {
    fn from(payload: UpdateTemplatePayload) -> Self {
        Self {
            name: payload.name,
            title_template: payload.title_template,
            glance_note_template: payload.glance_note_template,
            detail_note_template: payload.detail_note_template,
            estimated_duration_template: payload.estimated_duration_template,
            subtasks_template: payload.subtasks_template,
            area_id: payload.area_id,
        }
    }
}

impl From<CreateAreaPayload> for crate::services::CreateAreaData {
    fn from(payload: CreateAreaPayload) -> Self {
        Self {
            name: payload.name,
            color: payload.color,
            parent_area_id: payload.parent_area_id,
        }
    }
}

impl From<UpdateAreaPayload> for crate::services::UpdateAreaData {
    fn from(payload: UpdateAreaPayload) -> Self {
        Self {
            name: payload.name,
            color: payload.color,
            parent_area_id: payload.parent_area_id,
        }
    }
}

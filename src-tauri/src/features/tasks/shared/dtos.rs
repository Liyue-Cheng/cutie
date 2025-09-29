/// 任务功能的数据传输对象
///
/// 定义所有任务API端点共享的DTO结构
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::core::{ContextType, DueDateType, Subtask, Task, ValidationError};

/// 创建任务的请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub estimated_duration: Option<i32>,
    pub subtasks: Option<Vec<Subtask>>,
    pub area_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub due_date_type: Option<DueDateType>,
    pub context: CreationContext,
}

/// 创建上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationContext {
    pub context_type: ContextType,
    pub context_id: String,
}

/// 更新任务的请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub glance_note: Option<Option<String>>,
    pub detail_note: Option<Option<String>>,
    pub estimated_duration: Option<Option<i32>>,
    pub subtasks: Option<Option<Vec<Subtask>>>,
    pub project_id: Option<Option<Uuid>>,
    pub area_id: Option<Option<Uuid>>,
    pub due_date: Option<Option<DateTime<Utc>>>,
    pub due_date_type: Option<Option<DueDateType>>,
}

/// 任务响应体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub estimated_duration: Option<i32>,
    pub subtasks: Option<Vec<Subtask>>,
    pub project_id: Option<Uuid>,
    pub area_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub due_date_type: Option<DueDateType>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            glance_note: task.glance_note,
            detail_note: task.detail_note,
            estimated_duration: task.estimated_duration,
            subtasks: task.subtasks,
            project_id: task.project_id,
            area_id: task.area_id,
            due_date: task.due_date,
            due_date_type: task.due_date_type,
            completed_at: task.completed_at,
            created_at: task.created_at,
            updated_at: task.updated_at,
            is_deleted: task.is_deleted,
        }
    }
}

/// 任务统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatsResponse {
    pub total_count: i64,
    pub completed_count: i64,
    pub pending_count: i64,
    pub overdue_count: i64,
    pub today_count: i64,
    pub this_week_count: i64,
    pub this_month_count: i64,
}

/// 搜索查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            q: None,
            limit: Some(50),
        }
    }
}

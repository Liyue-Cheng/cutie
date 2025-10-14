use super::{TaskCardDto, TimeBlockViewDto};
/// 事务结果 (Transaction Result)
///
/// 统一的事务响应结构，用于 HTTP 响应和 SSE 事件。
/// 确保前端从两个渠道接收到完全一致的数据。
use serde::Serialize;

/// 任务事务结果
///
/// 包含主资源（任务）和所有副作用（时间块、其他任务等）。
/// HTTP 响应和 SSE 事件使用相同的数据结构。
#[derive(Debug, Serialize, Clone)]
pub struct TaskTransactionResult {
    /// 主资源：任务
    pub task: TaskCardDto,

    /// 副作用：修改的其他资源
    #[serde(skip_serializing_if = "SideEffects::is_empty")]
    pub side_effects: SideEffects,
}

/// 副作用集合
///
/// 包含事务中修改的所有关联资源。
/// 如果所有字段都为空，在序列化时会被跳过。
#[derive(Debug, Serialize, Clone, Default)]
pub struct SideEffects {
    /// 被删除的时间块
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_time_blocks: Option<Vec<TimeBlockViewDto>>,

    /// 被截断的时间块
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated_time_blocks: Option<Vec<TimeBlockViewDto>>,

    /// 被更新的时间块
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_time_blocks: Option<Vec<TimeBlockViewDto>>,

    /// 被创建的时间块
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time_blocks: Option<Vec<TimeBlockViewDto>>,

    /// 被更新的其他任务（如果有跨任务副作用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_tasks: Option<Vec<TaskCardDto>>,
}

impl SideEffects {
    /// 检查是否所有副作用都为空
    pub fn is_empty(&self) -> bool {
        self.deleted_time_blocks.is_none()
            && self.truncated_time_blocks.is_none()
            && self.updated_time_blocks.is_none()
            && self.created_time_blocks.is_none()
            && self.updated_tasks.is_none()
    }

    /// 创建空的副作用集合
    pub fn empty() -> Self {
        Self::default()
    }
}

/// 时间块事务结果
///
/// 用于时间块相关操作的统一响应结构
#[derive(Debug, Serialize, Clone)]
pub struct TimeBlockTransactionResult {
    /// 主资源：时间块
    pub time_block: TimeBlockViewDto,

    /// 副作用：修改的其他资源
    #[serde(skip_serializing_if = "TimeBlockSideEffects::is_empty")]
    pub side_effects: TimeBlockSideEffects,
}

/// 时间块操作的副作用
#[derive(Debug, Serialize, Clone, Default)]
pub struct TimeBlockSideEffects {
    /// 被更新的任务（可能是一个或多个）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_tasks: Option<Vec<TaskCardDto>>,

    /// 其他被更新的时间块
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_time_blocks: Option<Vec<TimeBlockViewDto>>,
}

impl TimeBlockSideEffects {
    /// 检查是否所有副作用都为空
    pub fn is_empty(&self) -> bool {
        self.updated_tasks.is_none() && self.updated_time_blocks.is_none()
    }

    /// 创建空的副作用集合
    pub fn empty() -> Self {
        Self::default()
    }
}

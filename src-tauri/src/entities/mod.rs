/// 实体层 (Entities Layer)
///
/// 本层统一管理所有数据结构，按纯业务概念组织：
/// - 每个概念包含其核心模型、DTOs、枚举和值对象
/// - 不包含HTTP响应结构（应在shared/http中）
/// - 不使用shared等通用命名
// 按业务概念组织的实体
pub mod area;
pub mod recurrence_link;
pub mod schedule;
pub mod task;
pub mod task_recurrence;
pub mod template;
pub mod time_block;
pub mod view_preference;

// 显式导出所有公共类型，避免 ambiguous glob re-exports 警告

// Area 相关类型
pub use area::{Area, AreaDto, AreaRow, AreaTreeDto, CreateAreaRequest, UpdateAreaRequest};

// RecurrenceLink 相关类型
pub use recurrence_link::{TaskRecurrenceLink, TaskRecurrenceLinkRow};

// Schedule 相关类型
pub use schedule::{TaskSchedule, TaskScheduleRow};

// Task 相关类型
pub use task::{
    // 响应 DTOs
    AreaSummary,
    // 枚举
    ContextType,
    // 请求 DTOs
    CreateTaskRequest,
    DailyOutcome,
    DueDateInfo,
    DueDateType,
    Outcome,
    ProjectSummary,
    ScheduleInfo,
    ScheduleRecord,
    ScheduleStatus,
    SearchTasksQuery,
    // 值对象
    SourceInfo,
    Subtask,
    SubtaskDto,
    // 核心模型
    Task,
    TaskCardDto,
    TaskDetailDto,
    TaskRow,
    UpdateTaskRequest,
};

// Template 相关类型
pub use template::{Template, TemplateRow};

// TaskRecurrence 相关类型
pub use task_recurrence::{
    CreateTaskRecurrenceRequest, TaskRecurrence, TaskRecurrenceDto, TaskRecurrenceRow, TimeType,
    UpdateTaskRecurrenceRequest,
};

// TimeBlock 相关类型
pub use time_block::{
    CreateTimeBlockRequest, LinkedTaskSummary, TimeBlock, TimeBlockRow, TimeBlockViewDto,
    UpdateTimeBlockRequest,
};

use serde::{Deserialize, Serialize};

/// 截止日期类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DueDateType {
    /// 软截止日期 - 提醒性质
    Soft,
    /// 硬截止日期 - 必须完成
    Hard,
}

/// 任务日程结局枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Outcome {
    /// 已计划
    Planned,
    /// 已记录努力
    PresenceLogged,
    /// 当日完成
    CompletedOnDay,
    /// 延期
    CarriedOver,
}

/// 项目状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProjectStatus {
    /// 活跃
    Active,
    /// 暂停
    Paused,
    /// 完成
    Completed,
    /// 归档
    Archived,
}

/// 项目类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProjectType {
    /// 项目
    Project,
    /// 体验
    Experience,
}

/// 上下文类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContextType {
    /// 每日看板
    DailyKanban,
    /// 项目列表
    ProjectList,
    /// 领域过滤
    AreaFilter,
    /// 其他
    Misc,
}

/// 子任务结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subtask {
    /// 子任务ID
    pub id: uuid::Uuid,
    /// 子任务标题
    pub title: String,
    /// 是否完成
    pub is_completed: bool,
    /// 排序顺序
    pub sort_order: String,
}

/// 项目资源结构（JSON存储）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectResources {
    /// 资源链接
    pub links: Vec<String>,
    /// 文件路径
    pub files: Vec<String>,
    /// 其他资源
    pub others: std::collections::HashMap<String, serde_json::Value>,
}

/// 外部来源信息结构（JSON存储）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceInfo {
    /// 来源类型
    pub source_type: String,
    /// 来源描述
    pub description: Option<String>,
    /// 其他信息
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

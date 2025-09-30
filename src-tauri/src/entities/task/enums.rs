/// Task相关的枚举类型
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

impl std::fmt::Display for ContextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextType::DailyKanban => write!(f, "DAILY_KANBAN"),
            ContextType::ProjectList => write!(f, "PROJECT_LIST"),
            ContextType::AreaFilter => write!(f, "AREA_FILTER"),
            ContextType::Misc => write!(f, "MISC"),
        }
    }
}

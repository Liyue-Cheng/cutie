/// Ordering核心模型
///
/// 从shared/core/models/ordering.rs迁移而来

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::task::ContextType;

/// Ordering (排序) 实体定义
///
/// 一条关联记录，定义了一个Task在某个特定“上下文”中的显示顺序。
///
/// ## 不变量
/// - context_type和context_id的组合必须唯一地标识一个可排序的视图
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ordering {
    /// 排序ID (主键)
    pub id: Uuid,

    /// 上下文类型
    ///
    /// **不变量:** context_type和context_id的组合必须唯一地标识一个可排序的视图
    pub context_type: ContextType,

    /// 上下文ID
    ///
    /// **前置条件:** 必须遵循已定义的规范化格式：
    /// - DAILY_KANBAN: Unix时间戳字符串 (e.g., '1729555200')
    /// - PROJECT_LIST: project::{project_id}
    /// - AREA_FILTER: area::{area_id}
    /// - MISC: 纯小写蛇形命名 (e.g., 'floating', 'staging_all')
    pub context_id: String,

    /// 任务ID (外键)
    pub task_id: Uuid,

    /// 排序顺序
    ///
    /// **前置条件:** 必须是符合LexoRank或类似算法格式的有效排序字符串
    pub sort_order: String,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Ordering {
    /// 创建新的排序记录
    pub fn new(
        id: Uuid,
        context_type: ContextType,
        context_id: String,
        task_id: Uuid,
        sort_order: String,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        // 验证context_id格式
        Self::validate_context_id(&context_type, &context_id)?;

        Ok(Self {
            id,
            context_type,
            context_id,
            task_id,
            sort_order,
            updated_at,
        })
    }

    /// 验证context_id格式
    pub fn validate_context_id(context_type: &ContextType, context_id: &str) -> Result<(), String> {
        match context_type {
            ContextType::DailyKanban => {
                // 验证是否为Unix时间戳字符串
                context_id.parse::<i64>().map_err(|_| {
                    "DailyKanban context_id must be a valid Unix timestamp string".to_string()
                })?;
            }
            ContextType::ProjectList => {
                if !context_id.starts_with("project::") {
                    return Err("ProjectList context_id must start with 'project::'".to_string());
                }
                // 验证UUID部分
                let uuid_part = &context_id[9..];
                Uuid::parse_str(uuid_part).map_err(|_| {
                    "ProjectList context_id must contain a valid UUID after 'project::'".to_string()
                })?;
            }
            ContextType::AreaFilter => {
                if !context_id.starts_with("area::") {
                    return Err("AreaFilter context_id must start with 'area::'".to_string());
                }
                // 验证UUID部分
                let uuid_part = &context_id[6..];
                Uuid::parse_str(uuid_part).map_err(|_| {
                    "AreaFilter context_id must contain a valid UUID after 'area::'".to_string()
                })?;
            }
            ContextType::Misc => {
                // 验证纯小写蛇形命名
                if !context_id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_')
                {
                    return Err("Misc context_id must be lowercase snake_case".to_string());
                }
            }
        }
        Ok(())
    }

    /// 更新排序顺序
    pub fn update_sort_order(&mut self, new_sort_order: String, updated_at: DateTime<Utc>) {
        self.sort_order = new_sort_order;
        self.updated_at = updated_at;
    }

    /// 获取项目ID（如果是项目列表上下文）
    pub fn get_project_id(&self) -> Option<Uuid> {
        if self.context_type == ContextType::ProjectList && self.context_id.starts_with("project::")
        {
            Uuid::parse_str(&self.context_id[9..]).ok()
        } else {
            None
        }
    }

    /// 获取领域ID（如果是领域过滤上下文）
    pub fn get_area_id(&self) -> Option<Uuid> {
        if self.context_type == ContextType::AreaFilter && self.context_id.starts_with("area::") {
            Uuid::parse_str(&self.context_id[6..]).ok()
        } else {
            None
        }
    }

    /// 获取日期时间戳（如果是每日看板上下文）
    pub fn get_daily_timestamp(&self) -> Option<i64> {
        if self.context_type == ContextType::DailyKanban {
            self.context_id.parse().ok()
        } else {
            None
        }
    }
}

/// Project核心模型
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Project (项目) 状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectStatus {
    Active,
    Completed,
}

impl ProjectStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "ACTIVE",
            ProjectStatus::Completed => "COMPLETED",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(ProjectStatus::Active),
            "COMPLETED" => Ok(ProjectStatus::Completed),
            _ => Err(format!("Invalid project status: {}", s)),
        }
    }
}

/// Project (项目) 实体定义
///
/// 代表一个项目容器，用于组织和管理任务
///
/// ## 不变量
/// - 项目颜色从关联的 Area 继承
/// - total_tasks 和 completed_tasks 由后端维护
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// 项目ID (主键)
    pub id: Uuid,

    /// 项目名称
    pub name: String,

    /// 项目描述
    pub description: Option<String>,

    /// 项目状态
    pub status: ProjectStatus,

    /// 截止日期 (YYYY-MM-DD)
    pub due_date: Option<NaiveDate>,

    /// 完成时间 (UTC)
    pub completed_at: Option<DateTime<Utc>>,

    /// 关联的 Area ID（用于颜色继承）
    pub area_id: Option<Uuid>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 逻辑删除标记
    pub is_deleted: bool,
}

/// ProjectRow - 数据库行映射结构
#[derive(Debug, FromRow)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub due_date: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub area_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl TryFrom<ProjectRow> for Project {
    type Error = String;

    fn try_from(row: ProjectRow) -> Result<Self, Self::Error> {
        Ok(Project {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            name: row.name,
            description: row.description,
            status: ProjectStatus::from_str(&row.status)?,
            due_date: row
                .due_date
                .as_ref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
            completed_at: row.completed_at,
            area_id: row
                .area_id
                .as_ref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        })
    }
}

impl Project {
    /// 创建新的项目
    pub fn new(id: Uuid, name: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            name,
            description: None,
            status: ProjectStatus::Active,
            due_date: None,
            completed_at: None,
            area_id: None,
            created_at,
            updated_at: created_at,
            is_deleted: false,
        }
    }

    /// 完成项目
    pub fn complete(&mut self, completed_at: DateTime<Utc>) {
        self.status = ProjectStatus::Completed;
        self.completed_at = Some(completed_at);
        self.updated_at = completed_at;
    }

    /// 重新打开项目
    pub fn reopen(&mut self, updated_at: DateTime<Utc>) {
        self.status = ProjectStatus::Active;
        self.completed_at = None;
        self.updated_at = updated_at;
    }

    /// 检查项目是否完成
    pub fn is_completed(&self) -> bool {
        matches!(self.status, ProjectStatus::Completed)
    }
}


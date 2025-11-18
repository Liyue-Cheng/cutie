/// ProjectSection核心模型
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// ProjectSection (项目章节) 实体定义
///
/// 代表项目下的一个章节，用于组织任务
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSection {
    /// 章节ID (主键)
    pub id: Uuid,

    /// 所属项目ID
    pub project_id: Uuid,

    /// 章节标题
    pub title: String,

    /// 章节描述
    pub description: Option<String>,

    /// 排序字段 (Lexorank)
    pub sort_order: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 逻辑删除标记
    pub is_deleted: bool,
}

/// ProjectSectionRow - 数据库行映射结构
#[derive(Debug, FromRow)]
pub struct ProjectSectionRow {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub sort_order: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl TryFrom<ProjectSectionRow> for ProjectSection {
    type Error = String;

    fn try_from(row: ProjectSectionRow) -> Result<Self, Self::Error> {
        Ok(ProjectSection {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            project_id: Uuid::parse_str(&row.project_id).map_err(|e| e.to_string())?,
            title: row.title,
            description: row.description,
            sort_order: row.sort_order,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        })
    }
}

impl ProjectSection {
    /// 创建新的章节
    pub fn new(id: Uuid, project_id: Uuid, title: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            project_id,
            title,
            description: None,
            sort_order: None,
            created_at,
            updated_at: created_at,
            is_deleted: false,
        }
    }
}


/// Project 请求 DTOs
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::ProjectStatus;

/// 创建 Project 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub area_id: Option<Uuid>,
}

/// 更新 Project 的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<ProjectStatus>,
    pub due_date: Option<Option<NaiveDate>>,
    pub area_id: Option<Option<Uuid>>,
}

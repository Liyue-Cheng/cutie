/// 模板相关的HTTP处理器
///
/// 实现所有与模板相关的API端点处理逻辑
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    error_handler::{created_response, no_content_response, success_response},
    payloads::{CreateTaskFromTemplatePayload, CreateTemplatePayload, UpdateTemplatePayload},
};
use crate::common::error::AppError;
use crate::startup::AppState;

/// 查询参数：搜索模板
#[derive(Debug, Deserialize)]
pub struct SearchTemplatesQuery {
    /// 搜索关键词
    pub q: Option<String>,

    /// 领域ID
    pub area_id: Option<Uuid>,

    /// 变量名
    pub variable: Option<String>,
}

/// 创建模板处理器
///
/// **端点:** `POST /templates`
pub async fn create_template_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTemplatePayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Creating template with name: {}", payload.name);

    // 转换载荷为服务层数据结构
    let create_data = crate::services::CreateTemplateData::from(payload);

    // 调用服务层
    let created_template = app_state
        .template_service
        .create_template(create_data)
        .await?;

    log::info!("Template created successfully: {}", created_template.id);

    Ok(created_response(created_template))
}

/// 获取模板详情处理器
///
/// **端点:** `GET /templates/{id}`
pub async fn get_template_handler(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting template: {}", template_id);

    let template = app_state
        .template_service
        .get_template(template_id)
        .await?
        .ok_or_else(|| AppError::not_found("Template", template_id.to_string()))?;

    Ok(success_response(template))
}

/// 更新模板处理器
///
/// **端点:** `PUT /templates/{id}`
pub async fn update_template_handler(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<UpdateTemplatePayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Updating template: {}", template_id);

    // 转换载荷为服务层数据结构
    let update_data = crate::services::UpdateTemplateData::from(payload);

    // 调用服务层
    let updated_template = app_state
        .template_service
        .update_template(template_id, update_data)
        .await?;

    log::info!("Template updated successfully: {}", template_id);

    Ok(success_response(updated_template))
}

/// 删除模板处理器
///
/// **端点:** `DELETE /templates/{id}`
pub async fn delete_template_handler(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Deleting template: {}", template_id);

    // 调用服务层
    app_state
        .template_service
        .delete_template(template_id)
        .await?;

    log::info!("Template deleted successfully: {}", template_id);

    Ok(no_content_response())
}

/// 获取所有模板处理器
///
/// **端点:** `GET /templates`
pub async fn get_templates_handler(
    State(app_state): State<AppState>,
    Query(query): Query<SearchTemplatesQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let templates = if let Some(search_query) = query.q {
        // 按名称搜索模板
        log::debug!("Searching templates with query: '{}'", search_query);
        app_state
            .template_service
            .search_templates(&search_query)
            .await?
    } else if let Some(area_id) = query.area_id {
        // 按领域查找模板
        log::debug!("Getting templates for area: {}", area_id);
        app_state
            .template_service
            .get_templates_by_area(area_id)
            .await?
    } else if let Some(variable) = query.variable {
        // 查找包含特定变量的模板
        log::debug!("Finding templates containing variable: {}", variable);
        app_state
            .template_service
            .find_templates_with_variable(&variable)
            .await?
    } else {
        // 获取所有模板
        log::debug!("Getting all templates");
        app_state.template_service.get_all_templates().await?
    };

    Ok(success_response(templates))
}

/// 基于模板创建任务处理器
///
/// **端点:** `POST /templates/{id}/tasks`
pub async fn create_task_from_template_handler(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<CreateTaskFromTemplatePayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Creating task from template: {}", template_id);

    // 转换载荷为服务层数据结构
    let context = crate::services::CreationContext::from(payload.context);

    // 调用服务层
    let created_task = app_state
        .template_service
        .create_task_from_template(template_id, &context)
        .await?;

    log::info!(
        "Task created from template {} successfully: {}",
        template_id,
        created_task.id
    );

    Ok(created_response(created_task))
}

/// 克隆模板处理器
///
/// **端点:** `POST /templates/{id}/clone`
#[derive(Debug, serde::Deserialize)]
pub struct CloneTemplatePayload {
    /// 新模板名称
    pub new_name: String,
}

pub async fn clone_template_handler(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<CloneTemplatePayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Cloning template {} with new name: {}",
        template_id,
        payload.new_name
    );

    // 调用服务层
    let cloned_template = app_state
        .template_service
        .clone_template(template_id, payload.new_name)
        .await?;

    log::info!(
        "Template {} cloned successfully: {}",
        template_id,
        cloned_template.id
    );

    Ok(created_response(cloned_template))
}

/// 获取模板统计处理器
///
/// **端点:** `GET /templates/stats`
pub async fn get_template_stats_handler(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting template statistics");

    let stats = app_state.template_service.get_template_statistics().await?;

    let stats_response = super::responses::StatsResponse {
        stats: serde_json::to_value(stats).unwrap_or(serde_json::Value::Null),
        generated_at: chrono::Utc::now(),
        range: None,
    };

    Ok(success_response(stats_response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ContextType;

    #[test]
    fn test_create_template_payload_serialization() {
        let payload = CreateTemplatePayload {
            name: "Daily Standup".to_string(),
            title_template: "Standup - {{date}}".to_string(),
            glance_note_template: Some("Daily team sync".to_string()),
            detail_note_template: None,
            estimated_duration_template: Some(30),
            subtasks_template: None,
            area_id: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Daily Standup"));
        assert!(json.contains("{{date}}"));

        let deserialized: CreateTemplatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, payload.name);
        assert_eq!(deserialized.title_template, payload.title_template);
    }

    #[test]
    fn test_create_task_from_template_payload_serialization() {
        let payload = CreateTaskFromTemplatePayload {
            template_id: Uuid::new_v4(),
            context: crate::handlers::payloads::CreationContextPayload {
                context_type: ContextType::DailyKanban,
                context_id: "1729555200".to_string(),
            },
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("DAILY_KANBAN"));

        let deserialized: CreateTaskFromTemplatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.template_id, payload.template_id);
    }

    #[test]
    fn test_search_templates_query_parsing() {
        let query_str = "q=meeting&area_id=550e8400-e29b-41d4-a716-446655440000";
        let query: SearchTemplatesQuery = serde_urlencoded::from_str(query_str).unwrap();

        assert_eq!(query.q, Some("meeting".to_string()));
        assert!(query.area_id.is_some());
        assert!(query.variable.is_none());
    }
}

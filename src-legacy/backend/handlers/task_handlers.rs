/// 任务相关的HTTP处理器
///
/// 实现所有与任务相关的API端点处理逻辑
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error_handler::{created_response, no_content_response, success_response},
    payloads::{CreateTaskPayload, UpdateTaskPayload},
    responses::ApiResponse,
};
use crate::common::error::AppError;
use crate::core::models::Task;
use crate::startup::AppState;

/// 查询参数：搜索任务
#[derive(Debug, Deserialize)]
pub struct SearchTasksQuery {
    /// 搜索关键词
    pub q: Option<String>,

    /// 限制数量
    pub limit: Option<i64>,
}

/// 创建任务处理器
///
/// **端点:** `POST /tasks`
/// **函数签名:** `pub async fn create_task_handler(State<AppState>, Json<CreateTaskPayload>) -> Result<Json<Task>, AppError>`
/// **预期行为简介:** 在指定的上下文中创建一个新任务。
/// **输入规范:**
/// - `content-type`: `application/json`
/// - `schema`: `CreateTaskPayload`，包含`title`, `glance_note` (optional), `detail_note` (optional), `context`等字段。
/// **后置条件:**
/// - `201 Created`: 成功创建。响应体为新创建的`Task`对象的完整JSON。
/// - `422 Unprocessable Entity`: 输入数据验证失败（如`title`为空）。
/// - `404 Not Found`: `context`中指定的实体（如`project_id`）不存在。
/// **预期副作用:** 调用`TaskService::create_in_context`。
pub async fn create_task_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTaskPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Creating task with title: {}", payload.title);

    // 转换载荷为服务层数据结构
    let create_data = crate::services::CreateTaskData::from(payload.clone());
    let context = crate::services::CreationContext::from(payload.context);

    // 调用服务层
    let created_task = app_state
        .task_service
        .create_in_context(create_data, &context)
        .await?;

    log::info!("Task created successfully: {}", created_task.id);

    // 返回201 Created响应
    Ok(created_response(created_task))
}

/// 获取任务详情处理器
///
/// **端点:** `GET /tasks/{id}`
pub async fn get_task_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting task: {}", task_id);

    let task = app_state
        .task_service
        .get_task(task_id)
        .await?
        .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

    Ok(success_response(task))
}

/// 更新任务处理器
///
/// **端点:** `PUT /tasks/{id}`
pub async fn update_task_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Updating task: {}", task_id);

    // 转换载荷为服务层数据结构
    let update_data = crate::services::UpdateTaskData::from(payload);

    // 调用服务层
    let updated_task = app_state
        .task_service
        .update_task(task_id, update_data)
        .await?;

    log::info!("Task updated successfully: {}", task_id);

    Ok(success_response(updated_task))
}

/// 完成任务处理器
///
/// **端点:** `POST /tasks/{id}/completion`
/// **函数签名:** `pub async fn complete_task_handler(State<AppState>, Path<Uuid>) -> Result<Json<Task>, AppError>`
/// **预期行为简介:** 全局完成一个任务。
/// **输入规范:** `id` (path, UUID) - 要完成的任务ID。
/// **后置条件:**
/// - `200 OK`: 成功完成或任务本就已完成（幂等）。响应体为更新后的`Task`对象。
/// - `404 Not Found`: 任务不存在。
/// **预期副作用:** 调用`TaskService::complete_task`，可能触发大量数据库写操作和WebSocket推送。
pub async fn complete_task_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Completing task: {}", task_id);

    // 调用服务层
    let completed_task = app_state.task_service.complete_task(task_id).await?;

    log::info!("Task completed successfully: {}", task_id);

    Ok(success_response(completed_task))
}

/// 重新打开任务处理器
///
/// **端点:** `POST /tasks/{id}/reopen`
pub async fn reopen_task_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Reopening task: {}", task_id);

    // 调用服务层
    let reopened_task = app_state.task_service.reopen_task(task_id).await?;

    log::info!("Task reopened successfully: {}", task_id);

    Ok(success_response(reopened_task))
}

/// 删除任务处理器
///
/// **端点:** `DELETE /tasks/{id}`
pub async fn delete_task_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Deleting task: {}", task_id);

    // 调用服务层
    app_state.task_service.delete_task(task_id).await?;

    log::info!("Task deleted successfully: {}", task_id);

    Ok(no_content_response())
}

/// 搜索任务处理器
///
/// **端点:** `GET /tasks/search`
pub async fn search_tasks_handler(
    State(app_state): State<AppState>,
    Query(query): Query<SearchTasksQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let search_query = query.q.unwrap_or_default();
    let limit = query.limit;

    log::debug!("Searching tasks with query: '{}'", search_query);

    let tasks = if search_query.is_empty() {
        // 如果没有搜索词，返回未安排的任务
        app_state.task_service.get_unscheduled_tasks().await?
    } else {
        app_state
            .task_service
            .search_tasks(&search_query, limit)
            .await?
    };

    Ok(success_response(tasks))
}

/// 获取未安排任务处理器（Staging区）
///
/// **端点:** `GET /tasks/unscheduled`
pub async fn get_unscheduled_tasks_handler(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting unscheduled tasks");

    let tasks = app_state.task_service.get_unscheduled_tasks().await?;

    Ok(success_response(tasks))
}

/// 获取任务统计处理器
///
/// **端点:** `GET /tasks/stats`
pub async fn get_task_stats_handler(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting task statistics");

    let stats = app_state.task_service.get_task_statistics().await?;

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
    fn test_create_task_payload_serialization() {
        let payload = CreateTaskPayload {
            title: "Test Task".to_string(),
            glance_note: Some("Test note".to_string()),
            detail_note: None,
            estimated_duration: Some(60),
            subtasks: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            context: crate::handlers::payloads::CreationContextPayload {
                context_type: ContextType::Misc,
                context_id: "floating".to_string(),
            },
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Test Task"));
        assert!(json.contains("floating"));

        let deserialized: CreateTaskPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, payload.title);
        assert_eq!(deserialized.context.context_id, payload.context.context_id);
    }

    #[test]
    fn test_update_task_payload_serialization() {
        let payload = UpdateTaskPayload {
            title: Some("Updated Task".to_string()),
            glance_note: Some(Some("Updated note".to_string())),
            detail_note: Some(None),
            estimated_duration: None,
            subtasks: None,
            project_id: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Updated Task"));

        let deserialized: UpdateTaskPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, payload.title);
    }

    #[test]
    fn test_search_tasks_query_parsing() {
        // 测试查询参数解析
        let query_str = "q=test&limit=10";
        let query: SearchTasksQuery = serde_urlencoded::from_str(query_str).unwrap();

        assert_eq!(query.q, Some("test".to_string()));
        assert_eq!(query.limit, Some(10));
    }
}

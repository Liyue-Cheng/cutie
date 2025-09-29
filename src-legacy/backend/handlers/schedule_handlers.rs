/// 日程相关的HTTP处理器
///
/// 实现所有与任务日程相关的API端点处理逻辑
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    error_handler::{created_response, no_content_response, success_response},
    payloads::{LogPresencePayload, ScheduleMode, ScheduleTaskPayload},
};
use crate::common::error::AppError;
use crate::core::models::TaskSchedule;
use crate::startup::AppState;

/// 查询参数：获取日期范围内的日程
#[derive(Debug, Deserialize)]
pub struct GetSchedulesQuery {
    /// 开始日期
    pub start_date: Option<DateTime<Utc>>,

    /// 结束日期
    pub end_date: Option<DateTime<Utc>>,

    /// 特定日期
    pub date: Option<DateTime<Utc>>,
}

/// 安排任务处理器
///
/// **端点:** `POST /schedules`
/// **函数签名:** `pub async fn schedule_task_handler(State<AppState>, Json<ScheduleTaskPayload>) -> Result<Json<TaskSchedule>, AppError>`
/// **预期行为简介:** 安排一个任务到某一天，支持"移动"或"链接"模式。
/// **输入规范:**
/// - `schema`: `ScheduleTaskPayload`，包含`task_id`和`target_day`，以及一个区分`move`或`link`的模式字段。
/// **后置条件:**
/// - `201 Created` (for link) / `200 OK` (for move): 操作成功。响应体为新创建或更新后的`TaskSchedule`对象。
/// - `409 Conflict`: 任务已完成，无法安排。
/// - `404 Not Found`: 任务或（在move模式下）源日程不存在。
/// **预期副作用:** 调用`ScheduleService::schedule_task`。
pub async fn schedule_task_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<ScheduleTaskPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Scheduling task {} for {}, mode: {:?}",
        payload.task_id,
        payload.target_day,
        payload.mode
    );

    match payload.mode {
        ScheduleMode::Link => {
            // 创建额外的日程安排
            let schedule = app_state
                .schedule_service
                .create_additional_schedule(payload.task_id, payload.target_day)
                .await?;

            log::info!("Task {} linked to {}", payload.task_id, payload.target_day);
            Ok(created_response(schedule))
        }
        ScheduleMode::Move => {
            // 移动现有日程
            let source_schedule_id = payload.source_schedule_id.ok_or_else(|| {
                AppError::validation_error(
                    "source_schedule_id",
                    "Source schedule ID is required for move mode",
                    "SOURCE_SCHEDULE_ID_REQUIRED",
                )
            })?;

            let schedule = app_state
                .schedule_service
                .reschedule_task(source_schedule_id, payload.target_day)
                .await?;

            log::info!("Task {} moved to {}", payload.task_id, payload.target_day);
            Ok(created_response(schedule))
        }
    }
}

/// 取消任务所有日程处理器
///
/// **端点:** `DELETE /schedules/tasks/{taskId}`
/// **函数签名:** `pub async fn unschedule_task_completely_handler(State<AppState>, Path<Uuid>) -> Result<StatusCode, AppError>`
/// **预期行为简介:** 将一个任务从所有日程中移除，使其回归Staging。
/// **输入规范:** `taskId` (path, UUID)。
/// **后置条件:**
/// - `204 No Content`: 操作成功。
/// - `404 Not Found`: 任务不存在。
/// **预期副作用:** 调用`ScheduleService::unschedule_task_completely`。
pub async fn unschedule_task_completely_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Unscheduling task completely: {}", task_id);

    // 调用服务层
    app_state
        .schedule_service
        .unschedule_task_completely(task_id)
        .await?;

    log::info!("Task {} unscheduled completely", task_id);

    Ok(no_content_response())
}

/// 删除单个日程处理器
///
/// **端点:** `DELETE /schedules/{id}`
pub async fn delete_schedule_handler(
    State(app_state): State<AppState>,
    Path(schedule_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Deleting schedule: {}", schedule_id);

    // 调用服务层
    app_state
        .schedule_service
        .delete_schedule(schedule_id)
        .await?;

    log::info!("Schedule {} deleted successfully", schedule_id);

    Ok(no_content_response())
}

/// 记录努力处理器
///
/// **端点:** `POST /schedules/{id}/presence`
pub async fn log_presence_handler(
    State(app_state): State<AppState>,
    Path(schedule_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Logging presence for schedule: {}", schedule_id);

    // 调用服务层
    let updated_schedule = app_state.schedule_service.log_presence(schedule_id).await?;

    log::info!("Presence logged for schedule: {}", schedule_id);

    Ok(success_response(updated_schedule))
}

/// 获取指定日期的日程处理器
///
/// **端点:** `GET /schedules`
pub async fn get_schedules_handler(
    State(app_state): State<AppState>,
    Query(query): Query<GetSchedulesQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if let Some(date) = query.date {
        // 获取特定日期的日程
        log::debug!("Getting schedules for date: {}", date);
        let schedules = app_state
            .schedule_service
            .get_schedules_for_day(date)
            .await?;
        Ok(success_response(schedules))
    } else if let (Some(start_date), Some(end_date)) = (query.start_date, query.end_date) {
        // 获取日期范围内的日程
        log::debug!(
            "Getting schedules for range: {} to {}",
            start_date,
            end_date
        );
        // 这里需要在ScheduleService中添加相应方法，暂时返回空列表
        let schedules: Vec<TaskSchedule> = Vec::new();
        Ok(success_response(schedules))
    } else {
        Err(AppError::validation_error(
            "query",
            "Either 'date' or both 'start_date' and 'end_date' must be provided",
            "INVALID_QUERY_PARAMS",
        ))
    }
}

/// 获取任务的所有日程处理器
///
/// **端点:** `GET /tasks/{id}/schedules`
pub async fn get_task_schedules_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting schedules for task: {}", task_id);

    let schedules = app_state
        .schedule_service
        .get_task_schedules(task_id)
        .await?;

    Ok(success_response(schedules))
}

/// 获取日程统计处理器
///
/// **端点:** `GET /schedules/stats`
pub async fn get_schedule_stats_handler(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting schedule statistics");

    let stats = app_state.schedule_service.get_schedule_statistics().await?;

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
    fn test_schedule_task_payload_serialization() {
        let payload = ScheduleTaskPayload {
            task_id: Uuid::new_v4(),
            target_day: chrono::Utc::now(),
            mode: ScheduleMode::Link,
            source_schedule_id: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("link"));

        let deserialized: ScheduleTaskPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.task_id, payload.task_id);
        assert!(matches!(deserialized.mode, ScheduleMode::Link));
    }

    #[test]
    fn test_schedule_mode_serialization() {
        let link_mode = ScheduleMode::Link;
        let move_mode = ScheduleMode::Move;

        let link_json = serde_json::to_string(&link_mode).unwrap();
        let move_json = serde_json::to_string(&move_mode).unwrap();

        assert_eq!(link_json, "\"link\"");
        assert_eq!(move_json, "\"move\"");

        let link_deserialized: ScheduleMode = serde_json::from_str(&link_json).unwrap();
        let move_deserialized: ScheduleMode = serde_json::from_str(&move_json).unwrap();

        assert!(matches!(link_deserialized, ScheduleMode::Link));
        assert!(matches!(move_deserialized, ScheduleMode::Move));
    }

    #[test]
    fn test_get_schedules_query_parsing() {
        let query_str = "date=2024-01-01T00:00:00Z";
        let query: GetSchedulesQuery = serde_urlencoded::from_str(query_str).unwrap();

        assert!(query.date.is_some());
        assert!(query.start_date.is_none());
        assert!(query.end_date.is_none());
    }
}

/// 时间块相关的HTTP处理器
///
/// 实现所有与时间块相关的API端点处理逻辑
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    error_handler::{created_response, no_content_response, success_response},
    payloads::{CreateTimeBlockPayload, LinkTaskToBlockPayload, UpdateTimeBlockPayload},
};
use crate::common::error::AppError;
use crate::startup::AppState;

/// 查询参数：获取时间块
#[derive(Debug, Deserialize)]
pub struct GetTimeBlocksQuery {
    /// 特定日期
    pub date: Option<DateTime<Utc>>,

    /// 开始日期
    pub start_date: Option<DateTime<Utc>>,

    /// 结束日期
    pub end_date: Option<DateTime<Utc>>,

    /// 任务ID
    pub task_id: Option<Uuid>,

    /// 领域ID
    pub area_id: Option<Uuid>,
}

/// 创建时间块处理器
///
/// **端点:** `POST /time-blocks`
pub async fn create_time_block_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTimeBlockPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Creating time block from {} to {}",
        payload.start_time,
        payload.end_time
    );

    // 转换载荷为服务层数据结构
    let create_data = crate::services::CreateTimeBlockData::from(payload);

    // 调用服务层
    let created_time_block = app_state
        .time_block_service
        .create_time_block(create_data)
        .await?;

    log::info!("Time block created successfully: {}", created_time_block.id);

    Ok(created_response(created_time_block))
}

/// 获取时间块详情处理器
///
/// **端点:** `GET /time-blocks/{id}`
pub async fn get_time_block_handler(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting time block: {}", block_id);

    let time_block = app_state
        .time_block_service
        .get_time_block(block_id)
        .await?
        .ok_or_else(|| AppError::not_found("TimeBlock", block_id.to_string()))?;

    Ok(success_response(time_block))
}

/// 更新时间块处理器
///
/// **端点:** `PUT /time-blocks/{id}`
pub async fn update_time_block_handler(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    Json(payload): Json<UpdateTimeBlockPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Updating time block: {}", block_id);

    // 转换载荷为服务层数据结构
    let update_data = crate::services::UpdateTimeBlockData::from(payload);

    // 调用服务层
    let updated_time_block = app_state
        .time_block_service
        .update_time_block(block_id, update_data)
        .await?;

    log::info!("Time block updated successfully: {}", block_id);

    Ok(success_response(updated_time_block))
}

/// 删除时间块处理器
///
/// **端点:** `DELETE /time-blocks/{id}`
pub async fn delete_time_block_handler(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Deleting time block: {}", block_id);

    // 调用服务层
    app_state
        .time_block_service
        .delete_time_block(block_id)
        .await?;

    log::info!("Time block deleted successfully: {}", block_id);

    Ok(no_content_response())
}

/// 获取时间块列表处理器
///
/// **端点:** `GET /time-blocks`
pub async fn get_time_blocks_handler(
    State(app_state): State<AppState>,
    Query(query): Query<GetTimeBlocksQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    if let Some(date) = query.date {
        // 获取特定日期的时间块
        log::debug!("Getting time blocks for date: {}", date);
        let time_blocks = app_state
            .time_block_service
            .get_time_blocks_for_date(date)
            .await?;
        Ok(success_response(time_blocks))
    } else if let (Some(start_date), Some(end_date)) = (query.start_date, query.end_date) {
        // 获取日期范围内的时间块
        log::debug!(
            "Getting time blocks for range: {} to {}",
            start_date,
            end_date
        );
        let time_blocks = app_state
            .time_block_service
            .get_time_blocks_for_range(start_date, end_date)
            .await?;
        Ok(success_response(time_blocks))
    } else if let Some(task_id) = query.task_id {
        // 获取与任务关联的时间块
        log::debug!("Getting time blocks for task: {}", task_id);
        let time_blocks = app_state
            .time_block_service
            .get_time_blocks_for_task(task_id)
            .await?;
        Ok(success_response(time_blocks))
    } else {
        Err(AppError::validation_error(
            "query",
            "At least one query parameter (date, date range, or task_id) must be provided",
            "INVALID_QUERY_PARAMS",
        ))
    }
}

/// 链接任务到时间块处理器
///
/// **端点:** `POST /time-blocks/{id}/tasks`
pub async fn link_task_to_block_handler(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    Json(payload): Json<LinkTaskToBlockPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Linking task {} to time block {}",
        payload.task_id,
        block_id
    );

    // 调用服务层
    app_state
        .time_block_service
        .link_task_to_block(block_id, payload.task_id)
        .await?;

    log::info!(
        "Task {} linked to time block {} successfully",
        payload.task_id,
        block_id
    );

    Ok(no_content_response())
}

/// 取消任务与时间块关联处理器
///
/// **端点:** `DELETE /time-blocks/{id}/tasks/{task_id}`
pub async fn unlink_task_from_block_handler(
    State(app_state): State<AppState>,
    Path((block_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Unlinking task {} from time block {}", task_id, block_id);

    // 调用服务层
    app_state
        .time_block_service
        .unlink_task_from_block(block_id, task_id)
        .await?;

    log::info!(
        "Task {} unlinked from time block {} successfully",
        task_id,
        block_id
    );

    Ok(no_content_response())
}

/// 检查时间冲突处理器
///
/// **端点:** `GET /time-blocks/conflicts`
#[derive(Debug, Deserialize)]
pub struct CheckConflictQuery {
    /// 开始时间
    pub start_time: DateTime<Utc>,

    /// 结束时间
    pub end_time: DateTime<Utc>,

    /// 排除的时间块ID
    pub exclude_id: Option<Uuid>,
}

pub async fn check_time_conflict_handler(
    State(app_state): State<AppState>,
    Query(query): Query<CheckConflictQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Checking time conflict from {} to {}",
        query.start_time,
        query.end_time
    );

    let has_conflict = app_state
        .time_block_service
        .check_time_conflict(query.start_time, query.end_time, query.exclude_id)
        .await?;

    #[derive(serde::Serialize)]
    struct ConflictResponse {
        has_conflict: bool,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    }

    Ok(success_response(ConflictResponse {
        has_conflict,
        start_time: query.start_time,
        end_time: query.end_time,
    }))
}

/// 查找空闲时间段处理器
///
/// **端点:** `GET /time-blocks/free-slots`
#[derive(Debug, Deserialize)]
pub struct FindFreeSlotsQuery {
    /// 开始时间
    pub start_time: DateTime<Utc>,

    /// 结束时间
    pub end_time: DateTime<Utc>,

    /// 最小时长（分钟）
    pub min_duration_minutes: i32,
}

pub async fn find_free_slots_handler(
    State(app_state): State<AppState>,
    Query(query): Query<FindFreeSlotsQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Finding free slots from {} to {}, min duration: {} minutes",
        query.start_time,
        query.end_time,
        query.min_duration_minutes
    );

    let free_slots = app_state
        .time_block_service
        .find_free_time_slots(query.start_time, query.end_time, query.min_duration_minutes)
        .await?;

    Ok(success_response(free_slots))
}

/// 截断时间块处理器
///
/// **端点:** `POST /time-blocks/{id}/truncate`
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TruncateTimeBlockPayload {
    /// 截断时间点
    pub truncate_at: DateTime<Utc>,
}

pub async fn truncate_time_block_handler(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    Json(payload): Json<TruncateTimeBlockPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Truncating time block {} at {}",
        block_id,
        payload.truncate_at
    );

    let truncated_block = app_state
        .time_block_service
        .truncate_time_block(block_id, payload.truncate_at)
        .await?;

    log::info!("Time block {} truncated successfully", block_id);

    Ok(success_response(truncated_block))
}

/// 扩展时间块处理器
///
/// **端点:** `POST /time-blocks/{id}/extend`
#[derive(Debug, serde::Deserialize)]
pub struct ExtendTimeBlockPayload {
    /// 新的结束时间
    pub new_end_time: DateTime<Utc>,
}

pub async fn extend_time_block_handler(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    Json(payload): Json<ExtendTimeBlockPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Extending time block {} to {}",
        block_id,
        payload.new_end_time
    );

    let extended_block = app_state
        .time_block_service
        .extend_time_block(block_id, payload.new_end_time)
        .await?;

    log::info!("Time block {} extended successfully", block_id);

    Ok(success_response(extended_block))
}

/// 分割时间块处理器
///
/// **端点:** `POST /time-blocks/{id}/split`
#[derive(Debug, serde::Deserialize)]
pub struct SplitTimeBlockPayload {
    /// 分割时间点
    pub split_at: DateTime<Utc>,
}

pub async fn split_time_block_handler(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    Json(payload): Json<SplitTimeBlockPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Splitting time block {} at {}", block_id, payload.split_at);

    let (first_block, second_block) = app_state
        .time_block_service
        .split_time_block(block_id, payload.split_at)
        .await?;

    log::info!("Time block {} split successfully", block_id);

    #[derive(serde::Serialize)]
    struct SplitResponse {
        first_block: crate::core::models::TimeBlock,
        second_block: crate::core::models::TimeBlock,
    }

    Ok(success_response(SplitResponse {
        first_block,
        second_block,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_time_block_payload_serialization() {
        let payload = CreateTimeBlockPayload {
            title: Some("Meeting".to_string()),
            glance_note: None,
            detail_note: None,
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now() + chrono::Duration::hours(1),
            area_id: None,
            task_ids: vec![Uuid::new_v4()],
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Meeting"));

        let deserialized: CreateTimeBlockPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, payload.title);
        assert_eq!(deserialized.task_ids.len(), 1);
    }

    #[test]
    fn test_get_time_blocks_query_parsing() {
        let query_str = "date=2024-01-01T00:00:00Z&task_id=550e8400-e29b-41d4-a716-446655440000";
        let query: GetTimeBlocksQuery = serde_urlencoded::from_str(query_str).unwrap();

        assert!(query.date.is_some());
        assert!(query.task_id.is_some());
    }

    #[test]
    fn test_truncate_payload_serialization() {
        let payload = TruncateTimeBlockPayload {
            truncate_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("truncate_at"));

        let deserialized: TruncateTimeBlockPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.truncate_at, payload.truncate_at);
    }
}

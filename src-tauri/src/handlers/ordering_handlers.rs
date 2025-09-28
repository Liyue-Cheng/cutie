/// 排序相关的HTTP处理器
///
/// 实现所有与任务排序相关的API端点处理逻辑
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error_handler::{no_content_response, success_response},
    payloads::UpdateOrderPayload,
};
use crate::common::error::AppError;
use crate::core::models::{ContextType, Ordering};
use crate::startup::AppState;

/// 查询参数：获取上下文排序
#[derive(Debug, Deserialize)]
pub struct GetContextOrderingQuery {
    /// 上下文类型
    pub context_type: ContextType,

    /// 上下文ID
    pub context_id: String,
}

/// 更新排序处理器
///
/// **端点:** `PUT /ordering`
/// **函数签名:** `pub async fn update_order_handler(State<AppState>, Json<UpdateOrderPayload>) -> Result<StatusCode, AppError>`
/// **预期行为简介:** 更新一个任务在一个特定上下文中的排序位置。
/// **输入规范:**
/// - `schema`: `UpdateOrderPayload`，包含`context_type`, `context_id`, `task_id`, `new_sort_order`。
/// **后置条件:**
/// - `204 No Content`: 排序更新成功。
/// - `422 Unprocessable Entity`: 输入的上下文或排序值无效。
/// **预期副作用:** 调用`OrderingService::update_order`。
pub async fn update_order_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<UpdateOrderPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Updating order for task {} in context {}::{}",
        payload.task_id,
        serde_json::to_string(&payload.context_type).unwrap_or_default(),
        payload.context_id
    );

    // 转换载荷为服务层命令
    let command = crate::services::UpdateOrderCommand::from(payload);
    let task_id = command.task_id;

    // 调用服务层
    app_state.ordering_service.update_order(command).await?;

    log::info!("Order updated successfully for task {}", task_id);

    Ok(no_content_response())
}

/// 获取上下文排序处理器
///
/// **端点:** `GET /ordering`
pub async fn get_context_ordering_handler(
    State(app_state): State<AppState>,
    Query(query): Query<GetContextOrderingQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Getting ordering for context {}::{}",
        serde_json::to_string(&query.context_type).unwrap_or_default(),
        query.context_id
    );

    let orderings = app_state
        .ordering_service
        .get_context_ordering(&query.context_type, &query.context_id)
        .await?;

    Ok(success_response(orderings))
}

/// 获取任务的所有排序记录处理器
///
/// **端点:** `GET /tasks/{id}/ordering`
pub async fn get_task_orderings_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting orderings for task: {}", task_id);

    let orderings = app_state
        .ordering_service
        .get_task_orderings(task_id)
        .await?;

    Ok(success_response(orderings))
}

/// 清理上下文排序处理器
///
/// **端点:** `DELETE /ordering`
pub async fn clear_context_ordering_handler(
    State(app_state): State<AppState>,
    Query(query): Query<GetContextOrderingQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Clearing ordering for context {}::{}",
        serde_json::to_string(&query.context_type).unwrap_or_default(),
        query.context_id
    );

    app_state
        .ordering_service
        .clear_context(&query.context_type, &query.context_id)
        .await?;

    log::info!(
        "Context ordering cleared for {}::{}",
        serde_json::to_string(&query.context_type).unwrap_or_default(),
        query.context_id
    );

    Ok(no_content_response())
}

/// 批量更新排序处理器
///
/// **端点:** `PUT /ordering/batch`
pub async fn batch_update_ordering_handler(
    State(app_state): State<AppState>,
    Json(orderings): Json<Vec<Ordering>>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Batch updating {} orderings", orderings.len());

    app_state
        .ordering_service
        .batch_update_order(orderings)
        .await?;

    log::info!("Batch ordering update completed");

    Ok(no_content_response())
}

/// 计算排序位置处理器
///
/// **端点:** `GET /ordering/calculate`
#[derive(Debug, Deserialize)]
pub struct CalculateSortOrderQuery {
    /// 上下文类型
    pub context_type: ContextType,

    /// 上下文ID
    pub context_id: String,

    /// 前一个排序位置（可选）
    pub prev_sort_order: Option<String>,

    /// 后一个排序位置（可选）
    pub next_sort_order: Option<String>,
}

pub async fn calculate_sort_order_handler(
    State(app_state): State<AppState>,
    Query(query): Query<CalculateSortOrderQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Calculating sort order for context {}::{}",
        serde_json::to_string(&query.context_type).unwrap_or_default(),
        query.context_id
    );

    let sort_order = app_state
        .ordering_service
        .get_sort_order_between(
            &query.context_type,
            &query.context_id,
            query.prev_sort_order.as_deref(),
            query.next_sort_order.as_deref(),
        )
        .await?;

    #[derive(Serialize)]
    struct SortOrderResponse {
        sort_order: String,
    }

    Ok(success_response(SortOrderResponse { sort_order }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ContextType;

    #[test]
    fn test_schedule_task_payload_serialization() {
        let payload = crate::handlers::payloads::ScheduleTaskPayload {
            task_id: Uuid::new_v4(),
            target_day: chrono::Utc::now(),
            mode: crate::handlers::payloads::ScheduleMode::Link,
            source_schedule_id: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("link"));

        let deserialized: crate::handlers::payloads::ScheduleTaskPayload =
            serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.task_id, payload.task_id);
    }

    #[test]
    fn test_update_order_payload_serialization() {
        let payload = UpdateOrderPayload {
            context_type: ContextType::DailyKanban,
            context_id: "1729555200".to_string(),
            task_id: Uuid::new_v4(),
            new_sort_order: "n".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("DAILY_KANBAN"));
        assert!(json.contains("1729555200"));

        let deserialized: UpdateOrderPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.context_type, ContextType::DailyKanban);
        assert_eq!(deserialized.context_id, "1729555200");
    }

    #[test]
    fn test_get_context_ordering_query_parsing() {
        let query_str = "context_type=DAILY_KANBAN&context_id=1729555200";
        let query: GetContextOrderingQuery = serde_urlencoded::from_str(query_str).unwrap();

        assert_eq!(query.context_type, ContextType::DailyKanban);
        assert_eq!(query.context_id, "1729555200");
    }

    #[test]
    fn test_calculate_sort_order_query_parsing() {
        let query_str = "context_type=MISC&context_id=floating&prev_sort_order=a&next_sort_order=c";
        let query: CalculateSortOrderQuery = serde_urlencoded::from_str(query_str).unwrap();

        assert_eq!(query.context_type, ContextType::Misc);
        assert_eq!(query.context_id, "floating");
        assert_eq!(query.prev_sort_order, Some("a".to_string()));
        assert_eq!(query.next_sort_order, Some("c".to_string()));
    }
}

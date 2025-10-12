/// 更新排序 API - 单文件组件
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{task::ContextType, Ordering},
    crate::infra::core::{AppError, AppResult, ValidationError},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `update_order`

## API端点
PUT /api/ordering

## 预期行为简介
更新一个任务在一个特定上下文中的排序位置。使用 UPSERT 语义。

## 输入输出规范
- **前置条件**: 请求体必须包含有效的 `task_id`, `context_type`, `context_id`, `sort_order`。
- **后置条件**: 返回 `204 No Content`。
- **不变量**: context_type 和 context_id 的组合必须唯一标识一个可排序视图。

## 边界情况
- task_id 不存在: 返回 `404 Not Found`。
- context_id 格式无效: 返回 `422 Unprocessable Entity`。
- sort_order 格式无效: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 在 `ordering` 表中 UPSERT 1条记录。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "task_id": "uuid-string",
  "context_type": "DAILY_KANBAN",
  "context_id": "1729555200",
  "sort_order": "0|abc"
}
```
*/

#[derive(Deserialize)]
pub struct UpdateOrderRequest {
    task_id: String,
    context_type: String,
    context_id: String,
    sort_order: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<UpdateOrderRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub struct ValidatedOrderCommand {
        pub task_id: Uuid,
        pub context_type: ContextType,
        pub context_id: String,
        pub sort_order: String,
    }

    pub fn validate_request(
        request: &UpdateOrderRequest,
    ) -> Result<ValidatedOrderCommand, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 1. 验证 task_id
        let task_id = match Uuid::parse_str(&request.task_id) {
            Ok(id) => id,
            Err(_) => {
                errors.push(ValidationError::new(
                    "task_id",
                    "Task ID 格式无效",
                    "INVALID_TASK_ID",
                ));
                Uuid::nil() // 占位符
            }
        };

        // 2. 验证 context_type
        let context_type = match request.context_type.as_str() {
            "DAILY_KANBAN" => ContextType::DailyKanban,
            "PROJECT_LIST" => ContextType::ProjectList,
            "AREA_FILTER" => ContextType::AreaFilter,
            "MISC" => ContextType::Misc,
            _ => {
                errors.push(ValidationError::new(
                    "context_type",
                    "Context type 无效，必须是 DAILY_KANBAN, PROJECT_LIST, AREA_FILTER 或 MISC",
                    "INVALID_CONTEXT_TYPE",
                ));
                ContextType::Misc // 占位符
            }
        };

        // 3. 验证 context_id 格式
        if let Err(e) = Ordering::validate_context_id(&context_type, &request.context_id) {
            errors.push(ValidationError::new("context_id", &e, "INVALID_CONTEXT_ID"));
        }

        // 4. 验证 sort_order 格式（使用 LexoRank 验证）
        if !crate::infra::core::utils::sort_order_utils::is_valid_sort_order(&request.sort_order) {
            errors.push(ValidationError::new(
                "sort_order",
                "Sort order 格式无效，必须是有效的 LexoRank 格式",
                "INVALID_SORT_ORDER",
            ));
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidatedOrderCommand {
            task_id,
            context_type,
            context_id: request.context_id.clone(),
            sort_order: request.sort_order.clone(),
        })
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, request: UpdateOrderRequest) -> AppResult<()> {
        // 1. 验证请求
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 2. 验证任务存在
        let task_exists = database::task_exists_in_tx(&mut tx, validated.task_id).await?;
        if !task_exists {
            return Err(AppError::not_found("Task", validated.task_id.to_string()));
        }

        // 3. 核心操作：UPSERT 排序记录
        let now = app_state.clock().now_utc();
        database::upsert_ordering_in_tx(
            &mut tx,
            validated.task_id,
            validated.context_type,
            validated.context_id,
            validated.sort_order,
            now,
        )
        .await?;

        // 4. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use chrono::{DateTime, Utc};

    pub async fn task_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE id = ? AND deleted_at IS NULL")
                .bind(task_id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
                })?;

        Ok(count > 0)
    }

    pub async fn upsert_ordering_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        context_type: ContextType,
        context_id: String,
        sort_order: String,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        sqlx::query(
            r#"
            INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(context_type, context_id, task_id)
            DO UPDATE SET sort_order = excluded.sort_order, updated_at = excluded.updated_at
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(context_type_str)
        .bind(&context_id)
        .bind(task_id.to_string())
        .bind(&sort_order)
        .bind(updated_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}

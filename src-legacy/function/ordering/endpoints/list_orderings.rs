/// 查询排序列表 API - 单文件组件
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::{
    entities::Ordering,
    shared::core::{AppError, AppResult},
    shared::http::responses::ApiResponse,
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `list_orderings`

## API端点
GET /api/ordering

## 预期行为简介
根据 context_type 和 context_id 查询该上下文中的所有排序记录。

## 输入输出规范
- **前置条件**: 必须提供 context_type 和 context_id 查询参数
- **后置条件**: 返回该上下文中的所有排序记录，按 sort_order 升序排列
- **不变量**: 返回的记录都属于同一个 context

## 边界情况
- context 不存在: 返回空数组
- context_type 无效: 返回 422

## 预期副作用
- 无数据库修改，纯查询操作

## 查询参数
- context_type: DAILY_KANBAN | PROJECT_LIST | AREA_FILTER | MISC
- context_id: string

## 响应体
```json
{
  "data": [
    {
      "id": "uuid",
      "context_type": "DAILY_KANBAN",
      "context_id": "1729555200",
      "task_id": "task-uuid",
      "sort_order": "0|abc",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```
*/

#[derive(Deserialize)]
pub struct ListQuery {
    context_type: String,
    context_id: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(State(app_state): State<AppState>, Query(query): Query<ListQuery>) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(orderings) => Json(ApiResponse::success(orderings)).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub struct ValidatedRequest {
        pub context_type: String,
        pub context_id: String,
    }

    pub fn validate_request(
        request: &ListQuery,
    ) -> Result<ValidatedRequest, Vec<crate::shared::core::ValidationError>> {
        use crate::shared::core::ValidationError;

        let mut errors = Vec::new();

        // 验证 context_type
        if !["DAILY_KANBAN", "PROJECT_LIST", "AREA_FILTER", "MISC"]
            .contains(&request.context_type.as_str())
        {
            errors.push(ValidationError::new(
                "context_type",
                &format!("Invalid context_type: {}", request.context_type),
                "INVALID_CONTEXT_TYPE",
            ));
        }

        // 验证 context_id
        if request.context_id.trim().is_empty() {
            errors.push(ValidationError::new(
                "context_id",
                "context_id cannot be empty",
                "EMPTY_CONTEXT_ID",
            ));
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidatedRequest {
            context_type: request.context_type.clone(),
            context_id: request.context_id.clone(),
        })
    }
}

// ==================== 业务逻辑层 (Business Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: ListQuery,
    ) -> AppResult<Vec<Ordering>> {
        // 1. 验证请求
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        // 2. 从数据库查询
        let orderings = database::find_orderings_by_context(
            app_state.db_pool(),
            &validated.context_type,
            &validated.context_id,
        )
        .await?;

        Ok(orderings)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::OrderingRow;
    use sqlx::SqlitePool;

    pub async fn find_orderings_by_context(
        pool: &SqlitePool,
        context_type: &str,
        context_id: &str,
    ) -> AppResult<Vec<Ordering>> {
        let rows = sqlx::query_as::<_, OrderingRow>(
            r#"
            SELECT id, context_type, context_id, task_id, sort_order, updated_at
            FROM ordering
            WHERE context_type = ? AND context_id = ?
            ORDER BY sort_order ASC
            "#,
        )
        .bind(context_type)
        .bind(context_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 转换 OrderingRow 为 Ordering
        let orderings: Result<Vec<Ordering>, String> =
            rows.into_iter().map(|row| row.try_into()).collect();

        orderings.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::QueryError(format!(
                "Failed to convert OrderingRow: {}",
                e
            )))
        })
    }
}

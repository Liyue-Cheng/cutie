/// 计算排序位置 API - 单文件组件
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    shared::core::{AppError, AppResult},
    shared::http::responses::ApiResponse,
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `calculate_sort_order`

## API端点
GET /api/ordering/calculate

## 预期行为简介
根据前后位置计算一个新的 sort_order 值。

## 输入输出规范
- **前置条件**: 
  - 必须提供 context_type 和 context_id
  - prev_sort_order 和 next_sort_order 至少有一个为空（边界情况）或都提供（中间插入）
- **后置条件**: 返回计算出的 sort_order
- **不变量**: sort_order 字符串按字典序排列

## 边界情况
- prev 和 next 都为空: 生成初始 sort_order
- 只有 prev: 在末尾追加
- 只有 next: 在开头插入
- prev 和 next 都有: 在中间插入

## 预期副作用
- 无数据库修改，纯计算操作

## 查询参数
- context_type: DAILY_KANBAN | PROJECT_LIST | AREA_FILTER | MISC
- context_id: string
- prev_sort_order: optional string
- next_sort_order: optional string

## 响应体
```json
{
  "data": {
    "sort_order": "0|abc123"
  }
}
```
*/

#[derive(Deserialize)]
pub struct CalculateQuery {
    context_type: String,
    context_id: String,
    prev_sort_order: Option<String>,
    next_sort_order: Option<String>,
}

#[derive(Serialize)]
pub struct CalculateResponse {
    sort_order: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<CalculateQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(response) => Json(ApiResponse::success(response)).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub struct ValidatedRequest {
        pub _context_type: String,
        pub _context_id: String,
        pub prev_sort_order: Option<String>,
        pub next_sort_order: Option<String>,
    }

    pub fn validate_request(
        request: &CalculateQuery,
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
            _context_type: request.context_type.clone(),
            _context_id: request.context_id.clone(),
            prev_sort_order: request.prev_sort_order.clone(),
            next_sort_order: request.next_sort_order.clone(),
        })
    }
}

// ==================== 业务逻辑层 (Business Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        _app_state: &AppState,
        request: CalculateQuery,
    ) -> AppResult<CalculateResponse> {
        // 1. 验证请求
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        // 2. 计算 sort_order
        let sort_order = calculate_sort_order(
            validated.prev_sort_order.as_deref(),
            validated.next_sort_order.as_deref(),
        )?;

        Ok(CalculateResponse { sort_order })
    }

    fn calculate_sort_order(prev: Option<&str>, next: Option<&str>) -> AppResult<String> {
        use crate::shared::core::utils::sort_order_utils;
        use crate::shared::core::ValidationError;

        match (prev, next) {
            (None, None) => {
                // 空列表，生成初始值
                Ok(sort_order_utils::generate_initial_sort_order())
            }
            (Some(prev), None) => {
                // 插入到末尾
                sort_order_utils::get_rank_after(prev).map_err(|e| {
                    AppError::ValidationFailed(vec![ValidationError::new(
                        "sort_order",
                        &format!("Failed to calculate sort order after '{}': {}", prev, e),
                        "INVALID_SORT_ORDER",
                    )])
                })
            }
            (None, Some(next)) => {
                // 插入到开头
                sort_order_utils::get_rank_before(next).map_err(|e| {
                    AppError::ValidationFailed(vec![ValidationError::new(
                        "sort_order",
                        &format!("Failed to calculate sort order before '{}': {}", next, e),
                        "INVALID_SORT_ORDER",
                    )])
                })
            }
            (Some(prev), Some(next)) => {
                // 插入到中间
                sort_order_utils::get_mid_lexo_rank(prev, next).map_err(|e| {
                    AppError::ValidationFailed(vec![ValidationError::new(
                        "sort_order",
                        &format!(
                            "Failed to calculate sort order between '{}' and '{}': {}",
                            prev, next, e
                        ),
                        "INVALID_SORT_ORDER",
                    )])
                })
            }
        }
    }
}

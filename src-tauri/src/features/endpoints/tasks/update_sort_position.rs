/// 更新任务在视图中的排序位置 - 单文件组件
///
/// PATCH /api/tasks/:id/sort-position
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    features::shared::repositories::TaskSortRepository,
    infra::LexoRankService,
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `update_sort_position`

## 1. 端点签名

PATCH /api/tasks/{id}/sort-position

## 2. 预期行为简介

在指定视图中更新任务的 LexoRank 排序位置，仅影响该任务自身的 `sort_positions` 字段，
不更改其他任务的排序信息，避免并发拖拽互相覆盖。

## 3. 输入输出规范

### 3.1 请求

```json
{
  "view_context": "daily::2025-10-01",
  "prev_task_id": "uuid | null",
  "next_task_id": "uuid | null"
}
```

- `view_context` 必填，对应前端视图标识
- `prev_task_id` 可选，表示当前任务移动后前一个任务的ID（null表示移动到开头）
- `next_task_id` 可选，表示当前任务移动后后一个任务的ID（null表示移动到末尾）

### 3.2 响应

```json
{
  "task_id": "uuid",
  "view_context": "daily::2025-10-01",
  "new_rank": "0|m00000:",
  "updated_at": "2025-11-22T12:34:56Z"
}
```

## 4. 验证规则

- `view_context` 不能为空
- `prev_task_id` 与 `next_task_id` 可以同时为 null，但至少有一个不为空时需要确保存在
- 当提供 `prev_task_id` 或 `next_task_id` 时，对应任务必须存在且在该视图中已有 rank

## 5. 业务逻辑概述

1. 校验请求参数
2. 根据 `prev_task_id`/`next_task_id` 查询相邻任务的 rank
3. 调用 `LexoRankService::generate_between` 计算新的 rank
4. 更新当前任务 `sort_positions` 中对应视图的 rank
5. 返回新的 rank 及更新时间
*/

// ==================== 请求 / 响应定义 ====================
#[derive(Debug, Deserialize)]
pub struct UpdateSortPositionRequest {
    pub view_context: String,
    pub prev_task_id: Option<Uuid>,
    pub next_task_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct UpdateSortPositionResponse {
    pub task_id: Uuid,
    pub view_context: String,
    pub new_rank: String,
    pub updated_at: String,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    Path(task_id): Path<Uuid>,
    State(app_state): State<AppState>,
    Json(request): Json<UpdateSortPositionRequest>,
) -> Response {
    match logic::execute(&app_state, task_id, request).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        request: UpdateSortPositionRequest,
    ) -> AppResult<UpdateSortPositionResponse> {
        validation::validate(&request)?;

        let pool = app_state.db_pool();
        let view_context = request.view_context.trim();

        // 1. 获取相邻任务的 rank
        let prev_rank = if let Some(prev_id) = request.prev_task_id {
            Some(fetch_rank_or_fail(pool, prev_id, view_context).await?)
        } else {
            None
        };

        let next_rank = if let Some(next_id) = request.next_task_id {
            Some(fetch_rank_or_fail(pool, next_id, view_context).await?)
        } else {
            None
        };

        // 2. 计算新的 rank
        let new_rank =
            LexoRankService::generate_between(prev_rank.as_deref(), next_rank.as_deref())?;

        // 3. 更新任务
        let now = app_state.clock().now_utc();
        TaskSortRepository::update_task_rank(pool, task_id, view_context, &new_rank, now).await?;

        Ok(UpdateSortPositionResponse {
            task_id,
            view_context: view_context.to_string(),
            new_rank,
            updated_at: now.to_rfc3339(),
        })
    }

    async fn fetch_rank_or_fail(
        pool: &sqlx::SqlitePool,
        task_id: Uuid,
        view_context: &str,
    ) -> AppResult<String> {
        match TaskSortRepository::get_task_rank(pool, task_id, view_context).await? {
            Some(rank) => Ok(rank),
            None => Err(AppError::validation_error(
                "neighbor_task",
                format!("Task {} has no rank in view {}", task_id, view_context),
                "NEIGHBOR_TASK_MISSING_RANK",
            )),
        }
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate(request: &UpdateSortPositionRequest) -> AppResult<()> {
        if request.view_context.trim().is_empty() {
            return Err(AppError::validation_error(
                "view_context",
                "view_context cannot be empty",
                "VIEW_CONTEXT_REQUIRED",
            ));
        }

        if request.prev_task_id.is_none() && request.next_task_id.is_none() {
            // 允许两者都为空，表示移动到空列表或保持原位
            return Ok(());
        }

        Ok(())
    }
}

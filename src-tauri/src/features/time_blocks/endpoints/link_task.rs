/// 将任务链接到时间块 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::TimeBlockViewDto,
    features::{
        shared::TransactionHelper,
        tasks::shared::{
            assemblers::TimeBlockAssembler,
            repositories::{TaskRepository, TaskTimeBlockLinkRepository},
        },
        time_blocks::shared::repositories::TimeBlockRepository,
    },
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 链接任务到时间块的请求
#[derive(Debug, Deserialize)]
pub struct LinkTaskRequest {
    pub task_id: Uuid,
}

/// 链接任务到时间块的响应
#[derive(Debug, Serialize)]
pub struct LinkTaskResponse {
    pub time_block: TimeBlockViewDto,
}

// ==================== 文档层 ====================
/*
CABC for `link_task_to_time_block`

## 1. 端点签名 (Endpoint Signature)

POST /api/time-blocks/{block_id}/link-task

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要将任务拖动到日历上已有的时间片上，
> 将任务链接到该时间片，而不是创建新的时间片。

### 2.2. 核心业务逻辑 (Core Business Logic)

将指定任务链接到指定时间块。一个任务可以链接到多个时间块，
一个时间块也可以链接多个任务。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**路径参数:**
- `block_id`: UUID - 时间块ID

**请求体 (Request Body):** `application/json`

```json
{
  "task_id": "uuid"
}
```

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

```json
{
  "time_block": {
    "id": "uuid",
    "start_time": "2025-10-07T10:00:00Z",
    "end_time": "2025-10-07T11:00:00Z",
    "title": "任务标题",
    "glance_note": "简要笔记",
    "area": {...},
    "linked_tasks": [...]
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "时间块不存在"
}
```

或

```json
{
  "error_code": "NOT_FOUND",
  "message": "任务不存在"
}
```

## 4. 验证规则 (Validation Rules)

- `block_id`: 必须是有效的 UUID，且时间块必须存在
- `task_id`: 必须是有效的 UUID，且任务必须存在

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  验证请求体。
2.  获取写入许可。
3.  启动数据库事务。
4.  验证时间块存在。
5.  验证任务存在。
6.  检查链接是否已存在。
7.  如果不存在，创建链接。
8.  查询更新后的完整时间块数据（包括所有链接的任务）。
9.  写入领域事件到 outbox。
10. 提交事务。
11. 返回更新后的时间块。

## 6. 边界情况 (Edge Cases)

- **时间块不存在:** 返回 `404` 错误。
- **任务不存在:** 返回 `404` 错误。
- **链接已存在:** 不报错，视为幂等操作，返回成功。
- **幂等性:** 相同参数重复调用，结果一致。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`INSERT`:** 0-1 条记录到 `task_time_block_links` 表（如果链接不存在）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可。
- **SSE 事件:**
    - 发送 `time_block.task_linked` 事件，包含：
        - 更新后的时间块（`TimeBlockViewDto`）
        - 链接的任务ID
- **日志记录:**
    - 成功时，记录链接信息。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<LinkTaskRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, block_id, request, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        block_id: Uuid,
        request: LinkTaskRequest,
        correlation_id: Option<String>,
    ) -> AppResult<LinkTaskResponse> {
        let task_id = request.task_id;

        // ✅ 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        // 1. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 2. 验证时间块存在（find_by_id_in_tx 找不到会返回错误）
        let _block = TimeBlockRepository::find_by_id_in_tx(&mut tx, block_id).await?;

        // 3. 验证任务存在（find_by_id_in_tx 找不到会返回错误）
        let _task = TaskRepository::find_by_id_in_tx(&mut tx, task_id).await?;

        // 4. 检查链接是否已存在
        let exists = database::link_exists_in_tx(&mut tx, task_id, block_id).await?;

        // 5. 如果不存在，创建链接
        if !exists {
            TaskTimeBlockLinkRepository::link_in_tx(&mut tx, task_id, block_id).await?;
            tracing::info!("Linked task {} to time block {}", task_id, block_id);
        } else {
            tracing::info!(
                "Link already exists: task {} -> block {}",
                task_id,
                block_id
            );
        }

        // 6. 查询更新后的完整时间块数据
        let time_block = TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &[block_id]).await?;
        let time_block = time_block
            .into_iter()
            .next()
            .ok_or_else(|| AppError::not_found("TimeBlock", block_id.to_string()))?;

        // 7. 写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "time_block": time_block,
            "task_id": task_id,
        });

        let mut event = DomainEvent::new(
            "time_block.task_linked",
            "TimeBlock",
            block_id.to_string(),
            payload,
        );

        if let Some(cid) = correlation_id {
            event = event.with_correlation_id(cid);
        }

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        // 8. 提交事务
        TransactionHelper::commit(tx).await?;

        // 9. 返回结果
        Ok(LinkTaskResponse { time_block })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    /// 检查链接是否存在
    pub async fn link_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        block_id: Uuid,
    ) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_time_block_links
            WHERE task_id = ? AND time_block_id = ?
        "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(count > 0)
    }
}

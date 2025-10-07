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
            assemblers::TimeBlockAssembler, repositories::TaskTimeBlockLinkRepository,
        },
        time_blocks::shared::repositories::TimeBlockRepository,
    },
    shared::{
        core::AppResult,
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

POST /api/time-blocks/{block_id}/link

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要将任务拖动到日历上已有的时间块上，
> 系统能自动将任务链接到该时间块，这样我可以复用已有的时间块。

### 2.2. 核心业务逻辑 (Core Business Logic)

将指定的任务链接到指定的时间块。

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
    "title": "string | null",
    "glance_note": "string | null",
    "area": {...} | null,
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

- `block_id`: 必须存在于数据库中
- `task_id`: 必须存在于数据库中
- 链接允许重复（幂等性）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  验证请求参数。
2.  开启数据库事务。
3.  检查时间块是否存在。
4.  检查任务是否存在。
5.  创建任务到时间块的链接（如果已存在则忽略）。
6.  查询更新后的完整时间块数据。
7.  写入领域事件到 outbox。
8.  提交事务。
9.  返回更新后的时间块。

## 6. 边界情况 (Edge Cases)

- **时间块不存在:** 返回 `404` 错误。
- **任务不存在:** 返回 `404` 错误。
- **链接已存在:** 忽略，返回成功（幂等性）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`INSERT`:** 1条记录到 `task_time_block_links` 表（如果不存在）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `time_block.task_linked` 事件。
- **日志记录:**
    - 成功时，记录链接的任务ID和时间块ID。
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
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 检查时间块是否存在（find_by_id_in_tx 找不到时会返回错误）
        let time_block = TimeBlockRepository::find_by_id_in_tx(&mut tx, block_id).await?;

        // 检查任务是否存在（find_by_id_in_tx 找不到时会返回错误）
        use crate::features::tasks::shared::repositories::TaskRepository;
        let _task = TaskRepository::find_by_id_in_tx(&mut tx, request.task_id).await?;

        // 创建链接（幂等性：如果已存在则忽略）
        TaskTimeBlockLinkRepository::link_in_tx(&mut tx, request.task_id, block_id).await?;

        tracing::info!("Linked task {} to time block {}", request.task_id, block_id);

        // 提交事务
        TransactionHelper::commit(tx).await?;

        // 查询完整的时间块数据（包含新链接的任务）
        let time_block_view =
            TimeBlockAssembler::assemble_view(&time_block, app_state.db_pool()).await?;

        // 写入领域事件到 outbox（独立事务）
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let mut outbox_tx = TransactionHelper::begin(app_state.db_pool()).await?;
        {
            let payload = serde_json::json!({
                "time_block": time_block_view,
                "linked_task_id": request.task_id,
            });
            let mut event = DomainEvent::new(
                "time_block.task_linked",
                "TimeBlock",
                block_id.to_string(),
                payload,
            )
            .with_aggregate_version(now.timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
        }
        TransactionHelper::commit(outbox_tx).await?;

        Ok(LinkTaskResponse {
            time_block: time_block_view,
        })
    }
}

/// 删除时间块 API - 单文件组件
///
/// 软删除时间块，不影响任务的排期状态
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{TimeBlockSideEffects, TimeBlockViewDto},
    features::{
        shared::repositories::TaskTimeBlockLinkRepository,
        shared::repositories::TimeBlockRepository,
    },
    infra::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `delete_time_block`

## 1. 端点签名 (Endpoint Signature)

DELETE /api/time-blocks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我决定取消或移除一个日历上的时间块时，
> 我希望系统能够：
> 1. 删除这个时间块（软删除，标记为已删除）
> 2. 断开时间块与任务的链接关系
> 3. 保留任务的排期状态（任务仍然在"已排期"列表中，只是没有具体的时间段）

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除时间块（设置 `is_deleted = true`），不物理删除数据。
关键业务规则：
1. 删除时间块**不影响**任务的排期状态（`task_schedules` 记录保留）
2. 删除 `task_time_block_links` 表中的链接关系
3. 任务仍然保持"已排期"状态，只是失去了具体的执行时间段

Cutie 的设计哲学：
- 时间块代表"具体的执行时间"
- 任务排期代表"是否被安排到某一天"
- 删除时间块≠取消任务排期
- 任务仍在 Planned 列表中，只是没有具体时间段

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 时间块ID

### 3.2. 响应 (Responses)

**204 No Content:**

删除成功，无返回内容。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "TimeBlock not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `id`:
    - **必须**是有效的 UUID 格式。
    - 对应的时间块**必须**存在于数据库中（不论是否已删除）。
    - 违反时返回错误码：`NOT_FOUND`
- **幂等性**:
    - 如果时间块已被删除，仍然返回 `204 No Content`（幂等操作）。
    - 重复删除不会产生错误。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  启动数据库事务（`app_state.db_pool().begin()`）。
2.  调用 `TimeBlockRepository::exists_in_tx` 检查时间块是否存在：
    - 如果时间块不存在，返回 404 错误
3.  调用 `TimeBlockRepository::soft_delete_in_tx` 软删除时间块：
    - 设置 `is_deleted = true`
    - 不删除物理记录（保留审计和历史数据）
4.  调用 `TaskTimeBlockLinkRepository::delete_all_for_block_in_tx` 删除所有任务链接：
    - 断开时间块与任务的关联关系
    - **注意**：不删除 `task_schedules` 记录
5.  提交数据库事务。
6.  返回 `204 No Content`。

## 6. 边界情况 (Edge Cases)

- **时间块不存在:** 返回 `404` 错误。
- **时间块已被删除:** 返回 `204 No Content`（幂等性）。
- **时间块关联多个任务:** 删除所有链接关系，不影响任务的排期状态。
- **时间块没有关联任务:** 正常删除时间块。
- **并发删除:** 事务隔离保证只执行一次实际删除，其他请求返回成功（幂等）。
- **任务状态保持:** 删除时间块后，任务仍然保持"已排期"状态（如果有 `task_schedules` 记录）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，检查时间块是否存在。
    - **`UPDATE`:** 1条记录在 `time_blocks` 表（设置 `is_deleted = true`）。
    - **`DELETE`:** 0-N 条记录在 `task_time_block_links` 表（删除所有链接）。
    - **注意**：**不修改** `task_schedules` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **任务状态影响:**
    - 关联的任务失去具体的执行时间段。
    - 任务的排期状态（`schedule_status`）保持不变（如果有其他日程或时间块）。
    - 任务的 `schedules` 列表保持不变。
- **日志记录:**
    - 成功时，可能记录删除操作（如有配置）。
    - 失败时，记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*
*/

// ==================== 响应结构体 ====================
/// 删除时间块的响应
/// ✅ HTTP 响应和 SSE 事件使用相同的数据结构
///
/// 注意：删除时间块不返回时间块本身（已删除），而是返回受影响的任务列表
#[derive(Debug, Serialize)]
pub struct DeleteTimeBlockResponse {
    /// 删除的时间块 ID
    pub time_block_id: uuid::Uuid,

    /// 副作用：受影响的任务
    #[serde(skip_serializing_if = "TimeBlockSideEffects::is_empty")]
    pub side_effects: TimeBlockSideEffects,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, block_id, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        block_id: Uuid,
        correlation_id: Option<String>,
    ) -> AppResult<DeleteTimeBlockResponse> {
        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 1. 检查时间块是否存在（✅ 使用共享 Repository）
        let block_exists = TimeBlockRepository::exists_in_tx(&mut tx, block_id).await?;
        if !block_exists {
            return Err(AppError::not_found("TimeBlock", block_id.to_string()));
        }

        // 1.5 查询受影响的任务ID（在删除链接之前）
        let affected_task_ids =
            TaskTimeBlockLinkRepository::get_task_ids_for_block_in_tx(&mut tx, block_id).await?;

        // 2. 软删除时间块（✅ 使用共享 Repository）
        TimeBlockRepository::soft_delete_in_tx(&mut tx, block_id).await?;

        // 3. 删除任务链接（但保留 task_schedules！）（✅ 使用共享 Repository）
        TaskTimeBlockLinkRepository::delete_all_for_block_in_tx(&mut tx, block_id).await?;

        // 4. 组装受影响任务的完整数据（在事务内）
        let mut affected_tasks = Vec::new();
        if !affected_task_ids.is_empty() {
            use crate::features::shared::{repositories::TaskRepository, TaskAssembler};

            for task_id in &affected_task_ids {
                if let Some(task) = TaskRepository::find_by_id_in_tx(&mut tx, *task_id).await? {
                    let mut task_card = TaskAssembler::task_to_card_basic(&task);

                    // ✅ 填充 schedules 字段（必须在 SSE 之前）
                    task_card.schedules =
                        TaskAssembler::assemble_schedules_in_tx(&mut tx, *task_id).await?;
                    // schedule_status 已删除 - 前端根据 schedules 字段实时计算

                    // 填充 recurrence_expiry_behavior（使用 pool 查询，task_recurrences 表不在事务内修改）
                    TaskAssembler::fill_recurrence_expiry_behavior(&mut task_card, app_state.db_pool()).await?;

                    affected_tasks.push(task_card);
                }
            }
        }

        // 5. 构建统一的响应结构
        // ✅ HTTP 响应和 SSE 事件使用相同的数据结构
        let response = DeleteTimeBlockResponse {
            time_block_id: block_id,
            side_effects: TimeBlockSideEffects {
                updated_tasks: if affected_tasks.is_empty() {
                    None
                } else {
                    Some(affected_tasks.clone())
                },
                updated_time_blocks: None,
            },
        };

        // 6. 写入事件到 outbox（在事务内）
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            // ✅ 使用统一的响应结构作为事件载荷
            let payload = serde_json::to_value(&response)?;

            let mut event = DomainEvent::new(
                "time_blocks.deleted",
                "TimeBlock",
                block_id.to_string(),
                payload,
            )
            .with_aggregate_version(app_state.clock().now_utc().timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 7. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(response)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TimeBlockRepository::exists_in_tx
// - TimeBlockRepository::soft_delete_in_tx
// - TaskTimeBlockLinkRepository::delete_all_for_block_in_tx

/// 清空回收站 API - 单文件组件
///
/// 批量彻底删除回收站中的任务
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

use crate::{
    entities::TimeBlock,
    features::shared::{
        repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
        TransactionHelper,
    },
    infra::{
        core::AppResult,
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 清空回收站请求
#[derive(Debug, Deserialize)]
pub struct EmptyTrashRequest {
    /// 只删除超过指定天数的任务，默认 30 天
    #[serde(default = "default_older_than_days")]
    pub older_than_days: i64,
    /// 最大删除数量，默认 100（防止长事务）
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_older_than_days() -> i64 {
    30
}

fn default_limit() -> i64 {
    100
}

/// 清空回收站响应
#[derive(Debug, Serialize)]
pub struct EmptyTrashResponse {
    pub deleted_count: usize,
}

// ==================== 文档层 ====================
/*
CABC for `empty_trash`

## 1. 端点签名 (Endpoint Signature)

POST /api/trash/empty

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想清空回收站中的旧任务，
> 以便释放存储空间并保持数据整洁。

### 2.2. 核心业务逻辑 (Core Business Logic)

批量物理删除回收站中的任务：
1. 查询符合条件的任务（deleted_at IS NOT NULL AND deleted_at < now - older_than_days）
2. 对每个任务执行物理删除流程
3. 清理关联数据和孤儿时间块

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**Body:**
```json
{
  "older_than_days": 30,  // 0 = 删除所有任务，>0 = 只删除超过指定天数的任务
  "limit": 100
}
```

### 3.2. 响应 (Responses)

**200 OK:**

```json
{
  "deleted_count": 10
}
```

## 4. 验证规则 (Validation Rules)

- `older_than_days`: 必须 >= 0
- `limit`: 必须 > 0，最大 100

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 解析请求参数
2. 查询回收站中的任务
3. 根据 older_than_days 过滤任务：
   - 如果 older_than_days = 0：删除所有任务
   - 如果 older_than_days > 0：只删除 deleted_at < (now - older_than_days) 的任务
4. 对每个任务：
   - 查询关联的时间块
   - 删除链接和日程
   - 物理删除任务
   - 检查并删除孤儿时间块
5. 发送 SSE 事件
6. 返回删除数量

## 6. 边界情况 (Edge Cases)

- **回收站为空:** 返回 deleted_count = 0
- **没有符合条件的任务:** 返回 deleted_count = 0

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询回收站中的任务
    - **`DELETE`:** 0-N 条记录从 `tasks` 表
    - **`DELETE`:** 0-N 条记录从 `task_time_block_links` 表
    - **`DELETE`:** 0-N 条记录从 `task_schedules` 表
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表
- **SSE 事件:**
    - 发送 `trash.emptied` 事件

## 8. 契约 (Contract)

### Pre-conditions:
- 无

### Post-conditions:
- 符合条件的任务被物理删除
- 返回删除数量

### Invariants:
- 只删除 deleted_at IS NOT NULL 的任务
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<EmptyTrashRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, request, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;
    use chrono::Duration;

    pub async fn execute(
        app_state: &AppState,
        request: EmptyTrashRequest,
        correlation_id: Option<String>,
    ) -> AppResult<EmptyTrashResponse> {
        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 验证参数
        let older_than_days = request.older_than_days.max(0);
        let limit = request.limit.min(100).max(1);

        // 计算截止时间
        let now = app_state.clock().now_utc();

        // 查询符合条件的任务
        let tasks = TaskRepository::find_deleted_tasks(app_state.db_pool(), limit, 0).await?;

        tracing::info!(
            "[empty_trash] Found {} tasks in trash, older_than_days={}, limit={}",
            tasks.len(),
            older_than_days,
            limit
        );

        // 过滤出符合时间条件的任务
        let tasks_to_delete: Vec<_> = if older_than_days == 0 {
            // older_than_days = 0 表示删除所有任务，不进行时间过滤
            tracing::info!(
                "[empty_trash] older_than_days=0, deleting all {} tasks",
                tasks.len()
            );
            tasks
        } else {
            let cutoff_time = now - Duration::days(older_than_days);
            tracing::info!(
                "[empty_trash] Filtering tasks: now={}, cutoff_time={}, older_than_days={}",
                now,
                cutoff_time,
                older_than_days
            );
            tasks
                .into_iter()
                .filter(|task| {
                    if let Some(deleted_at) = task.deleted_at {
                        let should_delete = deleted_at < cutoff_time;
                        tracing::debug!(
                            "[empty_trash] Task {}: deleted_at={}, cutoff_time={}, should_delete={}",
                            task.id,
                            deleted_at,
                            cutoff_time,
                            should_delete
                        );
                        should_delete
                    } else {
                        false
                    }
                })
                .collect()
        };

        tracing::info!(
            "[empty_trash] After filtering: {} tasks to delete",
            tasks_to_delete.len()
        );

        let deleted_count = tasks_to_delete.len();
        let mut deleted_task_ids = Vec::new();

        // 收集所有被删除的时间块
        let mut all_deleted_time_block_ids = Vec::new();

        // 对每个任务执行删除
        for task in tasks_to_delete {
            let task_id = task.id;
            deleted_task_ids.push(task_id);

            let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

            // 查询关联的时间块
            let linked_blocks =
                TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task_id)
                    .await?;

            // 删除链接和日程
            TaskTimeBlockLinkRepository::delete_all_for_task_in_tx(&mut tx, task_id).await?;
            TaskScheduleRepository::delete_all_in_tx(&mut tx, task_id).await?;

            // 物理删除任务
            TaskRepository::permanently_delete_in_tx(&mut tx, task_id).await?;

            // 检查并删除孤儿时间块，收集被删除的时间块 ID
            for block in linked_blocks {
                let should_delete = should_delete_orphan_block(&block, &mut tx).await?;
                if should_delete {
                    all_deleted_time_block_ids.push(block.id);
                    use crate::features::shared::repositories::TimeBlockRepository;
                    TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
                }
            }

            TransactionHelper::commit(tx).await?;
        }

        // 查询所有被删除的时间块的完整数据（用于事件）
        use crate::features::shared::assemblers::TimeBlockAssembler;
        let mut final_tx = TransactionHelper::begin(app_state.db_pool()).await?;
        let deleted_time_blocks = TimeBlockAssembler::assemble_for_event_in_tx(
            &mut final_tx,
            &all_deleted_time_block_ids,
        )
        .await?;
        TransactionHelper::commit(final_tx).await?;

        // 发送 SSE 事件
        if !deleted_task_ids.is_empty() {
            let mut outbox_tx = TransactionHelper::begin(app_state.db_pool()).await?;

            use crate::infra::events::{
                models::DomainEvent,
                outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
            };
            let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

            let payload = serde_json::json!({
                "deleted_task_ids": deleted_task_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
                "deleted_count": deleted_count,
                "deleted_time_blocks": deleted_time_blocks, // ✅ 包含被删除的时间块
            });

            let mut event =
                DomainEvent::new("trash.emptied", "trash", "trash".to_string(), payload)
                    .with_aggregate_version(now.timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
            TransactionHelper::commit(outbox_tx).await?;
        }

        Ok(EmptyTrashResponse { deleted_count })
    }

    /// 判断是否应该删除孤儿时间块
    async fn should_delete_orphan_block(
        block: &TimeBlock,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<bool> {
        let remaining_tasks =
            TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx(tx, block.id).await?;
        if remaining_tasks > 0 {
            return Ok(false);
        }

        if let Some(source_info) = &block.source_info {
            if source_info.source_type == "native::from_task" {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_deleted_tasks, permanently_delete_in_tx
// - TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx, delete_all_for_task_in_tx, count_remaining_tasks_in_block_in_tx
// - TaskScheduleRepository::delete_all_in_tx
// - TimeBlockRepository::soft_delete_in_tx

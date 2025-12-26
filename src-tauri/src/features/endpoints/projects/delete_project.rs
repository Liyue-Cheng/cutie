/// 删除 Project API - 单文件组件
///
/// 软删除项目，并级联处理关联的 tasks 和 time_blocks
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TaskCardDto, TimeBlock, TimeBlockViewDto},
    features::shared::{
        assemblers::TimeBlockAssembler,
        repositories::{
            ProjectRepository, TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository,
            TimeBlockRepository,
        },
        TaskAssembler, TransactionHelper,
    },
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 响应结构 ====================

/// 删除项目的副作用
#[derive(Debug, Serialize, Clone, Default)]
pub struct DeleteProjectSideEffects {
    /// 被删除的任务列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_tasks: Option<Vec<TaskCardDto>>,

    /// 被删除的时间块列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_time_blocks: Option<Vec<TimeBlockViewDto>>,
}

impl DeleteProjectSideEffects {
    pub fn is_empty(&self) -> bool {
        self.deleted_tasks.is_none() && self.deleted_time_blocks.is_none()
    }
}

/// 删除项目的响应
/// ✅ HTTP 响应和 SSE 事件使用相同的数据结构
#[derive(Debug, Serialize)]
pub struct DeleteProjectResponse {
    /// 被删除的项目 ID
    pub id: String,

    /// 副作用：被删除的关联资源
    #[serde(skip_serializing_if = "DeleteProjectSideEffects::is_empty")]
    pub side_effects: DeleteProjectSideEffects,
}

// ==================== 文档层 ====================
/*
CABC for `delete_project`

## 1. 端点签名 (Endpoint Signature)

DELETE /api/projects/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要删除一个不再使用的项目，
> 系统应该同时删除所有关联的任务和孤儿时间块，
> 并通过 SSE 通知前端删除这些资源。

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除项目及其关联数据：
1. 软删除项目（设置 `is_deleted = true`）
2. 软删除所有关联的 sections
3. 对每个关联的 task：
   - 软删除任务
   - 删除任务的所有 links 和 schedules
   - 检查并删除孤儿时间块
4. 返回被删除的 tasks 和 time_blocks 列表

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 项目ID

### 3.2. 响应 (Responses)

**200 OK:**

```json
{
  "id": "uuid",
  "side_effects": {
    "deleted_tasks": [...],
    "deleted_time_blocks": [...]
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Project not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `project_id`:
    - **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。
    - **必须**存在于数据库中。
    - **必须**未被删除（`is_deleted = false`）。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 从路径参数中提取 `project_id`。
2. 获取写入许可。
3. 启动数据库事务。
4. 检查项目是否存在。
5. 查询项目下的所有未删除任务。
6. 对每个任务：
   a. 查询任务链接的所有时间块
   b. 删除任务的所有 links 和 schedules
   c. 检查并标记需要删除的孤儿时间块
7. 组装所有被删除任务的 TaskCardDto（在软删除之前）
8. 查询所有被删除时间块的完整数据
9. 软删除项目、sections 和 tasks（使用 ProjectRepository::soft_delete）
10. 软删除所有孤儿时间块
11. 写入 Event Outbox：发送 `project.deleted` 事件（包含完整副作用）
12. 提交数据库事务。
13. 返回响应（包含被删除的 tasks 和 time_blocks）。

## 6. 边界情况 (Edge Cases)

- **项目不存在:** 返回 `404` 错误。
- **项目已删除:** 幂等，返回 `404` 错误。
- **项目无关联任务:** 只删除项目和 sections，side_effects 为空。
- **时间块还有其他任务:** 不删除时间块。
- **时间块是手动创建的:** 不删除时间块。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询项目、任务、时间块
    - **`UPDATE`:** 软删除项目、sections、tasks、time_blocks
    - **`DELETE`:** 删除 task_time_block_links、task_schedules
    - **`INSERT`:** 1条记录到 `event_outbox` 表
    - **(事务):** 所有数据库写操作包含在一个数据库事务内
- **SSE 事件:**
    - **事件类型:** `project.deleted`
    - **载荷:** `{ "id": "uuid", "side_effects": { "deleted_tasks": [...], "deleted_time_blocks": [...] } }`

## 8. 契约 (Contract)

- 软删除保留数据用于恢复或审计
- 级联软删除所有关联的 sections 和 tasks
- 清理孤儿时间块（从任务拖拽创建的）
- HTTP 响应和 SSE 事件使用相同的数据结构
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(project_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, project_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        project_id: Uuid,
    ) -> AppResult<DeleteProjectResponse> {
        // 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 检查项目是否存在
        let exists = ProjectRepository::find_by_id(&mut *tx, project_id).await?;
        if exists.is_none() {
            return Err(AppError::not_found("Project", project_id.to_string()));
        }

        let now = app_state.clock().now_utc();

        // 1. 查询项目下的所有未删除任务
        let tasks = TaskRepository::find_by_project_in_tx(&mut tx, project_id).await?;

        // 2. 收集所有需要删除的时间块和任务卡片
        let mut all_blocks_to_delete: Vec<TimeBlock> = Vec::new();
        let mut task_cards: Vec<TaskCardDto> = Vec::new();

        for task in &tasks {
            // 2.1 组装 TaskCardDto（在软删除之前，手动设置 deleted_at）
            let mut task_card = TaskAssembler::task_to_card_basic(task);
            task_card.deleted_at = Some(now);
            task_card.is_deleted = true;

            // 2.2 填充 recurrence_expiry_behavior
            TaskAssembler::fill_recurrence_expiry_behavior(&mut task_card, app_state.db_pool())
                .await?;

            task_cards.push(task_card);

            // 2.3 查询任务链接的所有时间块
            let linked_blocks =
                TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task.id)
                    .await?;

            // 2.4 删除任务的所有 links 和 schedules
            TaskTimeBlockLinkRepository::delete_all_for_task_in_tx(&mut tx, task.id).await?;
            TaskScheduleRepository::delete_all_in_tx(&mut tx, task.id).await?;

            // 2.5 检查并标记需要删除的孤儿时间块
            for block in linked_blocks {
                let should_delete = should_delete_orphan_block(&block, &mut tx).await?;
                if should_delete {
                    // 避免重复添加
                    if !all_blocks_to_delete.iter().any(|b| b.id == block.id) {
                        tracing::info!(
                            "Will delete orphan time block {} (source_type={:?}) after deleting project {}",
                            block.id,
                            block.source_info.as_ref().map(|s| &s.source_type),
                            project_id
                        );
                        all_blocks_to_delete.push(block);
                    }
                }
            }
        }

        // 3. 查询被删除时间块的完整数据（用于 SSE 事件）
        let deleted_block_ids: Vec<Uuid> = all_blocks_to_delete.iter().map(|b| b.id).collect();
        let deleted_time_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &deleted_block_ids).await?;

        // 4. 软删除项目及关联数据（project、sections、tasks）
        ProjectRepository::soft_delete(&mut tx, project_id, now).await?;

        // 5. 软删除所有孤儿时间块
        for block in &all_blocks_to_delete {
            TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
        }

        // 6. 构建响应
        let side_effects = DeleteProjectSideEffects {
            deleted_tasks: if task_cards.is_empty() {
                None
            } else {
                Some(task_cards)
            },
            deleted_time_blocks: if deleted_time_blocks.is_empty() {
                None
            } else {
                Some(deleted_time_blocks)
            },
        };

        let response = DeleteProjectResponse {
            id: project_id.to_string(),
            side_effects: side_effects.clone(),
        };

        // 7. 写入 Event Outbox
        events::write_project_deleted_event(app_state, &mut tx, &response, now).await?;

        // 8. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(response)
    }

    /// 判断是否应该删除孤儿时间块
    ///
    /// 删除规则：
    /// 1. 时间块没有其他任务链接（孤儿）
    /// 2. 时间块的 source_type == "native::from_task"（从任务拖拽创建）
    ///
    /// 保留规则：
    /// - native::manual：手动创建的时间块
    /// - external::*：外部导入的时间块
    /// - 无 source_info：旧数据（向后兼容，默认保留）
    async fn should_delete_orphan_block(
        block: &TimeBlock,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<bool> {
        // 1. 检查时间块是否还有其他任务
        let remaining_tasks =
            TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx(tx, block.id).await?;
        if remaining_tasks > 0 {
            return Ok(false); // 还有其他任务，不删除
        }

        // 2. 基于 source_info 判断是否应删除
        if let Some(source_info) = &block.source_info {
            if source_info.source_type == "native::from_task" {
                return Ok(true); // 孤儿 + 从任务创建 = 删除
            }
        }

        // 3. 默认保留（手动创建、外部导入、或无来源信息的旧数据）
        Ok(false)
    }
}

// ==================== 事件层 ====================
mod events {
    use super::*;
    use crate::infra::events::{
        models::DomainEvent,
        outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
    };
    use chrono::{DateTime, Utc};

    pub async fn write_project_deleted_event(
        app_state: &AppState,
        tx: &mut Transaction<'_, Sqlite>,
        response: &DeleteProjectResponse,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        // ✅ 使用与 HTTP 响应相同的数据结构
        let payload = serde_json::to_value(response)?;

        let event = DomainEvent::new(
            "project.deleted",
            "project",
            response.id.clone(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        outbox_repo.append_in_tx(tx, &event).await?;

        Ok(())
    }
}

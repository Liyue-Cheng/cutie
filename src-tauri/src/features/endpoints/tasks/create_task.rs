/// 创建任务 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use std::collections::HashMap;

use crate::{
    entities::{CreateTaskRequest, Task, TaskCardDto},
    features::shared::{repositories::TaskRepository, TaskAssembler},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_task`

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要快速创建一个新任务并放入 Staging 区，
> 以便我能立即捕捉我的想法，而不需要复杂的步骤。

### 2.2. 核心业务逻辑 (Core Business Logic)

在数据库中创建一个新的 `Task` 实体，默认进入 Staging 区（未安排到具体日期）。
新任务的初始状态为未完成（`completed_at = NULL`），无日程安排记录。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "title": "string (required, 1-255 chars)",
  "glance_note": "string | null (optional)",
  "detail_note": "string | null (optional)",
  "estimated_duration": "number | null (optional, 分钟数，0-10080)",
  "area_id": "string (UUID) | null (optional)",
  "project_id": "string (UUID) | null (optional)",
  "section_id": "string (UUID) | null (optional)",
  "due_date": "string (YYYY-MM-DD) | null (optional)",
  "due_date_type": "'soft' | 'hard' | null (optional)",
  "subtasks": "array | null (optional, 最多50个)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto`

```json
{
  "id": "uuid",
  "title": "string",
  "glance_note": "string | null",
  "schedule_status": "staging",
  "is_completed": false,
  "area": { "id": "uuid", "name": "string", "color": "string" } | null,
  "project_id": null,
  "subtasks": [...] | null,
  "schedules": null,
  "due_date": {...} | null,
  "has_detail_note": boolean
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "title", "code": "TITLE_EMPTY", "message": "任务标题不能为空" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `title`:
    - **必须**存在。
    - **必须**为非空字符串 (trim后)。
    - 长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_EMPTY` 或 `TITLE_TOO_LONG`
- `estimated_duration`:
    - 如果提供，**必须**是大于等于 0 的整数。
    - 如果提供，**必须**小于等于 10080 (7天 = 7*24*60 分钟)。
    - 违反时返回错误码：`DURATION_NEGATIVE` 或 `DURATION_TOO_LONG`
- `subtasks`:
    - 如果提供，数组长度**必须**小于等于 50。
    - 违反时返回错误码：`TOO_MANY_SUBTASKS`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_create_request` 验证请求体。
2.  获取写入许可（`app_state.acquire_write_permit()`），确保写操作串行执行。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  通过 `IdGenerator` 生成新的 `task_id`（UUID）。
5.  通过 `Clock` 服务获取当前时间 `now`。
6.  构造 `Task` 领域实体对象：
    - 设置 `id`, `title`, `glance_note`, `detail_note` 等字段
    - 设置 `completed_at = None`（未完成）
    - 设置 `created_at = now`, `updated_at = now`
    - 设置 `deleted_at IS NULL`
7.  调用 `TaskRepository::insert_in_tx` 持久化任务到 `tasks` 表。
8.  提交数据库事务（`TransactionHelper::commit`）。
9.  调用 `TaskAssembler::task_to_card_basic` 组装 `TaskCardDto`。
10. 设置 `task_card.schedule_status = Staging`（因为新任务无日程）。
11. 填充 `task_card.schedules` 字段（应为 `None`，因为无日程）。
12. 返回 `201 Created` 和组装好的 `TaskCardDto`。

## 6. 边界情况 (Edge Cases)

- **`title` 为空或全空格:** 返回 `422` 错误，错误码 `TITLE_EMPTY`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **`estimated_duration` 为负数:** 返回 `422` 错误，错误码 `DURATION_NEGATIVE`。
- **`estimated_duration` 超过 10080:** 返回 `422` 错误，错误码 `DURATION_TOO_LONG`。
- **`subtasks` 超过 50 个:** 返回 `422` 错误，错误码 `TOO_MANY_SUBTASKS`。
- **`area_id` 不存在:** 当前实现中正常返回（area 字段为 null），未来可能需要验证。
- **并发创建:** 使用写入许可确保写操作串行执行，避免并发问题。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`INSERT`:** 1条记录到 `tasks` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **日志记录:**
    - 成功时，以 `INFO` 级别记录 "Task created successfully" 及任务ID（如有）。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*
*/

// ==================== HTTP 处理器 ====================
/// 创建任务的 HTTP 处理器
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Response {
    tracing::debug!(
        target: "ENDPOINT:TASKS:create_task",
        title = %request.title,
        area_id = ?request.area_id,
        project_id = ?request.project_id,
        section_id = ?request.section_id,
        has_subtasks = request.subtasks.is_some(),
        "Creating task"
    );

    match logic::execute(&app_state, request).await {
        Ok(task_card) => {
            tracing::info!(
                target: "ENDPOINT:TASKS:create_task",
                task_id = %task_card.id,
                title = %task_card.title,
                "Task created successfully"
            );
            created_response(task_card).into_response()
        }
        Err(err) => {
            tracing::error!(
                target: "ENDPOINT:TASKS:create_task",
                error = %err,
                "Failed to create task"
            );
            err.into_response()
        }
    }
}

// ==================== 验证层 ====================
// ✅ 已迁移到共享验证器：TaskValidator
// - 使用 TaskValidator::validate_create_request 统一验证逻辑

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::{repositories::ProjectRepository, TaskValidator, TransactionHelper};

    pub async fn execute(
        app_state: &AppState,
        request: CreateTaskRequest,
    ) -> AppResult<TaskCardDto> {
        // 1. 验证请求（✅ 使用共享 TaskValidator）
        TaskValidator::validate_create_request(&request)?;

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        tracing::debug!(
            target: "SERVICE:TASKS:create_task",
            "Transaction started"
        );

        // 3. 生成 UUID 和时间戳
        let task_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        tracing::trace!(
            target: "SERVICE:TASKS:create_task",
            task_id = %task_id,
            "Generated task ID"
        );

        // 🔥 如果指定了 project_id 但没有指定 area_id，从项目继承 area_id
        let area_id = if request.area_id.is_none() && request.project_id.is_some() {
            if let Some(project) = ProjectRepository::find_by_id(app_state.db_pool(), request.project_id.unwrap()).await? {
                tracing::debug!(
                    target: "SERVICE:TASKS:create_task",
                    project_id = %request.project_id.unwrap(),
                    inherited_area_id = ?project.area_id,
                    "Inheriting area_id from project"
                );
                project.area_id
            } else {
                None
            }
        } else {
            request.area_id
        };

        // 4. 创建任务实体
        let task = Task {
            id: task_id,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            estimated_duration: request.estimated_duration,
            subtasks: request.subtasks.clone(),
            sort_positions: HashMap::new(),
            project_id: request.project_id,
            section_id: request.section_id,
            area_id,
            due_date: request.due_date,
            due_date_type: request.due_date_type,
            completed_at: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_id: None,
            recurrence_original_date: None,
        };

        // 5. 插入任务到数据库（✅ 使用共享 Repository）
        TaskRepository::insert_in_tx(&mut tx, &task).await?;

        tracing::debug!(
            target: "SERVICE:TASKS:create_task",
            task_id = %task_id,
            "Task inserted into database"
        );

        // 6. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        tracing::debug!(
            target: "SERVICE:TASKS:create_task",
            task_id = %task_id,
            "Transaction committed"
        );

        // 7. 组装返回的 TaskCardDto（✅ area_id 已由 TaskAssembler 填充）
        let mut task_card = TaskAssembler::task_to_card_basic(&task);
        // schedule_status 已删除 - 前端根据 schedules 字段实时计算

        // 8. ✅ 填充 schedules 字段（新任务应该是 None）
        task_card.schedules =
            TaskAssembler::assemble_schedules(app_state.db_pool(), task.id).await?;

        // 9. ✅ 异步 AI 自动分类（不阻塞返回）
        // 条件：未指定 area_id 且不是从模板创建
        if task.area_id.is_none() && task.source_info.is_none() {
            let task_id = task.id;
            let task_title = task.title.clone();
            let pool = app_state.db_pool().clone();

            tracing::debug!(
                target: "SERVICE:TASKS:create_task",
                task_id = %task_id,
                "Spawning AI classification task"
            );

            // 异步任务：不阻塞当前请求
            tokio::spawn(async move {
                use crate::features::shared::AiClassificationService;

                if let Err(e) =
                    AiClassificationService::classify_and_update_task(task_id, &task_title, &pool)
                        .await
                {
                    tracing::error!(
                        target: "SERVICE:TASKS:auto_classify",
                        task_id = %task_id,
                        error = %e,
                        "Failed to auto-classify task"
                    );
                }
            });
        }

        Ok(task_card)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已迁移到共享 Repository：
// - TaskRepository::insert_in_tx

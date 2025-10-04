/// 重新打开任务 API - 单文件组件
///
/// 将已完成的任务重新打开，使其回到未完成状态
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::TaskCardDto,
    features::tasks::shared::{
        repositories::TaskRepository, TaskAssembler, TaskScheduleRepository,
    },
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 重新打开任务的响应
#[derive(Debug, Serialize)]
pub struct ReopenTaskResponse {
    pub task: TaskCardDto,
}

// ==================== 文档层 ====================
/*
CABC for `reopen_task`

## API端点
DELETE /api/tasks/{id}/completion

## 预期行为简介
将已完成的任务重新打开，使其回到未完成状态。这是 complete_task 的逆操作。

## Cutie 业务逻辑
1. 验证任务存在且已完成
2. 将任务的 completed_at 设置为 NULL
3. 更新任务的 updated_at 时间戳
4. 返回更新后的任务信息

## 输入输出规范
- **输入**:
  - Path 参数: task_id (UUID)

- **输出**:
  - 200 OK: 返回重新打开的任务信息
    ```json
    {
      "success": true,
      "data": {
        "task": {
          "id": "uuid",
          "title": "任务标题",
          "is_completed": false,
          ...
        }
      }
    }
    ```

- **前置条件**:
  - task_id 必须存在
  - 任务必须处于已完成状态 (completed_at IS NOT NULL)

- **后置条件**:
  - 任务的 completed_at 字段被设置为 NULL
  - 任务的 updated_at 字段被更新为当前时间
  - 任务回到未完成状态

- **不变量**:
  - 任务的其他属性（标题、描述、area等）保持不变
  - 任务的历史日程记录不受影响

## 边界情况
- **任务不存在**: 返回 404 Not Found
- **任务未完成**: 返回 409 Conflict，提示任务尚未完成
- **任务已删除**: 返回 404 Not Found（is_deleted = true 的任务不可见）

## 预期副作用
- **数据库写入**: 更新 tasks 表中对应记录的 completed_at 和 updated_at 字段
- **日程状态**: 不影响已有的日程记录（outcome 保持不变）
- **时间块**: 不影响已删除或截断的时间块（不恢复）

## 请求/响应示例

### 成功场景
**请求:**
```http
DELETE /api/tasks/550e8400-e29b-41d4-a716-446655440000/completion
```

**响应:**
```json
{
  "success": true,
  "data": {
    "task": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "完成项目报告",
      "glance_note": "需要包含数据分析部分",
      "is_completed": false,
      "completed_at": null,
      "schedule_status": "scheduled",  // 如果任务有日程记录则为 "scheduled"，否则为 "staging"
      "area": {
        "id": "area-123",
        "name": "工作",
        "color": "#4a90e2"
      },
      ...
    }
  },
  "message": null
}
```

### 错误场景 - 任务未完成
**请求:**
```http
DELETE /api/tasks/550e8400-e29b-41d4-a716-446655440000/completion
```

**响应:**
```json
{
  "success": false,
  "data": null,
  "message": "任务尚未完成"
}
```

### 错误场景 - 任务不存在
**请求:**
```http
DELETE /api/tasks/00000000-0000-0000-0000-000000000000/completion
```

**响应:**
```json
{
  "success": false,
  "data": null,
  "message": "Task with id 00000000-0000-0000-0000-000000000000 not found"
}
```

## 数据库操作
- **查询**: SELECT FROM tasks WHERE id = ? AND is_deleted = false
- **更新**: UPDATE tasks SET completed_at = NULL, updated_at = ? WHERE id = ?

## 事务管理
- 所有数据库操作在单个事务中执行
- 失败时自动回滚

## 幂等性
- 对已经未完成的任务调用 reopen 会返回 409 错误
- 不具有幂等性（不同于 complete 操作）

## 安全性
- 无需额外权限验证（V1.0 单用户应用）
- UUID 参数自动验证

## 性能考虑
- 单次数据库查询 + 单次更新
- 使用事务保证一致性
- 查询使用主键索引（高效）
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let handler_start = std::time::Instant::now();
    tracing::info!("[PERF] reopen_task HANDLER_START for task_id={}", task_id);

    let correlation_id = extract_correlation_id(&headers);

    let logic_start = std::time::Instant::now();
    let result = logic::execute(&app_state, task_id, correlation_id).await;
    tracing::info!(
        "[PERF] reopen_task LOGIC took {:.3}ms",
        logic_start.elapsed().as_secs_f64() * 1000.0
    );

    let response_start = std::time::Instant::now();
    let response = match result {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    };
    tracing::info!(
        "[PERF] reopen_task RESPONSE_BUILD took {:.3}ms",
        response_start.elapsed().as_secs_f64() * 1000.0
    );

    tracing::info!(
        "[PERF] reopen_task HANDLER_TOTAL took {:.3}ms",
        handler_start.elapsed().as_secs_f64() * 1000.0
    );

    response
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        _correlation_id: Option<String>,
    ) -> AppResult<ReopenTaskResponse> {
        let start_time = std::time::Instant::now();
        tracing::info!("[PERF] reopen_task START for task_id={}", task_id);

        let now = app_state.clock().now_utc();

        // ⏱️ 1. 取连接（✅ 使用 TransactionHelper）
        let acquire_start = std::time::Instant::now();
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
        tracing::info!(
            "[PERF] reopen_task ACQUIRE_CONNECTION took {:.3}ms",
            acquire_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 2. 查找任务（✅ 使用共享 Repository）
        let find_task_start = std::time::Instant::now();
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        tracing::info!(
            "[PERF] reopen_task FIND_TASK took {:.3}ms",
            find_task_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 3. 检查是否未完成
        let check_start = std::time::Instant::now();
        if task.completed_at.is_none() {
            return Err(AppError::conflict("任务尚未完成"));
        }
        tracing::info!(
            "[PERF] reopen_task CHECK_STATUS took {:.3}ms",
            check_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 4. 重新打开任务（✅ 使用共享 Repository）
        let update_start = std::time::Instant::now();
        TaskRepository::set_reopened_in_tx(&mut tx, task_id, now).await?;
        tracing::info!(
            "[PERF] reopen_task UPDATE_TASK took {:.3}ms",
            update_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 5. 提交事务（✅ 使用 TransactionHelper）
        let commit_start = std::time::Instant::now();
        TransactionHelper::commit(tx).await?;
        tracing::info!(
            "[PERF] reopen_task COMMIT took {:.3}ms",
            commit_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 6. 重新查询并组装返回数据（✅ 使用共享 Repository）
        let refetch_start = std::time::Instant::now();
        let updated_task = TaskRepository::find_by_id(app_state.db_pool(), task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        tracing::info!(
            "[PERF] reopen_task REFETCH_TASK took {:.3}ms",
            refetch_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 7. 组装响应（✅ area_id 已由 TaskAssembler 填充）
        let assemble_start = std::time::Instant::now();
        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // ✅ 修复：正确判断 schedule_status（✅ 使用共享 Repository）
        // 如果任务有任何 schedule 记录，状态就是 scheduled，否则是 staging
        let has_schedule =
            TaskScheduleRepository::has_any_schedule(app_state.db_pool(), task_id).await?;
        task_card.schedule_status = if has_schedule {
            crate::entities::ScheduleStatus::Scheduled
        } else {
            crate::entities::ScheduleStatus::Staging
        };

        // ✅ 填充 schedules 字段（事务已提交，使用 pool 查询）
        task_card.schedules =
            TaskAssembler::assemble_schedules(app_state.db_pool(), task_id).await?;

        tracing::info!(
            "[PERF] reopen_task ASSEMBLE_RESPONSE took {:.3}ms",
            assemble_start.elapsed().as_secs_f64() * 1000.0
        );

        tracing::info!(
            "[PERF] reopen_task TOTAL took {:.3}ms",
            start_time.elapsed().as_secs_f64() * 1000.0
        );

        Ok(ReopenTaskResponse { task: task_card })
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx, find_by_id, set_reopened_in_tx
// - TaskScheduleRepository::has_any_schedule

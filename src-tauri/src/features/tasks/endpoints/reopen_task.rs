/// 重新打开任务 API - 单文件组件
///
/// 将已完成的任务重新打开，使其回到未完成状态
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::TaskCardDto,
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
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
      "schedule_status": "staging",
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
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<ReopenTaskResponse> {
        let now = app_state.clock().now_utc();

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 查找任务
        let task = database::find_task_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 检查是否未完成
        if !task.is_completed() {
            return Err(AppError::conflict("任务尚未完成"));
        }

        // 3. 重新打开任务（设置 completed_at 为 NULL）
        database::set_task_reopened_in_tx(&mut tx, task_id, now).await?;

        // 4. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 5. 重新查询并组装返回数据
        let updated_task = database::find_task(app_state.db_pool(), task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let task_card = TaskAssembler::task_to_card_basic(&updated_task);

        Ok(ReopenTaskResponse { task: task_card })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    pub async fn find_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<crate::entities::Task>> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, estimated_duration, 
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks 
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, TaskRow>(query)
            .bind(task_id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(r) => {
                let task = crate::entities::Task::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    pub async fn find_task(
        pool: &sqlx::SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Option<crate::entities::Task>> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, estimated_duration, 
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks 
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, TaskRow>(query)
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(r) => {
                let task = crate::entities::Task::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    /// 重新打开任务：将 completed_at 设置为 NULL，更新 updated_at
    pub async fn set_task_reopened_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        updated_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let query = "UPDATE tasks SET completed_at = NULL, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(updated_at.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }
}

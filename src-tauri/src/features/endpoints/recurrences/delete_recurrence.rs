/// 删除循环规则 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `delete_recurrence`

## 1. 端点签名
DELETE /api/recurrences/:id

## 2. 预期行为简介
删除循环规则（软删除，标记为不激活）

## 3. 输入输出规范

### 3.1 请求 (Request)
无请求体

### 3.2 响应 (Responses)
**204 No Content:**
删除成功，无响应体

**404 Not Found:**
循环规则不存在

## 4. 业务逻辑详解
1. 开启事务
2. 标记循环规则为不激活
3. 提交事务
4. 返回 204

## 5. 预期副作用
- UPDATE: task_recurrences 表 (设置 is_active = false)
- SSE 事件: recurrence.deleted
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    features::{shared::TaskRecurrenceRepository, shared::TransactionHelper},
    infra::core::AppResult,
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(recurrence_id): Path<Uuid>,
) -> Response {
    match logic::execute(&app_state, recurrence_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::repositories::TaskRepository;

    pub async fn execute(app_state: &AppState, recurrence_id: Uuid) -> AppResult<()> {
        // 1. 获取时间
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 🔥 先查询所有未完成的任务实例（在删除链接表之前）
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Finding all uncompleted instances of recurrence {}",
            recurrence_id
        );

        let uncompleted_task_ids = find_all_uncompleted_instances(&mut tx, recurrence_id).await?;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Found {} uncompleted instances to delete",
            uncompleted_task_ids.len()
        );

        // 4. 🔥 删除所有链接记录（现在可以安全删除了，因为已经获取了任务ID）
        delete_all_recurrence_links(&mut tx, recurrence_id).await?;

        // 5. 🔥 清除所有任务的循环字段（包括已完成的）
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Clearing recurrence fields for all tasks of recurrence {}",
            recurrence_id
        );

        clear_all_recurrence_fields(&mut tx, recurrence_id, now).await?;

        // 6. 🔥 软删除所有未完成的任务实例
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Soft deleting {} uncompleted task instances",
            uncompleted_task_ids.len()
        );

        for task_id in uncompleted_task_ids {
            TaskRepository::soft_delete_in_tx(&mut tx, task_id, now).await?;
        }

        // 7. 标记循环规则为不激活
        TaskRecurrenceRepository::deactivate_in_tx(&mut tx, recurrence_id, now).await?;

        // 8. 提交事务
        TransactionHelper::commit(tx).await?;

        // 9. (可选) 发送 SSE 事件
        // TODO: 实现 SSE 事件

        Ok(())
    }

    /// 查询所有未完成的任务实例（在删除链接表之前调用）
    async fn find_all_uncompleted_instances(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<Vec<Uuid>> {
        let query = r#"
            SELECT trl.task_id
            FROM task_recurrence_links trl
            JOIN tasks t ON t.id = trl.task_id
            WHERE trl.recurrence_id = ?
              AND t.completed_at IS NULL
              AND t.deleted_at IS NULL
        "#;

        let task_id_strs: Vec<String> = sqlx::query_scalar(query)
            .bind(recurrence_id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        // 解析 UUID
        let task_ids: Vec<Uuid> = task_id_strs
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

        Ok(task_ids)
    }

    /// 删除所有循环链接记录
    async fn delete_all_recurrence_links(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<()> {
        let delete_links_query = r#"
            DELETE FROM task_recurrence_links
            WHERE recurrence_id = ?
        "#;

        let deleted_links = sqlx::query(delete_links_query)
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Deleted {} recurrence links",
            deleted_links.rows_affected()
        );

        Ok(())
    }

    /// 清除所有任务的循环字段（包括已完成的）
    async fn clear_all_recurrence_fields(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        let clear_all_query = r#"
            UPDATE tasks
            SET recurrence_id = NULL,
                recurrence_original_date = NULL,
                updated_at = ?
            WHERE recurrence_id = ?
              AND deleted_at IS NULL
        "#;

        let result = sqlx::query(clear_all_query)
            .bind(now.to_rfc3339())
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Cleared recurrence fields for {} tasks",
            result.rows_affected()
        );

        Ok(())
    }
}

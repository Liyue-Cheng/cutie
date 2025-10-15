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
        let today =
            crate::infra::core::utils::time_utils::format_date_yyyy_mm_dd(&now.date_naive());

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 🔥 删除所有未来的未完成任务实例，并清除其循环参数
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Deleting recurrence {} and cleaning up future instances...",
            recurrence_id
        );

        cleanup_all_future_instances(&mut tx, recurrence_id, &today, now).await?;

        // 4. 标记为不激活
        TaskRecurrenceRepository::deactivate_in_tx(&mut tx, recurrence_id, now).await?;

        // 5. 提交事务
        TransactionHelper::commit(tx).await?;

        // 6. (可选) 发送 SSE 事件
        // TODO: 实现 SSE 事件

        Ok(())
    }

    /// 清理所有未来的未完成任务实例，并清除其循环参数
    async fn cleanup_all_future_instances(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        today: &str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        // 1. 查询所有未来的未完成任务实例
        let query = r#"
            SELECT trl.task_id, trl.instance_date
            FROM task_recurrence_links trl
            JOIN tasks t ON t.id = trl.task_id
            WHERE trl.recurrence_id = ?
              AND trl.instance_date >= ?
              AND t.completed_at IS NULL
              AND t.deleted_at IS NULL
        "#;

        #[derive(sqlx::FromRow)]
        struct TaskInstance {
            task_id: String,
            instance_date: String,
        }

        let instances: Vec<TaskInstance> = sqlx::query_as(query)
            .bind(recurrence_id.to_string())
            .bind(today)
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Found {} future uncompleted instances to clean",
            instances.len()
        );

        // 2. 对每个实例：清除循环参数并软删除
        for instance in instances {
            let task_id = Uuid::parse_str(&instance.task_id).map_err(|e| {
                crate::infra::core::AppError::ValidationFailed(vec![
                    crate::infra::core::ValidationError::new(
                        "task_id".to_string(),
                        format!("Invalid UUID: {}", e),
                        "INVALID_UUID".to_string(),
                    ),
                ])
            })?;

            tracing::info!(
                "🔄 [DELETE_RECURRENCE] Cleaning task instance: {} on {}",
                task_id,
                instance.instance_date
            );

            // 清除循环参数
            let clear_params_query = r#"
                UPDATE tasks
                SET recurrence_id = NULL,
                    recurrence_original_date = NULL,
                    updated_at = ?
                WHERE id = ?
            "#;

            sqlx::query(clear_params_query)
                .bind(now)
                .bind(task_id.to_string())
                .execute(&mut **tx)
                .await
                .map_err(|e| {
                    crate::infra::core::AppError::DatabaseError(
                        crate::infra::core::DbError::ConnectionError(e),
                    )
                })?;

            // 软删除任务
            TaskRepository::soft_delete_in_tx(tx, task_id, now).await?;
        }

        // 3. 删除所有链接记录（包括已完成的）
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
}

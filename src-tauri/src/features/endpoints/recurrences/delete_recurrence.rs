/// 删除循环规则 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `delete_recurrence`

## 1. 端点签名
DELETE /api/recurrences/:id

## 2. 预期行为简介
硬删除循环规则及其关联的模板

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
2. 获取循环规则信息（包括 template_id）
3. 查询所有未完成的任务实例
4. 删除任务链接记录（task_recurrence_links）
5. 清除任务的循环字段
6. 软删除未完成的任务实例
7. 清理孤儿时间片（从任务创建的、无其他任务关联的）
8. 硬删除循环规则（task_recurrences）
9. 硬删除关联的模板（templates）
10. 提交事务
11. 返回 204

## 5. 预期副作用
- DELETE: task_recurrence_links 表
- UPDATE: tasks 表 (清除 recurrence_id 和 recurrence_original_date)
- UPDATE: tasks 表 (软删除未完成任务)
- UPDATE: time_blocks 表 (软删除孤儿时间块)
- DELETE: task_recurrences 表 (硬删除)
- DELETE: templates 表 (硬删除)
- SSE 事件: recurrence.deleted, template.deleted
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::TimeBlock,
    features::shared::{
        repositories::{TaskScheduleRepository, TaskTimeBlockLinkRepository, TimeBlockRepository},
        TransactionHelper,
    },
    infra::core::AppResult,
    startup::AppState,
};
use sqlx::{Sqlite, Transaction};

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

        // 3. 🔥 先获取循环规则信息（包括 template_id）
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Fetching recurrence rule {}",
            recurrence_id
        );

        let recurrence_info = get_recurrence_info(&mut tx, recurrence_id).await?;
        let template_id = recurrence_info.template_id;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Recurrence rule found, template_id: {}",
            template_id
        );

        // 4. 🔥 查询所有未完成的任务实例（在删除链接表之前）
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Finding all uncompleted instances of recurrence {}",
            recurrence_id
        );

        let uncompleted_task_ids = find_all_uncompleted_instances(&mut tx, recurrence_id).await?;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Found {} uncompleted instances to delete",
            uncompleted_task_ids.len()
        );

        // 5. 🔥 删除所有链接记录（现在可以安全删除了，因为已经获取了任务ID）
        delete_all_recurrence_links(&mut tx, recurrence_id).await?;

        // 5. 🔥 清除所有任务的循环字段（包括已完成的）
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Clearing recurrence fields for all tasks of recurrence {}",
            recurrence_id
        );

        clear_all_recurrence_fields(&mut tx, recurrence_id, now).await?;

        // 6. 🔥 收集所有待删除任务链接的时间块，并清理孤儿时间块
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Collecting time blocks for {} uncompleted task instances",
            uncompleted_task_ids.len()
        );

        let mut all_deleted_time_block_ids = Vec::new();

        for task_id in &uncompleted_task_ids {
            // 6.1 找到该任务链接的所有时间块
            let linked_blocks =
                TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, *task_id)
                    .await?;

            // 6.2 删除任务的所有链接和日程
            TaskTimeBlockLinkRepository::delete_all_for_task_in_tx(&mut tx, *task_id).await?;
            TaskScheduleRepository::delete_all_in_tx(&mut tx, *task_id).await?;

            // 6.3 软删除任务
            TaskRepository::soft_delete_in_tx(&mut tx, *task_id, now).await?;

            // 6.4 检查并删除孤儿时间块
            for block in linked_blocks {
                let should_delete = should_delete_orphan_block(&block, &mut tx).await?;
                if should_delete {
                    tracing::info!(
                        "🔄 [DELETE_RECURRENCE] Will delete orphan time block {} (source_type={:?}) after deleting task {}",
                        block.id,
                        block.source_info.as_ref().map(|s| &s.source_type),
                        task_id
                    );
                    TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
                    all_deleted_time_block_ids.push(block.id);
                }
            }
        }

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Deleted {} tasks and {} orphan time blocks",
            uncompleted_task_ids.len(),
            all_deleted_time_block_ids.len()
        );

        // 7. 🔥 硬删除循环规则（而不是软删除）
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Hard deleting recurrence rule {}",
            recurrence_id
        );
        hard_delete_recurrence(&mut tx, recurrence_id).await?;

        // 8. 🔥 硬删除关联的模板
        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Hard deleting associated template {}",
            template_id
        );
        hard_delete_template(&mut tx, template_id).await?;

        // 9. 提交事务
        TransactionHelper::commit(tx).await?;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Successfully deleted recurrence {} and template {}",
            recurrence_id,
            template_id
        );

        // 10. (可选) 发送 SSE 事件
        // TODO: 实现完整的 SSE 事件，包含被删除的时间块信息

        Ok(())
    }

    /// 获取循环规则信息
    struct RecurrenceInfo {
        template_id: Uuid,
    }

    async fn get_recurrence_info(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<RecurrenceInfo> {
        let query = r#"
            SELECT template_id
            FROM task_recurrences
            WHERE id = ?
        "#;

        let row: (String,) = sqlx::query_as(query)
            .bind(recurrence_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => crate::infra::core::AppError::NotFound {
                    entity_type: "TaskRecurrence".to_string(),
                    entity_id: recurrence_id.to_string(),
                },
                _ => crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                ),
            })?;

        let template_id =
            Uuid::parse_str(&row.0).map_err(|e| crate::infra::core::AppError::Conflict {
                message: format!("Invalid template_id UUID '{}': {}", row.0, e),
            })?;

        Ok(RecurrenceInfo { template_id })
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

    /// 硬删除循环规则
    async fn hard_delete_recurrence(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_recurrences
            WHERE id = ?
        "#;

        let result = sqlx::query(query)
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Hard deleted recurrence rule, rows affected: {}",
            result.rows_affected()
        );

        Ok(())
    }

    /// 硬删除模板
    async fn hard_delete_template(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        template_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM templates
            WHERE id = ?
        "#;

        let result = sqlx::query(query)
            .bind(template_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        tracing::info!(
            "🔄 [DELETE_RECURRENCE] Hard deleted template, rows affected: {}",
            result.rows_affected()
        );

        Ok(())
    }
}

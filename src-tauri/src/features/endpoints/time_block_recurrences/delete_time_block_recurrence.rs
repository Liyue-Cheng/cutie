/// 删除时间块循环规则 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `delete_time_block_recurrence`

## 1. 端点签名
DELETE /api/time-block-recurrences/:id

## 2. 预期行为简介
删除指定的时间块循环规则，并软删除所有未来的时间块实例

## 3. 输入输出规范

### 3.1 请求 (Request)
无请求体

### 3.2 响应 (Responses)
**204 No Content**

## 4. 业务逻辑详解
1. 验证规则存在
2. 查询所有关联的未来时间块（start_time >= 今天开始）
3. 删除链接记录（time_block_recurrence_links）
4. 清除时间块的循环字段
5. 软删除未来的时间块实例
6. 硬删除循环规则（time_block_recurrences）
7. 硬删除关联的模板（time_block_templates）

## 5. 预期副作用
- DELETE: time_block_recurrence_links 表
- UPDATE: time_blocks 表 (清除循环字段)
- UPDATE: time_blocks 表 (软删除未来实例)
- DELETE: time_block_recurrences 表 (硬删除)
- DELETE: time_block_templates 表 (硬删除)
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Local, NaiveTime, Utc};
use uuid::Uuid;

use crate::{
    features::shared::{
        TimeBlockRecurrenceRepository, TimeBlockTemplateRepository, TransactionHelper,
    },
    infra::core::{AppError, AppResult, DbError},
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, recurrence_id: Uuid) -> AppResult<()> {
        // 1. 验证循环规则是否存在
        let existing =
            TimeBlockRecurrenceRepository::find_by_id(app_state.db_pool(), recurrence_id).await?;
        let recurrence = existing.ok_or_else(|| AppError::NotFound {
            entity_type: "TimeBlockRecurrence".to_string(),
            entity_id: recurrence_id.to_string(),
        })?;

        // 2. 获取依赖
        let now = app_state.clock().now_utc();

        // 计算今天的开始时间（本地时间 00:00:00）
        let today_start = Local::now()
            .date_naive()
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc);

        // ✅ 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        // 3. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        tracing::info!(
            "🗑️ [DELETE_TB_RECURRENCE] Starting deletion of recurrence {} (template: {})",
            recurrence_id,
            recurrence.template_id
        );

        // 4. 查询所有关联的未来时间块（在删除链接之前！）
        let future_time_block_ids =
            find_future_time_blocks(&mut tx, recurrence_id, today_start).await?;

        tracing::info!(
            "🗑️ [DELETE_TB_RECURRENCE] Found {} future time blocks to delete",
            future_time_block_ids.len()
        );

        // 5. 清除所有关联时间块的循环字段（在删除链接之前，因为用到了子查询）
        clear_all_recurrence_fields(&mut tx, recurrence_id, now).await?;

        // 6. 删除所有链接记录
        delete_all_recurrence_links(&mut tx, recurrence_id).await?;

        // 7. 软删除未来的时间块实例
        for time_block_id in &future_time_block_ids {
            soft_delete_time_block(&mut tx, *time_block_id, now).await?;
        }

        tracing::info!(
            "🗑️ [DELETE_TB_RECURRENCE] Soft deleted {} future time blocks",
            future_time_block_ids.len()
        );

        // 8. 硬删除循环规则
        hard_delete_recurrence(&mut tx, recurrence_id).await?;

        // 9. 硬删除关联的模板
        hard_delete_template(&mut tx, recurrence.template_id).await?;

        // 10. 提交事务
        TransactionHelper::commit(tx).await?;

        tracing::info!(
            "🗑️ [DELETE_TB_RECURRENCE] Successfully deleted recurrence {} and {} future time blocks",
            recurrence_id,
            future_time_block_ids.len()
        );

        Ok(())
    }

    /// 查询所有关联的未来时间块（start_time >= 今天开始）
    async fn find_future_time_blocks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        today_start: DateTime<Utc>,
    ) -> AppResult<Vec<Uuid>> {
        let query = r#"
            SELECT tbrl.time_block_id
            FROM time_block_recurrence_links tbrl
            JOIN time_blocks tb ON tb.id = tbrl.time_block_id
            WHERE tbrl.recurrence_id = ?
              AND tb.start_time >= ?
              AND tb.is_deleted = 0
        "#;

        let time_block_id_strs: Vec<String> = sqlx::query_scalar(query)
            .bind(recurrence_id.to_string())
            .bind(today_start.to_rfc3339())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let time_block_ids: Vec<Uuid> = time_block_id_strs
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

        Ok(time_block_ids)
    }

    /// 删除所有链接记录
    async fn delete_all_recurrence_links(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM time_block_recurrence_links
            WHERE recurrence_id = ?
        "#;

        let result = sqlx::query(query)
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        tracing::info!(
            "🗑️ [DELETE_TB_RECURRENCE] Deleted {} recurrence links",
            result.rows_affected()
        );

        Ok(())
    }

    /// 清除所有关联时间块的循环字段
    async fn clear_all_recurrence_fields(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        // 通过链接表找到所有关联的时间块并清除循环字段
        let query = r#"
            UPDATE time_blocks
            SET recurrence_rule = NULL,
                recurrence_original_date = NULL,
                updated_at = ?
            WHERE id IN (
                SELECT time_block_id FROM time_block_recurrence_links
                WHERE recurrence_id = ?
            )
        "#;

        let result = sqlx::query(query)
            .bind(now.to_rfc3339())
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        tracing::info!(
            "🗑️ [DELETE_TB_RECURRENCE] Cleared recurrence fields for {} time blocks",
            result.rows_affected()
        );

        Ok(())
    }

    /// 软删除时间块
    async fn soft_delete_time_block(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        time_block_id: Uuid,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE time_blocks
            SET is_deleted = 1,
                updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(now.to_rfc3339())
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 硬删除循环规则
    async fn hard_delete_recurrence(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM time_block_recurrences
            WHERE id = ?
        "#;

        let result = sqlx::query(query)
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        tracing::info!(
            "🗑️ [DELETE_TB_RECURRENCE] Hard deleted recurrence rule, rows affected: {}",
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
            DELETE FROM time_block_templates
            WHERE id = ?
        "#;

        let result = sqlx::query(query)
            .bind(template_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        tracing::info!(
            "🗑️ [DELETE_TB_RECURRENCE] Hard deleted template, rows affected: {}",
            result.rows_affected()
        );

        Ok(())
    }
}

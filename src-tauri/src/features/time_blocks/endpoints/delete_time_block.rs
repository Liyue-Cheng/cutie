/// 删除时间块 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::TimeBlock,
    shared::core::{AppError, AppResult},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `delete_time_block`

## API端点
DELETE /api/time-blocks/{id}

## 预期行为简介
删除一个时间块及其所有任务链接。幂等操作。

## 输入输出规范
- **前置条件**: `id` 必须是有效的时间块ID。
- **后置条件**: 返回 `204 No Content`。
- **不变量**: 无。

## 边界情况
- 时间块不存在: 幂等地返回 `204`。

## 预期副作用
- 软删除 `time_blocks` 表中的1条记录（设置 is_deleted = true）。
- 删除 `task_time_block_links` 表中的所有相关记录（物理删除，因为有 ON DELETE CASCADE）。
- 所有数据库写入在单个事务中。
*/

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(State(app_state): State<AppState>, Path(block_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, block_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, block_id: Uuid) -> AppResult<()> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 验证时间块存在（幂等）
        let block = match database::find_time_block_by_id_in_tx(&mut tx, block_id).await? {
            Some(b) => b,
            None => {
                // 幂等：时间块不存在也返回成功
                return Ok(());
            }
        };

        // 2. 耦合操作：删除所有任务链接
        database::delete_all_links_for_block_in_tx(&mut tx, block.id).await?;

        // 3. 核心操作：软删除时间块
        let now = app_state.clock().now_utc();
        database::delete_time_block_in_tx(&mut tx, block_id, now).await?;

        // 4. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::time_block::TimeBlockRow;
    use chrono::DateTime;

    pub async fn find_time_block_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<Option<TimeBlock>> {
        let row = sqlx::query_as::<_, TimeBlockRow>(
            r#"
            SELECT id, title, glance_note, detail_note, start_time, end_time,
                   area_id, created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM time_blocks WHERE id = ? AND is_deleted = false
            "#,
        )
        .bind(block_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| TimeBlock::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn delete_all_links_for_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM task_time_block_links WHERE time_block_id = ?")
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }

    pub async fn delete_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query("UPDATE time_blocks SET is_deleted = true, updated_at = ? WHERE id = ?")
            .bind(updated_at.to_rfc3339())
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }
}



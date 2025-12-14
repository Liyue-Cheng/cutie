/// 停止时间块循环 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `stop_time_block_recurrence`

## 1. 端点签名
POST /api/time-block-recurrences/:id/stop

## 2. 预期行为简介
在指定日期停止时间块循环规则，并删除该日期之后的所有时间块实例

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "stop_date": "YYYY-MM-DD (required)" // 停止日期，该日期之后的实例将被删除
}

### 3.2 响应 (Responses)
**200 OK:** TimeBlockRecurrenceDto

## 4. 业务逻辑详解
1. 验证规则存在
2. 设置 end_date 为 stop_date
3. 查询 stop_date 之后的所有时间块（不包含 stop_date 当天）
4. 删除这些时间块的链接记录
5. 软删除这些时间块实例
6. 返回更新后的循环规则

## 5. 预期副作用
- UPDATE: time_block_recurrences 表 (设置 end_date)
- DELETE: time_block_recurrence_links 表 (删除 stop_date 之后的链接)
- UPDATE: time_blocks 表 (软删除 stop_date 之后的实例)
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::TimeBlockRecurrenceDto,
    features::shared::{TimeBlockRecurrenceRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult, DbError},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 请求 DTO ====================
#[derive(Debug, Deserialize)]
pub struct StopRecurrenceRequest {
    /// 停止日期（YYYY-MM-DD），该日期之后的实例将被删除
    pub stop_date: String,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<StopRecurrenceRequest>,
) -> Response {
    match logic::execute(&app_state, id, request).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        recurrence_id: Uuid,
        request: StopRecurrenceRequest,
    ) -> AppResult<TimeBlockRecurrenceDto> {
        // 1. 解析停止日期
        let stop_date =
            NaiveDate::parse_from_str(&request.stop_date, "%Y-%m-%d").map_err(|_| {
                AppError::validation_error(
                    "stop_date",
                    "停止日期格式无效，应为 YYYY-MM-DD",
                    "INVALID_DATE_FORMAT",
                )
            })?;

        // 2. 验证循环规则是否存在
        let existing =
            TimeBlockRecurrenceRepository::find_by_id(app_state.db_pool(), recurrence_id).await?;
        let _recurrence = existing.ok_or_else(|| AppError::NotFound {
            entity_type: "TimeBlockRecurrence".to_string(),
            entity_id: recurrence_id.to_string(),
        })?;

        // 3. 获取依赖
        let now = app_state.clock().now_utc();

        // 计算 stop_date 的下一天开始时间（本地时间）作为删除阈值
        let next_day = stop_date.succ_opt().unwrap_or(stop_date);
        let delete_threshold = next_day.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let delete_threshold_utc = Local
            .from_local_datetime(&delete_threshold)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now());

        // ✅ 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        // 4. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        tracing::info!(
            "⏹️ [STOP_TB_RECURRENCE] Stopping recurrence {} at date {}, deleting instances after {}",
            recurrence_id,
            stop_date,
            delete_threshold_utc.to_rfc3339()
        );

        // 5. 查询需要删除的时间块（stop_date 之后的）
        let time_block_ids_to_delete =
            find_time_blocks_after_date(&mut tx, recurrence_id, delete_threshold_utc).await?;

        tracing::info!(
            "⏹️ [STOP_TB_RECURRENCE] Found {} time blocks to delete after {}",
            time_block_ids_to_delete.len(),
            stop_date
        );

        // 6. 删除这些时间块的链接记录
        delete_links_for_time_blocks(&mut tx, &time_block_ids_to_delete).await?;

        // 7. 清除这些时间块的循环字段
        clear_recurrence_fields_for_time_blocks(&mut tx, &time_block_ids_to_delete, now).await?;

        // 8. 软删除这些时间块
        soft_delete_time_blocks(&mut tx, &time_block_ids_to_delete, now).await?;

        // 9. 更新循环规则的 end_date
        let update_request = crate::entities::UpdateTimeBlockRecurrenceRequest {
            template_id: None,
            rule: None,
            time_type: None,
            start_date: None,
            end_date: Some(Some(request.stop_date.clone())), // 使用请求中的字符串
            timezone: None,
            is_active: None,
        };
        let updated = TimeBlockRecurrenceRepository::update_in_tx(
            &mut tx,
            recurrence_id,
            &update_request,
            now,
        )
        .await?;

        // 10. 提交事务
        TransactionHelper::commit(tx).await?;

        tracing::info!(
            "⏹️ [STOP_TB_RECURRENCE] Successfully stopped recurrence {} at {}, deleted {} time blocks",
            recurrence_id,
            stop_date,
            time_block_ids_to_delete.len()
        );

        // 11. 组装 DTO
        let dto = TimeBlockRecurrenceDto {
            id: updated.id,
            template_id: updated.template_id,
            rule: updated.rule,
            time_type: updated.time_type,
            start_date: updated.start_date,
            end_date: updated.end_date,
            timezone: updated.timezone,
            is_active: updated.is_active,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        };

        Ok(dto)
    }

    /// 查询指定日期之后的时间块
    async fn find_time_blocks_after_date(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        threshold: DateTime<Utc>,
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
            .bind(threshold.to_rfc3339())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let time_block_ids: Vec<Uuid> = time_block_id_strs
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

        Ok(time_block_ids)
    }

    /// 删除指定时间块的链接记录
    async fn delete_links_for_time_blocks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        time_block_ids: &[Uuid],
    ) -> AppResult<()> {
        if time_block_ids.is_empty() {
            return Ok(());
        }

        for time_block_id in time_block_ids {
            let query = r#"
                DELETE FROM time_block_recurrence_links
                WHERE time_block_id = ?
            "#;

            sqlx::query(query)
                .bind(time_block_id.to_string())
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        }

        Ok(())
    }

    /// 清除时间块的循环字段
    async fn clear_recurrence_fields_for_time_blocks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        time_block_ids: &[Uuid],
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        if time_block_ids.is_empty() {
            return Ok(());
        }

        for time_block_id in time_block_ids {
            let query = r#"
                UPDATE time_blocks
                SET recurrence_rule = NULL,
                    recurrence_original_date = NULL,
                    updated_at = ?
                WHERE id = ?
            "#;

            sqlx::query(query)
                .bind(now.to_rfc3339())
                .bind(time_block_id.to_string())
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        }

        Ok(())
    }

    /// 软删除时间块
    async fn soft_delete_time_blocks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        time_block_ids: &[Uuid],
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        if time_block_ids.is_empty() {
            return Ok(());
        }

        for time_block_id in time_block_ids {
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
        }

        Ok(())
    }
}

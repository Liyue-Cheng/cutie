/// 编辑时间块循环规则 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `edit_time_block_recurrence`

## 1. 端点签名
POST /api/time-block-recurrences/:id/edit

## 2. 预期行为简介
允许用户更新时间块循环规则与模板信息，并从当前本地时间起删除所有未来实例

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "rule": "string (optional)",
  "end_date": "YYYY-MM-DD | null (optional)",
  "timezone": "string | null (optional)",
  "skip_conflicts": "boolean (optional)",
  "time_type": "FLOATING | FIXED (optional)",
  "title": "string | null (optional)",
  "glance_note_template": "string | null (optional)",
  "detail_note_template": "string | null (optional)",
  "duration_minutes": 90 (optional),
  "is_all_day": true (optional),
  "area_id": "uuid | null (optional)",
  "local_now": "YYYY-MM-DDTHH:mm",
  "delete_future_instances": true
}

### 3.2 响应 (Responses)
**200 OK:** TimeBlockRecurrenceEditResultDto

## 4. 业务逻辑详解
1. 校验循环规则存在
2. 校验 local_now 字符串格式
3. 更新模板与循环规则（禁止修改 start_time_local / start_date）
4. 根据 local_now 删除所有未来时间块实例（可选）
5. 返回更新后的循环规则及删除的实例 ID 列表

## 5. 预期副作用
- UPDATE: time_block_templates 表
- UPDATE: time_block_recurrences 表
- DELETE: time_block_recurrence_links（未来实例）
- UPDATE: time_blocks（清除循环字段并软删除未来实例）
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use uuid::Uuid;

use crate::{
    entities::{
        time_block_recurrence::{
            EditTimeBlockRecurrenceRequest, TimeBlockRecurrenceDetailDto, TimeBlockRecurrenceEditResultDto,
        },
        time_block_template::UpdateTimeBlockTemplateRequest,
        TimeBlockTemplateInfo,
    },
    features::shared::{
        TimeBlockRecurrenceRepository, TimeBlockTemplateRepository, TransactionHelper,
    },
    infra::{
        core::{AppError, AppResult, DbError, ValidationError},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<EditTimeBlockRecurrenceRequest>,
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
        request: EditTimeBlockRecurrenceRequest,
    ) -> AppResult<TimeBlockRecurrenceEditResultDto> {
        validate_request(&request)?;

        // 1. 查询循环规则与模板
        let existing = TimeBlockRecurrenceRepository::find_by_id(app_state.db_pool(), recurrence_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "TimeBlockRecurrence".to_string(),
                entity_id: recurrence_id.to_string(),
            })?;

        let template = TimeBlockTemplateRepository::find_by_id(app_state.db_pool(), existing.template_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "TimeBlockTemplate".to_string(),
                entity_id: existing.template_id.to_string(),
            })?;

        // 2. 解析 local_now（本地时间，精确到分钟）
        let cutoff_local = parse_local_now(&request.local_now)?;
        let cutoff_utc = cutoff_local.with_timezone(&Utc);

        let delete_future_instances = request.delete_future_instances.unwrap_or(true);

        // 3. 组装更新 payload
        let template_update = build_template_update(&request);
        let recurrence_update = build_recurrence_update(&request);

        // 4. 获取依赖
        let now = app_state.clock().now_utc();
        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 5. 更新模板（如有修改）
        if has_template_updates(&template_update) {
            TimeBlockTemplateRepository::update_in_tx(
                &mut tx,
                template.id,
                &template_update,
                now,
            )
            .await?;
        }

        // 6. 更新循环规则（如有修改）
        if has_recurrence_updates(&recurrence_update) {
            TimeBlockRecurrenceRepository::update_in_tx(
                &mut tx,
                recurrence_id,
                &recurrence_update,
                now,
            )
            .await?;
        }

        // 7. 删除未来时间块实例
        let mut deleted_time_block_ids: Vec<Uuid> = Vec::new();
        if delete_future_instances {
            deleted_time_block_ids =
                cleanup_future_time_blocks(&mut tx, recurrence_id, cutoff_utc).await?;
        }

        // 8. 提交事务
        TransactionHelper::commit(tx).await?;

        // 9. 重新加载详情 DTO
        let detail = load_recurrence_detail(app_state, recurrence_id).await?;

        Ok(TimeBlockRecurrenceEditResultDto {
            recurrence: detail,
            deleted_count: deleted_time_block_ids.len(),
            deleted_time_block_ids,
        })
    }

    fn validate_request(request: &EditTimeBlockRecurrenceRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        if request.local_now.trim().is_empty() {
            errors.push(ValidationError::new(
                "local_now".to_string(),
                "local_now is required".to_string(),
                "LOCAL_NOW_REQUIRED".to_string(),
            ));
        }

        if let Some(Some(ref end_date)) = request.end_date {
            if chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d").is_err() {
                errors.push(ValidationError::new(
                    "end_date".to_string(),
                    "end_date must be in YYYY-MM-DD format".to_string(),
                    "INVALID_DATE".to_string(),
                ));
            }
        }

        if let Some(ref rule) = request.rule {
            if rule.trim().is_empty() {
                errors.push(ValidationError::new(
                    "rule".to_string(),
                    "rule cannot be empty".to_string(),
                    "RULE_EMPTY".to_string(),
                ));
            }
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors));
        }

        Ok(())
    }

    fn parse_local_now(local_now: &str) -> AppResult<DateTime<Local>> {
        let naive = NaiveDateTime::parse_from_str(local_now, "%Y-%m-%dT%H:%M").map_err(|_| {
            AppError::validation_error(
                "local_now",
                "local_now must be in YYYY-MM-DDTHH:mm format",
                "INVALID_LOCAL_NOW",
            )
        })?;

        Local
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| AppError::validation_error("local_now", "Invalid local time", "INVALID_LOCAL_NOW"))
    }

    fn build_template_update(request: &EditTimeBlockRecurrenceRequest) -> UpdateTimeBlockTemplateRequest {
        UpdateTimeBlockTemplateRequest {
            title: request.title.clone(),
            glance_note_template: request.glance_note_template.clone(),
            detail_note_template: request.detail_note_template.clone(),
            duration_minutes: request.duration_minutes,
            start_time_local: None, // 起始时间不可编辑
            time_type: request.time_type,
            is_all_day: request.is_all_day,
            area_id: request.area_id.clone(),
        }
    }

    fn build_recurrence_update(
        request: &EditTimeBlockRecurrenceRequest,
    ) -> crate::entities::time_block_recurrence::UpdateTimeBlockRecurrenceRequest {
        crate::entities::time_block_recurrence::UpdateTimeBlockRecurrenceRequest {
            template_id: None,
            rule: request.rule.clone(),
            time_type: request.time_type,
            start_date: None, // 起始日期禁止修改
            end_date: request.end_date.clone(),
            timezone: request.timezone.clone(),
            skip_conflicts: request.skip_conflicts,
            is_active: None,
        }
    }

    fn has_template_updates(update: &UpdateTimeBlockTemplateRequest) -> bool {
        update.title.is_some()
            || update.glance_note_template.is_some()
            || update.detail_note_template.is_some()
            || update.duration_minutes.is_some()
            || update.time_type.is_some()
            || update.is_all_day.is_some()
            || update.area_id.is_some()
    }

    fn has_recurrence_updates(
        update: &crate::entities::time_block_recurrence::UpdateTimeBlockRecurrenceRequest,
    ) -> bool {
        update.rule.is_some()
            || update.time_type.is_some()
            || update.end_date.is_some()
            || update.timezone.is_some()
            || update.skip_conflicts.is_some()
    }

    async fn cleanup_future_time_blocks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        cutoff: DateTime<Utc>,
    ) -> AppResult<Vec<Uuid>> {
        // 查询需要删除的时间块
        let query = r#"
            SELECT tb.id
            FROM time_block_recurrence_links tbrl
            JOIN time_blocks tb ON tb.id = tbrl.time_block_id
            WHERE tbrl.recurrence_id = ?
              AND tb.start_time >= ?
              AND tb.is_deleted = 0
        "#;

        let id_strings: Vec<String> = sqlx::query_scalar(query)
            .bind(recurrence_id.to_string())
            .bind(cutoff.to_rfc3339())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let mut ids: Vec<Uuid> = Vec::with_capacity(id_strings.len());
        for id_str in id_strings {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                ids.push(id);
            }
        }

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let now = Utc::now();

        delete_links_for_time_blocks(tx, &ids).await?;
        clear_recurrence_fields_for_time_blocks(tx, &ids, now).await?;
        soft_delete_time_blocks(tx, &ids, now).await?;

        Ok(ids)
    }

    async fn delete_links_for_time_blocks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        time_block_ids: &[Uuid],
    ) -> AppResult<()> {
        if time_block_ids.is_empty() {
            return Ok(());
        }

        let query = r#"
            DELETE FROM time_block_recurrence_links
            WHERE time_block_id = ?
        "#;

        for id in time_block_ids {
            sqlx::query(query)
                .bind(id.to_string())
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        }
        Ok(())
    }

    async fn clear_recurrence_fields_for_time_blocks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        time_block_ids: &[Uuid],
        timestamp: DateTime<Utc>,
    ) -> AppResult<()> {
        if time_block_ids.is_empty() {
            return Ok(());
        }

        let query = r#"
            UPDATE time_blocks
            SET recurrence_rule = NULL,
                recurrence_original_date = NULL,
                updated_at = ?
            WHERE id = ?
        "#;

        for id in time_block_ids {
            sqlx::query(query)
                .bind(timestamp.to_rfc3339())
                .bind(id.to_string())
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        }
        Ok(())
    }

    async fn soft_delete_time_blocks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        time_block_ids: &[Uuid],
        timestamp: DateTime<Utc>,
    ) -> AppResult<()> {
        if time_block_ids.is_empty() {
            return Ok(());
        }

        let query = r#"
            UPDATE time_blocks
            SET is_deleted = 1,
                updated_at = ?
            WHERE id = ?
        "#;

        for id in time_block_ids {
            sqlx::query(query)
                .bind(timestamp.to_rfc3339())
                .bind(id.to_string())
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        }
        Ok(())
    }

    async fn load_recurrence_detail(
        app_state: &AppState,
        recurrence_id: Uuid,
    ) -> AppResult<TimeBlockRecurrenceDetailDto> {
        let recurrence =
            TimeBlockRecurrenceRepository::find_by_id(app_state.db_pool(), recurrence_id)
                .await?
                .ok_or_else(|| AppError::NotFound {
                    entity_type: "TimeBlockRecurrence".to_string(),
                    entity_id: recurrence_id.to_string(),
                })?;

        let template_info =
            TimeBlockTemplateRepository::find_by_id(app_state.db_pool(), recurrence.template_id)
                .await?
                .map(|template| TimeBlockTemplateInfo {
                    id: template.id,
                    title: template.title,
                    glance_note_template: template.glance_note_template,
                    detail_note_template: template.detail_note_template,
                    duration_minutes: template.duration_minutes,
                    start_time_local: template.start_time_local,
                    is_all_day: template.is_all_day,
                    area_id: template.area_id,
                });

        Ok(TimeBlockRecurrenceDetailDto {
            id: recurrence.id,
            template_id: recurrence.template_id,
            rule: recurrence.rule,
            time_type: recurrence.time_type,
            start_date: recurrence.start_date,
            end_date: recurrence.end_date,
            timezone: recurrence.timezone,
            skip_conflicts: recurrence.skip_conflicts,
            is_active: recurrence.is_active,
            created_at: recurrence.created_at,
            updated_at: recurrence.updated_at,
            template: template_info,
        })
    }
}

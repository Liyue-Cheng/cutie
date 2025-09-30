/// 更新时间块 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::TimeBlock,
    shared::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `update_time_block`

## API端点
PATCH /api/time-blocks/{id}

## 预期行为简介
更新一个时间块的属性，如起止时间、标题、笔记、Area等。

## 输入输出规范
- **前置条件**: `id` 必须是有效的时间块ID。请求体中所有非 `None` 的字段都必须通过验证。
- **后置条件**: 返回 `200 OK` 和更新后的 `TimeBlock` 对象。
- **不变量**: start_time <= end_time

## 边界情况
- 时间块不存在: 返回 `404 Not Found`。
- 输入数据验证失败: 返回 `422 Unprocessable Entity`。
- start_time 晚于 end_time: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 更新 `time_blocks` 表中的1条记录。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "start_time": "2024-01-01T10:00:00Z",
  "end_time": "2024-01-01T11:30:00Z",
  "title": "更新后的会议",
  "glance_note": "与客户讨论项目进展",
  "detail_note": null,
  "area_id": "uuid-string"
}
```
*/

#[derive(Deserialize)]
pub struct UpdateTimeBlockRequest {
    start_time: Option<DateTime<Utc>>, // Serde自动处理RFC3339格式
    end_time: Option<DateTime<Utc>>, // Serde自动处理RFC3339格式
    title: Option<Option<String>>,
    glance_note: Option<Option<String>>,
    detail_note: Option<Option<String>>,
    area_id: Option<Option<String>>,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    Json(request): Json<UpdateTimeBlockRequest>,
) -> Response {
    match logic::execute(&app_state, block_id, request).await {
        Ok(time_block) => success_response(time_block).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub struct ValidatedUpdates {
        pub start_time: Option<DateTime<Utc>>,
        pub end_time: Option<DateTime<Utc>>,
        pub title: Option<Option<String>>,
        pub glance_note: Option<Option<String>>,
        pub detail_note: Option<Option<String>>,
        pub area_id: Option<Option<Uuid>>,
    }

    pub fn validate_request(
        request: &UpdateTimeBlockRequest,
    ) -> Result<ValidatedUpdates, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Serde已经处理了时间格式验证，直接使用即可
        let start_time = request.start_time;
        let end_time = request.end_time;

        // 2. 验证 area_id
        let area_id = if let Some(ref maybe_area_id) = request.area_id {
            Some(if let Some(ref area_id_str) = maybe_area_id {
                match Uuid::parse_str(area_id_str) {
                    Ok(id) => Some(id),
                    Err(_) => {
                        errors.push(ValidationError::new(
                            "area_id",
                            "Area ID 格式无效",
                            "INVALID_AREA_ID",
                        ));
                        None
                    }
                }
            } else {
                None
            })
        } else {
            None
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidatedUpdates {
            start_time,
            end_time,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            area_id,
        })
    }

    pub fn validate_time_range(
        block: &TimeBlock,
        updates: &ValidatedUpdates,
    ) -> Result<(), Vec<ValidationError>> {
        let final_start = updates.start_time.unwrap_or(block.start_time);
        let final_end = updates.end_time.unwrap_or(block.end_time);

        if final_start > final_end {
            return Err(vec![ValidationError::new(
                "time_range",
                "开始时间不能晚于结束时间",
                "INVALID_TIME_RANGE",
            )]);
        }

        Ok(())
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        block_id: Uuid,
        request: UpdateTimeBlockRequest,
    ) -> AppResult<TimeBlock> {
        // 1. 验证请求
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 获取现有时间块
        let mut block = database::find_time_block_by_id_in_tx(&mut tx, block_id)
            .await?
            .ok_or_else(|| AppError::not_found("TimeBlock", block_id.to_string()))?;

        // 3. 验证时间范围
        validation::validate_time_range(&block, &validated)
            .map_err(AppError::ValidationFailed)?;

        // 4. 验证 area_id 是否存在（如果要更新）
        if let Some(Some(area_id)) = validated.area_id {
            let area_exists = database::area_exists_in_tx(&mut tx, area_id).await?;
            if !area_exists {
                return Err(AppError::not_found("Area", area_id.to_string()));
            }
        }

        // 5. 应用更新
        let now = app_state.clock().now_utc();

        if let Some(start_time) = validated.start_time {
            block.start_time = start_time;
        }
        if let Some(end_time) = validated.end_time {
            block.end_time = end_time;
        }
        if let Some(title) = validated.title {
            block.title = title;
        }
        if let Some(glance_note) = validated.glance_note {
            block.glance_note = glance_note;
        }
        if let Some(detail_note) = validated.detail_note {
            block.detail_note = detail_note;
        }
        if let Some(area_id) = validated.area_id {
            block.area_id = area_id;
        }

        block.updated_at = now;

        // 6. 核心操作：持久化更新
        let updated_block = database::update_time_block_in_tx(&mut tx, &block).await?;

        // 7. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_block)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::time_block::TimeBlockRow;

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

    pub async fn area_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM areas WHERE id = ? AND is_deleted = false")
                .bind(area_id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
                })?;

        Ok(count > 0)
    }

    pub async fn update_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block: &TimeBlock,
    ) -> AppResult<TimeBlock> {
        let source_info_json = block
            .source_info
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        let external_metadata_json = block
            .external_source_metadata
            .as_ref()
            .and_then(|m| serde_json::to_string(m).ok());

        let recurrence_exclusions_json = block
            .recurrence_exclusions
            .as_ref()
            .and_then(|e| serde_json::to_string(e).ok());

        let row = sqlx::query_as::<_, TimeBlockRow>(
            r#"
            UPDATE time_blocks SET
                title = ?, glance_note = ?, detail_note = ?,
                start_time = ?, end_time = ?, area_id = ?, updated_at = ?
            WHERE id = ? AND is_deleted = false
            RETURNING id, title, glance_note, detail_note, start_time, end_time,
                      area_id, created_at, updated_at, is_deleted, source_info,
                      external_source_id, external_source_provider, external_source_metadata,
                      recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            "#,
        )
        .bind(&block.title)
        .bind(&block.glance_note)
        .bind(&block.detail_note)
        .bind(block.start_time.to_rfc3339())
        .bind(block.end_time.to_rfc3339())
        .bind(block.area_id.map(|id| id.to_string()))
        .bind(block.updated_at.to_rfc3339())
        .bind(block.id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        TimeBlock::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}



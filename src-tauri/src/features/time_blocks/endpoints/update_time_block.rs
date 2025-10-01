/// 更新时间块 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{
        task::response_dtos::AreaSummary, LinkedTaskSummary, TimeBlock, TimeBlockRow,
        TimeBlockViewDto, UpdateTimeBlockRequest,
    },
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `update_time_block`

## API端点
PATCH /api/time-blocks/:id

## 预期行为简介
更新现有时间块的时间范围、标题、笔记或区域。
支持拖曳时间块调整时间的核心功能。

## 输入输出规范
- **前置条件**:
  - 时间块必须存在且未被删除
  - 如果更新时间范围，start_time < end_time
  - 更新后的时间不与其他时间块重叠
- **后置条件**:
  - 更新 time_blocks 表中的记录
  - 返回完整的 TimeBlockViewDto（包含关联任务和区域信息）

## 边界情况
- 如果时间块不存在，返回 404 Not Found
- 如果时间范围无效，返回 400 Bad Request
- 如果与其他时间块重叠，返回 409 Conflict

## 预期副作用
- 更新一条 time_blocks 记录
- 更新 updated_at 时间戳

## 事务保证
- 所有数据库操作在单个事务中执行
- 如果任何步骤失败，整个操作回滚
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTimeBlockRequest>,
) -> Response {
    match logic::execute(&app_state, id, request).await {
        Ok(time_block_view) => success_response(time_block_view).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_update_request(request: &UpdateTimeBlockRequest) -> AppResult<()> {
        // 如果同时更新开始和结束时间，验证时间范围
        if let (Some(start), Some(end)) = (request.start_time, request.end_time) {
            if start >= end {
                return Err(AppError::validation_error(
                    "time_range",
                    "开始时间必须早于结束时间",
                    "INVALID_TIME_RANGE",
                ));
            }
        }

        // 验证标题长度（如果有）
        if let Some(Some(title)) = &request.title {
            if title.len() > 255 {
                return Err(AppError::validation_error(
                    "title",
                    "标题不能超过255个字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        Ok(())
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        id: Uuid,
        request: UpdateTimeBlockRequest,
    ) -> AppResult<TimeBlockViewDto> {
        // 1. 验证请求
        validation::validate_update_request(&request)?;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 获取现有时间块（确保存在）
        let existing_block = database::get_time_block_in_tx(&mut tx, id).await?;

        // 4. 确定最终的时间范围
        let final_start_time = request.start_time.unwrap_or(existing_block.start_time);
        let final_end_time = request.end_time.unwrap_or(existing_block.end_time);

        // 5. 再次验证最终时间范围
        if final_start_time >= final_end_time {
            return Err(AppError::validation_error(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        // 6. 如果时间范围发生变化，检查时间冲突
        if request.start_time.is_some() || request.end_time.is_some() {
            let has_conflict = database::check_time_conflict_in_tx(
                &mut tx,
                &final_start_time,
                &final_end_time,
                Some(id), // 排除当前时间块
            )
            .await?;

            if has_conflict {
                return Err(AppError::conflict(
                    "该时间段与现有时间块重叠，时间块不允许重叠",
                ));
            }
        }

        // 7. 获取当前时间戳
        let now = app_state.clock().now_utc();

        // 8. 更新时间块
        database::update_time_block_in_tx(&mut tx, id, &request, now).await?;

        // 9. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 10. 重新查询时间块以获取最新数据
        let updated_block = database::get_time_block(app_state.db_pool(), id).await?;

        // 11. 组装返回的 TimeBlockViewDto
        let mut time_block_view = TimeBlockViewDto {
            id: updated_block.id,
            start_time: updated_block.start_time,
            end_time: updated_block.end_time,
            title: updated_block.title,
            glance_note: updated_block.glance_note,
            detail_note: updated_block.detail_note,
            area: None,
            linked_tasks: Vec::new(),
            is_recurring: updated_block.recurrence_rule.is_some(),
        };

        // 12. 获取区域信息（如果有）
        if let Some(area_id) = updated_block.area_id {
            time_block_view.area = database::get_area_summary(app_state.db_pool(), area_id).await?;
        }

        // 13. 获取关联的任务摘要
        time_block_view.linked_tasks =
            database::get_linked_tasks_for_block(app_state.db_pool(), id).await?;

        tracing::info!("Updated time block: {}", id);

        Ok(time_block_view)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    /// 在事务中获取时间块
    pub async fn get_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        id: Uuid,
    ) -> AppResult<TimeBlock> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, start_time, end_time, area_id,
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM time_blocks
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, TimeBlockRow>(query)
            .bind(id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        row.ok_or_else(|| AppError::not_found("TimeBlock", id.to_string()))
            .and_then(|r| {
                TimeBlock::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })
            })
    }

    /// 获取时间块（非事务）
    pub async fn get_time_block(pool: &sqlx::SqlitePool, id: Uuid) -> AppResult<TimeBlock> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, start_time, end_time, area_id,
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM time_blocks
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, TimeBlockRow>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        row.ok_or_else(|| AppError::not_found("TimeBlock", "unknown"))
            .and_then(|r| {
                TimeBlock::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })
            })
    }

    /// 在事务中检查时间冲突
    pub async fn check_time_conflict_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
        exclude_id: Option<Uuid>,
    ) -> AppResult<bool> {
        let mut query = String::from(
            r#"
            SELECT COUNT(*) as count
            FROM time_blocks
            WHERE is_deleted = false
              AND start_time < ?
              AND end_time > ?
        "#,
        );

        if let Some(id) = exclude_id {
            query.push_str(&format!(" AND id != '{}'", id));
        }

        let count: i64 = sqlx::query_scalar(&query)
            .bind(end_time.to_rfc3339())
            .bind(start_time.to_rfc3339())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(count > 0)
    }

    /// 在事务中更新时间块
    pub async fn update_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        id: Uuid,
        request: &UpdateTimeBlockRequest,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        // 构建动态 UPDATE 语句
        if let Some(ref title_opt) = request.title {
            updates.push("title = ?");
            bindings.push(title_opt.clone().unwrap_or_default());
        }

        if let Some(ref glance_note_opt) = request.glance_note {
            updates.push("glance_note = ?");
            bindings.push(glance_note_opt.clone().unwrap_or_default());
        }

        if let Some(ref detail_note_opt) = request.detail_note {
            updates.push("detail_note = ?");
            bindings.push(detail_note_opt.clone().unwrap_or_default());
        }

        if let Some(start_time) = request.start_time {
            updates.push("start_time = ?");
            bindings.push(start_time.to_rfc3339());
        }

        if let Some(end_time) = request.end_time {
            updates.push("end_time = ?");
            bindings.push(end_time.to_rfc3339());
        }

        if let Some(ref area_id_opt) = request.area_id {
            updates.push("area_id = ?");
            bindings.push(area_id_opt.map(|id| id.to_string()).unwrap_or_default());
        }

        // 如果没有任何字段要更新，直接返回
        if updates.is_empty() {
            return Ok(());
        }

        // 添加 updated_at
        updates.push("updated_at = ?");

        let query = format!("UPDATE time_blocks SET {} WHERE id = ?", updates.join(", "));

        let mut query_builder = sqlx::query(&query);

        // 绑定参数
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        // 绑定 updated_at 和 id
        query_builder = query_builder
            .bind(updated_at.to_rfc3339())
            .bind(id.to_string());

        query_builder.execute(&mut **tx).await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        Ok(())
    }

    /// 获取区域摘要信息
    pub async fn get_area_summary(
        pool: &sqlx::SqlitePool,
        area_id: Uuid,
    ) -> AppResult<Option<AreaSummary>> {
        let query = r#"
            SELECT id, name, color
            FROM areas
            WHERE id = ? AND is_deleted = false
        "#;

        let result = sqlx::query_as::<_, (String, String, String)>(query)
            .bind(area_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(result.map(|(id, name, color)| AreaSummary {
            id: Uuid::parse_str(&id).unwrap(),
            name,
            color,
        }))
    }

    /// 获取时间块关联的任务摘要
    pub async fn get_linked_tasks_for_block(
        pool: &sqlx::SqlitePool,
        block_id: Uuid,
    ) -> AppResult<Vec<LinkedTaskSummary>> {
        let query = r#"
            SELECT t.id, t.title, t.completed_at
            FROM tasks t
            INNER JOIN task_time_block_links ttbl ON t.id = ttbl.task_id
            WHERE ttbl.time_block_id = ? AND t.is_deleted = false
        "#;

        let rows = sqlx::query_as::<_, (String, String, Option<String>)>(query)
            .bind(block_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let summaries = rows
            .into_iter()
            .map(|(id, title, completed_at)| LinkedTaskSummary {
                id: Uuid::parse_str(&id).unwrap(),
                title,
                is_completed: completed_at.is_some(),
            })
            .collect();

        Ok(summaries)
    }
}

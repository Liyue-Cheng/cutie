/// 创建时间块 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{
        task::response_dtos::AreaSummary, CreateTimeBlockRequest, LinkedTaskSummary, TimeBlock,
        TimeBlockViewDto,
    },
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_time_block`

## API端点
POST /api/time-blocks

## 预期行为简介
创建一个新的时间块，并可选地将其与一个或多个任务关联。
支持 Cutie 的核心特性：任务与时间块多对多连接。

## 输入输出规范
- **前置条件**:
  - start_time < end_time
  - 时间块不与现有时间块重叠（关键约束）
  - linked_task_ids 中的任务必须存在
- **后置条件**:
  - 在 time_blocks 表中创建新时间块
  - 在 task_time_block_links 表中创建关联记录
  - 返回完整的 TimeBlockViewDto

## 边界情况
- 如果时间范围无效，返回 400 Bad Request
- 如果与现有时间块重叠，返回 409 Conflict
- 如果关联的任务不存在，返回 404 Not Found

## 预期副作用
- 插入一条 time_blocks 记录
- 插入 N 条 task_time_block_links 记录

## 事务保证
- 所有数据库操作在单个事务中执行
- 如果任何步骤失败，整个操作回滚
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTimeBlockRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(time_block_view) => created_response(time_block_view).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_create_request(request: &CreateTimeBlockRequest) -> AppResult<()> {
        // 验证时间范围
        if request.start_time >= request.end_time {
            return Err(AppError::validation_error(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        // 验证时间不在过去太远（可选，根据需求）
        // 验证标题长度（如果有）
        if let Some(title) = &request.title {
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
        request: CreateTimeBlockRequest,
    ) -> AppResult<TimeBlockViewDto> {
        // 1. 验证请求
        validation::validate_create_request(&request)?;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 检查时间冲突
        let has_conflict = database::check_time_conflict_in_tx(
            &mut tx,
            &request.start_time,
            &request.end_time,
            None, // 新建时没有要排除的ID
        )
        .await?;

        if has_conflict {
            return Err(AppError::conflict(
                "该时间段与现有时间块重叠，时间块不允许重叠",
            ));
        }

        // 4. 生成 UUID 和时间戳
        let block_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 5. 创建时间块实体
        let time_block = TimeBlock {
            id: block_id,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            start_time: request.start_time,
            end_time: request.end_time,
            area_id: request.area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: None,
            recurrence_parent_id: None,
            recurrence_original_date: None,
            recurrence_exclusions: None,
        };

        // 6. 插入时间块到数据库
        database::insert_time_block_in_tx(&mut tx, &time_block).await?;

        // 7. 创建任务链接
        if let Some(task_ids) = &request.linked_task_ids {
            for task_id in task_ids {
                database::link_task_to_block_in_tx(&mut tx, *task_id, block_id).await?;
            }
        }

        // 8. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 9. 组装返回的 TimeBlockViewDto
        let mut time_block_view = TimeBlockViewDto {
            id: time_block.id,
            start_time: time_block.start_time,
            end_time: time_block.end_time,
            title: time_block.title,
            glance_note: time_block.glance_note,
            detail_note: time_block.detail_note,
            area: None,
            linked_tasks: Vec::new(),
            is_recurring: time_block.recurrence_rule.is_some(),
        };

        // 10. 获取区域信息（如果有）
        if let Some(area_id) = time_block.area_id {
            time_block_view.area = database::get_area_summary(app_state.db_pool(), area_id).await?;
        }

        // 11. 获取关联的任务摘要
        if let Some(task_ids) = request.linked_task_ids {
            time_block_view.linked_tasks =
                database::get_linked_tasks_summary(app_state.db_pool(), &task_ids).await?;
        }

        Ok(time_block_view)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

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

    /// 在事务中插入时间块
    pub async fn insert_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block: &TimeBlock,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO time_blocks (
                id, title, glance_note, detail_note, start_time, end_time, area_id,
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(block.id.to_string())
            .bind(&block.title)
            .bind(&block.glance_note)
            .bind(&block.detail_note)
            .bind(block.start_time.to_rfc3339())
            .bind(block.end_time.to_rfc3339())
            .bind(block.area_id.map(|id| id.to_string()))
            .bind(block.created_at.to_rfc3339())
            .bind(block.updated_at.to_rfc3339())
            .bind(block.is_deleted)
            .bind(
                block
                    .source_info
                    .as_ref()
                    .map(|s| serde_json::to_string(s).unwrap()),
            )
            .bind(&block.external_source_id)
            .bind(&block.external_source_provider)
            .bind(
                block
                    .external_source_metadata
                    .as_ref()
                    .map(|m| serde_json::to_string(m).unwrap()),
            )
            .bind(&block.recurrence_rule)
            .bind(block.recurrence_parent_id.map(|id| id.to_string()))
            .bind(block.recurrence_original_date.map(|d| d.to_rfc3339()))
            .bind(
                block
                    .recurrence_exclusions
                    .as_ref()
                    .map(|e| serde_json::to_string(e).unwrap()),
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }

    /// 在事务中创建任务与时间块的链接
    pub async fn link_task_to_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        block_id: Uuid,
    ) -> AppResult<()> {
        let now = Utc::now();

        let query = r#"
            INSERT INTO task_time_block_links (task_id, time_block_id, created_at)
            VALUES (?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(block_id.to_string())
            .bind(now.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
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

    /// 获取关联任务的摘要信息
    pub async fn get_linked_tasks_summary(
        pool: &sqlx::SqlitePool,
        task_ids: &[Uuid],
    ) -> AppResult<Vec<LinkedTaskSummary>> {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        // 构建 IN 查询
        let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            r#"
            SELECT id, title, completed_at
            FROM tasks
            WHERE id IN ({}) AND is_deleted = false
            "#,
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, (String, String, Option<String>)>(&query);
        for task_id in task_ids {
            query_builder = query_builder.bind(task_id.to_string());
        }

        let rows = query_builder.fetch_all(pool).await.map_err(|e| {
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

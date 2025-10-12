/// 创建时间块 API - 单文件组件
use axum::{
    extract::State,
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
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `create_time_block`

## API端点
POST /api/time-blocks

## 预期行为简介
创建一个新的时间块，并选择性地链接一个或多个任务。

## 输入输出规范
- **前置条件**: 请求体必须包含有效的 `start_time` 和 `end_time`。start_time 不能晚于 end_time。
- **后置条件**: 返回 `201 Created` 和新创建的 `TimeBlock` 对象。
- **不变量**: start_time <= end_time

## 边界情况
- start_time 晚于 end_time: 返回 `422 Unprocessable Entity`。
- 关联的 task_id 不存在: 返回 `404 Not Found`。
- 时间格式无效: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 在 `time_blocks` 表插入1条记录。
- 如果提供了 `task_ids`，在 `task_time_block_links` 表插入对应数量的记录。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "start_time": "2024-01-01T10:00:00Z",
  "end_time": "2024-01-01T11:00:00Z",
  "title": "会议",
  "glance_note": "与客户讨论项目",
  "detail_note": "详细讨论项目需求和时间表",
  "area_id": "uuid-string",
  "task_ids": ["task-uuid-1", "task-uuid-2"]
}
```
*/

#[derive(Deserialize)]
pub struct CreateTimeBlockRequest {
    start_time: DateTime<Utc>, // Serde自动处理RFC3339格式
    end_time: DateTime<Utc>,   // Serde自动处理RFC3339格式
    title: Option<String>,
    glance_note: Option<String>,
    detail_note: Option<String>,
    area_id: Option<String>,
    task_ids: Option<Vec<String>>,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTimeBlockRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(time_block) => created_response(time_block).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub struct ValidatedTimeBlockData {
        pub start_time: DateTime<Utc>,
        pub end_time: DateTime<Utc>,
        pub title: Option<String>,
        pub glance_note: Option<String>,
        pub detail_note: Option<String>,
        pub area_id: Option<Uuid>,
        pub task_ids: Vec<Uuid>,
    }

    pub fn validate_request(
        request: &CreateTimeBlockRequest,
    ) -> Result<ValidatedTimeBlockData, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Serde已经处理了时间格式验证，直接使用即可
        let start_time = request.start_time;
        let end_time = request.end_time;

        // 1. 验证时间范围
        if start_time > end_time {
            errors.push(ValidationError::new(
                "time_range",
                "开始时间不能晚于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        // 3. 验证 area_id
        let area_id = if let Some(ref area_id_str) = request.area_id {
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
        };

        // 4. 验证 task_ids
        let mut task_ids = Vec::new();
        if let Some(ref ids) = request.task_ids {
            for (i, id_str) in ids.iter().enumerate() {
                match Uuid::parse_str(id_str) {
                    Ok(id) => task_ids.push(id),
                    Err(_) => {
                        errors.push(ValidationError::new(
                            &format!("task_ids[{}]", i),
                            "Task ID 格式无效",
                            "INVALID_TASK_ID",
                        ));
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidatedTimeBlockData {
            start_time,
            end_time,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            area_id,
            task_ids,
        })
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateTimeBlockRequest,
    ) -> AppResult<TimeBlock> {
        // 1. 验证请求
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 2. 验证所有关联的任务是否存在
        for task_id in &validated.task_ids {
            let task_exists = database::task_exists_in_tx(&mut tx, *task_id).await?;
            if !task_exists {
                return Err(AppError::not_found("Task", task_id.to_string()));
            }
        }

        // 3. 验证 area_id 是否存在（如果提供）
        if let Some(area_id) = validated.area_id {
            let area_exists = database::area_exists_in_tx(&mut tx, area_id).await?;
            if !area_exists {
                return Err(AppError::not_found("Area", area_id.to_string()));
            }
        }

        // 4. 生成 ID 和时间戳
        let new_block_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 5. 核心操作：创建时间块
        let mut new_block =
            TimeBlock::new(new_block_id, validated.start_time, validated.end_time, now).map_err(
                |e| {
                    AppError::ValidationFailed(vec![ValidationError::new(
                        "time_range",
                        &e,
                        "INVALID_TIME_RANGE",
                    )])
                },
            )?;

        new_block.title = validated.title;
        new_block.glance_note = validated.glance_note;
        new_block.detail_note = validated.detail_note;
        new_block.area_id = validated.area_id;

        let created_block = database::create_time_block_in_tx(&mut tx, &new_block).await?;

        // 6. 耦合操作：链接任务
        for task_id in &validated.task_ids {
            database::link_task_to_block_in_tx(&mut tx, *task_id, created_block.id, now).await?;
        }

        // 7. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(created_block)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::time_block::TimeBlockRow;

    pub async fn task_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE id = ? AND deleted_at IS NULL")
                .bind(task_id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
                })?;

        Ok(count > 0)
    }

    pub async fn area_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM areas WHERE id = ? AND deleted_at IS NULL")
                .bind(area_id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
                })?;

        Ok(count > 0)
    }

    pub async fn create_time_block_in_tx(
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
            INSERT INTO time_blocks (
                id, title, glance_note, detail_note, start_time, end_time,
                area_id, created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, title, glance_note, detail_note, start_time, end_time,
                      area_id, created_at, updated_at, is_deleted, source_info,
                      external_source_id, external_source_provider, external_source_metadata,
                      recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            "#,
        )
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
        .bind(source_info_json)
        .bind(&block.external_source_id)
        .bind(&block.external_source_provider)
        .bind(external_metadata_json)
        .bind(&block.recurrence_rule)
        .bind(block.recurrence_parent_id.map(|id| id.to_string()))
        .bind(block.recurrence_original_date.map(|dt| dt.to_rfc3339()))
        .bind(recurrence_exclusions_json)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        TimeBlock::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
    }

    pub async fn link_task_to_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        time_block_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO task_time_block_links (task_id, time_block_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(task_id.to_string())
        .bind(time_block_id.to_string())
        .bind(created_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}

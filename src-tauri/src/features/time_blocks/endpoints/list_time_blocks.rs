/// 获取时间块列表 API - 单文件组件
///
/// 支持按日期范围查询时间块
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::{task::response_dtos::AreaSummary, LinkedTaskSummary, TimeBlock, TimeBlockViewDto},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `list_time_blocks`

## API端点
GET /api/time-blocks?start_date=...&end_date=...

## 预期行为简介
查询指定时间范围内的所有时间块，返回包含关联任务信息的视图模型列表。

## 输入输出规范
- **查询参数**:
  - start_date: 开始时间（ISO 8601 UTC）
  - end_date: 结束时间（ISO 8601 UTC）
- **后置条件**:
  - 返回该时间范围内所有未删除的时间块
  - 每个时间块包含关联的任务摘要
  - 时间块按 start_time 排序

## 边界情况
- 如果时间范围无效（start > end），返回 400
- 如果没有时间块，返回空数组

## 预期副作用
- 无（只读操作）
*/

// ==================== 请求参数 ====================
#[derive(Debug, Deserialize)]
pub struct ListTimeBlocksQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<ListTimeBlocksQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(time_blocks) => success_response(time_blocks).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        query: ListTimeBlocksQuery,
    ) -> AppResult<Vec<TimeBlockViewDto>> {
        let pool = app_state.db_pool();

        // 1. 解析时间范围
        let start_time = query
            .start_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let end_time = query
            .end_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // 2. 查询时间块
        let time_blocks = database::find_time_blocks_in_range(pool, start_time, end_time).await?;

        // 3. 为每个时间块组装视图模型
        let mut result = Vec::new();
        for block in time_blocks {
            let view = assemble_time_block_view(&block, pool).await?;
            result.push(view);
        }

        // 4. 按 start_time 排序
        result.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        Ok(result)
    }

    /// 组装单个时间块的视图模型
    async fn assemble_time_block_view(
        block: &TimeBlock,
        pool: &sqlx::SqlitePool,
    ) -> AppResult<TimeBlockViewDto> {
        // 1. 创建基础视图
        let mut view = TimeBlockViewDto {
            id: block.id,
            start_time: block.start_time,
            end_time: block.end_time,
            title: block.title.clone(),
            glance_note: block.glance_note.clone(),
            detail_note: block.detail_note.clone(),
            area: None,
            linked_tasks: Vec::new(),
            is_recurring: block.recurrence_rule.is_some(),
        };

        // 2. 获取区域信息
        if let Some(area_id) = block.area_id {
            view.area = database::get_area_summary(pool, area_id).await?;
        }

        // 3. 获取关联的任务
        view.linked_tasks = database::get_linked_tasks_for_block(pool, block.id).await?;

        Ok(view)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TimeBlockRow;

    /// 查询时间范围内的时间块
    pub async fn find_time_blocks_in_range(
        pool: &sqlx::SqlitePool,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> AppResult<Vec<TimeBlock>> {
        let mut query = String::from(
            r#"
            SELECT 
                id, title, glance_note, detail_note, start_time, end_time, area_id,
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM time_blocks
            WHERE is_deleted = false
        "#,
        );

        // 添加时间范围过滤
        if start_time.is_some() {
            query.push_str(" AND end_time >= ?");
        }
        if end_time.is_some() {
            query.push_str(" AND start_time < ?");
        }

        query.push_str(" ORDER BY start_time ASC");

        let mut query_builder = sqlx::query_as::<_, TimeBlockRow>(&query);

        if let Some(start) = start_time {
            query_builder = query_builder.bind(start.to_rfc3339());
        }
        if let Some(end) = end_time {
            query_builder = query_builder.bind(end.to_rfc3339());
        }

        let rows = query_builder.fetch_all(pool).await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        let blocks: Result<Vec<TimeBlock>, _> = rows.into_iter().map(TimeBlock::try_from).collect();

        blocks.map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
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
            ORDER BY t.created_at ASC
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
}

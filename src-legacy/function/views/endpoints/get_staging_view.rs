/// 获取Staging视图 API - 单文件组件
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::Task,
    crate::infra::core::{AppError, AppResult, ValidationError},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `get_staging_view`

## API端点
GET /api/views/staging?filter=floating
GET /api/views/staging?filter=project::{id}

## 预期行为简介
获取所有未被安排的任务（Staging区），可按项目或浮动任务过滤。

## 输入输出规范
- **前置条件**: 查询参数 `filter` 可选，值为 "floating" 或 "project::{uuid}"。
- **后置条件**: 返回 `200 OK` 和任务数组。
- **不变量**: 无。

## 边界情况
- 没有任务: 返回空数组。
- filter 格式无效: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 无副作用（只读操作）。
*/

#[derive(Deserialize)]
pub struct StagingQuery {
    filter: Option<String>,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<StagingQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(tasks) => Json(tasks).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub enum StagingFilter {
        All,
        Floating,
        Project(Uuid),
    }

    pub fn validate_query(query: &StagingQuery) -> Result<StagingFilter, Vec<ValidationError>> {
        if let Some(ref filter) = query.filter {
            if filter == "floating" {
                return Ok(StagingFilter::Floating);
            } else if filter.starts_with("project::") {
                let project_id_str = &filter[9..];
                match Uuid::parse_str(project_id_str) {
                    Ok(id) => return Ok(StagingFilter::Project(id)),
                    Err(_) => {
                        return Err(vec![ValidationError::new(
                            "filter",
                            "项目ID格式无效",
                            "INVALID_PROJECT_ID",
                        )])
                    }
                }
            } else {
                return Err(vec![ValidationError::new(
                    "filter",
                    "过滤器格式无效，应为 'floating' 或 'project::{uuid}'",
                    "INVALID_FILTER",
                )]);
            }
        }

        Ok(StagingFilter::All)
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, query: StagingQuery) -> AppResult<Vec<Task>> {
        let filter = validation::validate_query(&query).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        let tasks = database::find_staging_tasks_in_tx(&mut tx, &filter).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(tasks)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::task::TaskRow;

    pub async fn find_staging_tasks_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        filter: &validation::StagingFilter,
    ) -> AppResult<Vec<Task>> {
        let (where_clause, context_type, context_id) = match filter {
            validation::StagingFilter::All => ("", "MISC", "".to_string()),
            validation::StagingFilter::Floating => (
                "AND t.project_id IS NULL",
                "MISC",
                "".to_string(),
            ),
            validation::StagingFilter::Project(project_id) => (
                "AND t.project_id = ?",
                "PROJECT_LIST",
                format!("project::{}", project_id),
            ),
        };

        let query_str = format!(
            r#"
            SELECT DISTINCT t.id, t.title, t.glance_note, t.detail_note, t.estimated_duration,
                   t.subtasks, t.project_id, t.area_id, t.due_date, t.due_date_type, t.completed_at,
                   t.created_at, t.updated_at, t.is_deleted, t.source_info,
                   t.external_source_id, t.external_source_provider, t.external_source_metadata,
                   t.recurrence_rule, t.recurrence_parent_id, t.recurrence_original_date, t.recurrence_exclusions
            FROM tasks t
            LEFT JOIN task_schedules ts ON t.id = ts.task_id
            LEFT JOIN ordering o ON t.id = o.task_id AND o.context_type = ? AND o.context_id = ?
            WHERE ts.id IS NULL
            AND t.deleted_at IS NULL
            {}
            ORDER BY COALESCE(o.sort_order, 'z')
            "#,
            where_clause
        );

        let mut query = sqlx::query_as::<_, TaskRow>(&query_str)
            .bind(context_type)
            .bind(context_id);

        if let validation::StagingFilter::Project(project_id) = filter {
            query = query.bind(project_id.to_string());
        }

        let rows = query
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        rows.into_iter()
            .map(|r| {
                Task::try_from(r)
                    .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
            })
            .collect()
    }
}

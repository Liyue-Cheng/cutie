/// 获取指定日期的任务视图 - 单文件组件
///
/// GET /api/views/daily/:date
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use chrono::{NaiveDate, Utc};
use serde::Serialize;

use crate::{
    entities::{Task, TaskCardDto},
    features::views::shared::ViewTaskCardAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_daily_tasks`

## API端点
GET /api/views/daily/:date

## 预期行为简介
获取指定日期的所有已安排任务（scheduled），包括已完成和未完成的任务。

## 业务逻辑
1. 解析日期参数（YYYY-MM-DD）
2. 查询该日期的所有 task_schedules
3. 获取对应的 tasks（未删除，包括已完成）
4. 组装 TaskCard（包含完整的上下文信息）
5. 返回任务列表

## 输入输出规范
- **前置条件**:
  - date 格式为 YYYY-MM-DD
- **后置条件**:
  - 返回该日期的所有 scheduled 任务

## 边界情况
- 日期格式错误 → 400
- 该日期没有任务 → 返回空数组
*/

// ==================== 响应结构体 ====================
#[derive(Debug, Serialize)]
pub struct GetDailyTasksResponse {
    pub tasks: Vec<TaskCardDto>,
    pub date: String,
    pub count: usize,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(date_str): Path<String>) -> Response {
    match logic::execute(&app_state, &date_str).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn parse_date(date_str: &str) -> AppResult<chrono::DateTime<Utc>> {
        let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            AppError::validation_error(
                "date",
                "日期格式错误，请使用 YYYY-MM-DD 格式",
                "INVALID_DATE_FORMAT",
            )
        })?;

        let datetime = naive_date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::validation_error("date", "无效的日期", "INVALID_DATE"))?
            .and_utc();

        Ok(datetime)
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, date_str: &str) -> AppResult<GetDailyTasksResponse> {
        // 1. 解析日期
        let target_date = validation::parse_date(date_str)?;

        // 2. 查询该日期的所有任务
        let tasks = database::find_tasks_for_date(app_state.db_pool(), target_date).await?;

        tracing::info!("Found {} tasks for date {}", tasks.len(), date_str);

        // 3. 组装完整的 TaskCards（使用共享装配器）
        let task_cards = ViewTaskCardAssembler::assemble_batch(tasks, app_state.db_pool()).await?;

        // 4. 返回结果
        Ok(GetDailyTasksResponse {
            count: task_cards.len(),
            date: date_str.to_string(),
            tasks: task_cards,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    /// 查询指定日期的所有任务
    pub async fn find_tasks_for_date(
        pool: &sqlx::SqlitePool,
        target_date: chrono::DateTime<Utc>,
    ) -> AppResult<Vec<Task>> {
        let query = r#"
            SELECT DISTINCT t.id, t.title, t.glance_note, t.detail_note, t.estimated_duration,
                   t.subtasks, t.project_id, t.area_id, t.due_date, t.due_date_type, t.completed_at,
                   t.created_at, t.updated_at, t.is_deleted, t.source_info,
                   t.external_source_id, t.external_source_provider, t.external_source_metadata,
                   t.recurrence_rule, t.recurrence_parent_id, t.recurrence_original_date, t.recurrence_exclusions
            FROM tasks t
            INNER JOIN task_schedules ts ON ts.task_id = t.id
            WHERE DATE(ts.scheduled_day) = DATE(?)
              AND t.is_deleted = false
            ORDER BY t.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, crate::entities::TaskRow>(query)
            .bind(target_date.to_rfc3339())
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        let tasks = rows
            .into_iter()
            .map(|row| {
                Task::try_from(row).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })
            })
            .collect::<AppResult<Vec<Task>>>()?;

        Ok(tasks)
    }
}

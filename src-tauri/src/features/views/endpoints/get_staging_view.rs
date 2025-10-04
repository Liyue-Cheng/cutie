use crate::{
    entities::{ScheduleStatus, Task, TaskCardDto},
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};
/// 获取 Staging 视图 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

// ==================== 文档层 ====================
/*
CABC for `get_staging_view`

## API端点
GET /api/views/staging

## 预期行为简介
返回所有未排期（staging）的任务列表。

## 输入输出规范
- **前置条件**: 无
- **后置条件**:
  - 返回所有未删除、未完成、且未被安排到任何日期的任务
  - 每个任务包含完整的 TaskCard 信息

## 边界情况
- 如果没有 staging 任务，返回空数组

## 预期副作用
- 无（只读操作）

## 请求/响应示例
Response: 200 OK
[
  {
    "id": "...",
    "title": "未排期的任务",
    "schedule_status": "staging",
    ...
  }
]
*/

// ==================== HTTP 处理器 ====================
/// 获取 Staging 视图的 HTTP 处理器
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(task_cards) => success_response(task_cards).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<TaskCardDto>> {
        // 只读操作，不需要事务，直接使用连接池
        let pool = app_state.db_pool();

        // 1. 获取所有 staging 任务
        let tasks = database::find_staging_tasks(pool).await?;

        // 2. 为每个任务获取额外信息并组装成 TaskCardDto
        let mut task_cards = Vec::new();
        for task in tasks {
            let task_card = assemble_task_card(&task, pool).await?;
            task_cards.push(task_card);
        }

        Ok(task_cards)
    }

    /// 组装单个任务的 TaskCard（包含完整的 schedules + time_blocks）
    async fn assemble_task_card(task: &Task, pool: &sqlx::SqlitePool) -> AppResult<TaskCardDto> {
        // 1. 创建基础 TaskCard
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 2. 组装完整的 schedules（对于 staging 任务应该是 None）
        let schedules = TaskAssembler::assemble_schedules(pool, task.id).await?;
        card.schedules = schedules;

        // 3. 设置 schedule_status 为 staging
        card.schedule_status = ScheduleStatus::Staging;

        Ok(card)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    /// 查询所有 staging 任务
    ///
    /// 条件：
    /// - is_deleted = false
    /// - completed_at IS NULL
    /// - 不存在于 task_schedules 表中
    pub async fn find_staging_tasks(pool: &sqlx::SqlitePool) -> AppResult<Vec<Task>> {
        let query = r#"
            SELECT 
                t.id, t.title, t.glance_note, t.detail_note, t.estimated_duration, 
                t.subtasks, t.project_id, t.area_id, t.due_date, t.due_date_type, 
                t.completed_at, t.created_at, t.updated_at, t.is_deleted, t.source_info,
                t.external_source_id, t.external_source_provider, t.external_source_metadata,
                t.recurrence_rule, t.recurrence_parent_id, t.recurrence_original_date, 
                t.recurrence_exclusions
            FROM tasks t
            WHERE t.is_deleted = false
              AND t.completed_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM task_schedules ts 
                  WHERE ts.task_id = t.id
              )
            ORDER BY t.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TaskRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let tasks: Result<Vec<Task>, _> = rows.into_iter().map(Task::try_from).collect();

        tasks.map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}

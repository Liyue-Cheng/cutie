/// Task 装配器 (Assembler)
///
/// 职责：将数据库实体（Task）和其他相关数据组装成响应 DTO
///
/// 装配器模式的优势：
/// - entities 层保持纯粹的数据结构定义
/// - 业务逻辑集中在 features 层
/// - 易于测试和维护
use crate::entities::task::TaskScheduleDto;
use crate::entities::{
    DueDateInfo, DueDateType, ProjectSummary, ScheduleInfo, ScheduleStatus, SubtaskDto, Task,
    TaskCardDto, TaskDetailDto,
};
use crate::infra::core::AppResult;
use uuid::Uuid;

/// Task 装配器
pub struct TaskAssembler;

impl TaskAssembler {
    /// 从 Task 实体创建 TaskCardDto（基础版本）
    ///
    /// 只填充可以直接从 Task 实体获取的字段
    /// 需要额外数据的字段保持默认值，调用者需要手动设置：
    /// - schedule_status（需要从 Schedule 表判断）
    /// - schedule_info（需要从 Schedule 表计算）
    pub fn task_to_card_basic(task: &Task) -> TaskCardDto {
        TaskCardDto {
            id: task.id,
            title: task.title.clone(),
            glance_note: task.glance_note.clone(),
            is_completed: task.is_completed(),
            is_archived: task.is_archived(),
            is_deleted: task.is_deleted(),
            deleted_at: task.deleted_at,
            schedule_status: ScheduleStatus::Staging, // 默认 Staging，需要后续判断
            subtasks: task.subtasks.as_ref().map(|subtasks| {
                subtasks
                    .iter()
                    .map(|s| SubtaskDto::from(s.clone()))
                    .collect()
            }),
            estimated_duration: task.estimated_duration,
            area_id: task.area_id, // ✅ 直接传递 area_id，前端从 area store 获取完整信息
            project_id: task.project_id,
            schedule_info: None, // 需要后续填充
            due_date: task.due_date.map(|date| {
                // 使用本地时区进行逾期判断
                use chrono::Local;
                let now_local = Local::now().date_naive();

                DueDateInfo {
                    date,
                    due_date_type: task.due_date_type.clone().unwrap_or(DueDateType::Soft),
                    is_overdue: now_local > date,
                }
            }),
            schedules: None, // 需要后续填充（调用 assemble_schedules）
            has_detail_note: task.detail_note.is_some(),
            recurrence_id: task.recurrence_id,
            recurrence_original_date: task.recurrence_original_date.clone(),
            recurrence_expiry_behavior: None, // 需要后续填充（调用 fill_recurrence_expiry_behavior）
        }
    }

    /// 填充 TaskCardDto 的 recurrence_expiry_behavior 字段（从数据库查询）
    pub async fn fill_recurrence_expiry_behavior(
        card: &mut TaskCardDto,
        pool: &sqlx::SqlitePool,
    ) -> AppResult<()> {
        if let Some(recurrence_id) = card.recurrence_id {
            let query = r#"
                SELECT expiry_behavior
                FROM task_recurrences
                WHERE id = ?
            "#;

            let expiry_behavior: Option<String> = sqlx::query_scalar(query)
                .bind(recurrence_id.to_string())
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    crate::infra::core::AppError::DatabaseError(
                        crate::infra::core::DbError::ConnectionError(e),
                    )
                })?;

            card.recurrence_expiry_behavior = expiry_behavior;
        }
        Ok(())
    }

    /// 批量填充 TaskCardDto 的 recurrence_expiry_behavior 字段
    pub async fn fill_recurrence_expiry_behavior_batch(
        cards: &mut [TaskCardDto],
        pool: &sqlx::SqlitePool,
    ) -> AppResult<()> {
        // 收集所有需要查询的 recurrence_id
        let recurrence_ids: Vec<Uuid> =
            cards.iter().filter_map(|card| card.recurrence_id).collect();

        if recurrence_ids.is_empty() {
            return Ok(());
        }

        // 批量查询所有 recurrence 的 expiry_behavior
        let placeholders = recurrence_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "SELECT id, expiry_behavior FROM task_recurrences WHERE id IN ({})",
            placeholders
        );

        let mut q = sqlx::query_as::<_, (String, String)>(&query);
        for id in &recurrence_ids {
            q = q.bind(id.to_string());
        }

        let results: Vec<(String, String)> = q.fetch_all(pool).await.map_err(|e| {
            crate::infra::core::AppError::DatabaseError(
                crate::infra::core::DbError::ConnectionError(e),
            )
        })?;

        // 构建 id -> expiry_behavior 的映射
        let expiry_map: std::collections::HashMap<Uuid, String> = results
            .into_iter()
            .filter_map(|(id_str, expiry_behavior)| {
                Uuid::parse_str(&id_str)
                    .ok()
                    .map(|id| (id, expiry_behavior))
            })
            .collect();

        // 填充每个 card
        for card in cards.iter_mut() {
            if let Some(recurrence_id) = card.recurrence_id {
                card.recurrence_expiry_behavior = expiry_map.get(&recurrence_id).cloned();
            }
        }

        Ok(())
    }

    /// 在事务中组装任务的 schedules（包含 time_blocks）
    ///
    /// 用于在事务内填充 TaskCardDto.schedules 字段
    pub async fn assemble_schedules_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<Vec<TaskScheduleDto>>> {
        use crate::entities::task::{DailyOutcome, TaskScheduleDto, TimeBlockSummary};
        use crate::entities::Outcome;
        use crate::infra::core::{AppError, AppResult as Result, DbError};
        use chrono::{DateTime, Utc};
        use uuid::Uuid as UuidType;

        // 1. 查询所有日程
        let schedule_query = r#"
            SELECT id, task_id, scheduled_date, outcome, created_at, updated_at
            FROM task_schedules
            WHERE task_id = ?
            ORDER BY scheduled_date ASC
        "#;

        let schedule_rows = sqlx::query_as::<_, crate::entities::TaskScheduleRow>(schedule_query)
            .bind(task_id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        if schedule_rows.is_empty() {
            return Ok(None); // staging 任务
        }

        // 2. 转换为 TaskSchedule 实体
        let schedules: Vec<crate::entities::TaskSchedule> = schedule_rows
            .into_iter()
            .map(|row| {
                crate::entities::TaskSchedule::try_from(row)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
            })
            .collect::<Result<Vec<_>>>()?;

        // 3. 为每个日程查询时间片
        let mut schedule_dtos = Vec::new();

        // 2.5 查询该任务的所有时间块（不按日期过滤，后面在代码中按本地日期分组）
        let all_blocks_query = r#"
            SELECT tb.id, tb.title, tb.glance_note, tb.start_time, tb.end_time
            FROM time_blocks tb
            INNER JOIN task_time_block_links ttbl ON ttbl.time_block_id = tb.id
            WHERE ttbl.task_id = ?
              AND tb.is_deleted = false
            ORDER BY tb.start_time ASC
        "#;

        let all_block_rows: Vec<(
            String,
            Option<String>,
            Option<String>,
            DateTime<Utc>,
            DateTime<Utc>,
        )> = sqlx::query_as(all_blocks_query)
            .bind(task_id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        // 将时间块按本地日期分组
        use crate::infra::core::utils::time_utils;
        let mut blocks_by_date: std::collections::HashMap<String, Vec<TimeBlockSummary>> =
            std::collections::HashMap::new();

        for (id_str, title, glance_note, start_time, end_time) in all_block_rows {
            let id =
                UuidType::parse_str(&id_str).map_err(|e| AppError::StringError(e.to_string()))?;
            // 使用系统本地时区提取日期
            use chrono::Local;
            let local_start = start_time.with_timezone(&Local);
            let formatted_date = time_utils::format_date_yyyy_mm_dd(&local_start.date_naive());

            blocks_by_date
                .entry(formatted_date)
                .or_insert_with(Vec::new)
                .push(TimeBlockSummary {
                    id,
                    start_time,
                    end_time,
                    title,
                    glance_note,
                });
        }

        for schedule in schedules {
            let scheduled_day_str = &schedule.scheduled_date;

            // 获取该日期的时间块（使用字符串日期匹配）
            let time_blocks = blocks_by_date
                .get(&schedule.scheduled_date)
                .cloned()
                .unwrap_or_default();

            // 转换 Outcome 为 DailyOutcome
            let outcome = match schedule.outcome {
                Outcome::Planned => DailyOutcome::Planned,
                Outcome::PresenceLogged => DailyOutcome::PresenceLogged,
                Outcome::CompletedOnDay => DailyOutcome::Completed,
                Outcome::CarriedOver => DailyOutcome::CarriedOver,
            };

            schedule_dtos.push(TaskScheduleDto {
                scheduled_day: scheduled_day_str.to_string(),
                outcome,
                time_blocks,
            });
        }

        Ok(Some(schedule_dtos))
    }

    /// 使用连接池组装任务的 schedules（包含 time_blocks）
    ///
    /// 用于在事务外填充 TaskCardDto.schedules 字段
    pub async fn assemble_schedules(
        pool: &sqlx::SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Option<Vec<TaskScheduleDto>>> {
        use crate::entities::task::{DailyOutcome, TaskScheduleDto, TimeBlockSummary};
        use crate::entities::Outcome;
        use crate::infra::core::{AppError, AppResult as Result, DbError};
        use chrono::{DateTime, Utc};
        use uuid::Uuid as UuidType;

        // 1. 查询所有日程
        let schedule_query = r#"
            SELECT id, task_id, scheduled_date, outcome, created_at, updated_at
            FROM task_schedules
            WHERE task_id = ?
            ORDER BY scheduled_date ASC
        "#;

        let schedule_rows = sqlx::query_as::<_, crate::entities::TaskScheduleRow>(schedule_query)
            .bind(task_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        if schedule_rows.is_empty() {
            return Ok(None); // staging 任务
        }

        // 2. 转换为 TaskSchedule 实体
        let schedules: Vec<crate::entities::TaskSchedule> = schedule_rows
            .into_iter()
            .map(|row| {
                crate::entities::TaskSchedule::try_from(row)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
            })
            .collect::<Result<Vec<_>>>()?;

        // 3. 为每个日程查询时间片
        let mut schedule_dtos = Vec::new();

        // 2.5 查询该任务的所有时间块（不按日期过滤，后面在代码中按本地日期分组）
        let all_blocks_query = r#"
            SELECT tb.id, tb.title, tb.glance_note, tb.start_time, tb.end_time
            FROM time_blocks tb
            INNER JOIN task_time_block_links ttbl ON ttbl.time_block_id = tb.id
            WHERE ttbl.task_id = ?
              AND tb.is_deleted = false
            ORDER BY tb.start_time ASC
        "#;

        let all_block_rows: Vec<(
            String,
            Option<String>,
            Option<String>,
            DateTime<Utc>,
            DateTime<Utc>,
        )> = sqlx::query_as(all_blocks_query)
            .bind(task_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        // 将时间块按本地日期分组
        use crate::infra::core::utils::time_utils;
        let mut blocks_by_date: std::collections::HashMap<String, Vec<TimeBlockSummary>> =
            std::collections::HashMap::new();

        for (id_str, title, glance_note, start_time, end_time) in all_block_rows {
            let id =
                UuidType::parse_str(&id_str).map_err(|e| AppError::StringError(e.to_string()))?;
            // 使用系统本地时区提取日期
            use chrono::Local;
            let local_start = start_time.with_timezone(&Local);
            let formatted_date = time_utils::format_date_yyyy_mm_dd(&local_start.date_naive());

            blocks_by_date
                .entry(formatted_date)
                .or_insert_with(Vec::new)
                .push(TimeBlockSummary {
                    id,
                    start_time,
                    end_time,
                    title,
                    glance_note,
                });
        }

        for schedule in schedules {
            let scheduled_day_str = &schedule.scheduled_date;

            // 获取该日期的时间块（使用字符串日期匹配）
            let time_blocks = blocks_by_date
                .get(&schedule.scheduled_date)
                .cloned()
                .unwrap_or_default();

            // 转换 Outcome 为 DailyOutcome
            let outcome = match schedule.outcome {
                Outcome::Planned => DailyOutcome::Planned,
                Outcome::PresenceLogged => DailyOutcome::PresenceLogged,
                Outcome::CompletedOnDay => DailyOutcome::Completed,
                Outcome::CarriedOver => DailyOutcome::CarriedOver,
            };

            schedule_dtos.push(TaskScheduleDto {
                scheduled_day: scheduled_day_str.to_string(),
                outcome,
                time_blocks,
            });
        }

        Ok(Some(schedule_dtos))
    }

    /// 从 Task 实体创建完整的 TaskCardDto
    ///
    /// 包含所有必要的上下文信息
    #[allow(dead_code)]
    pub fn task_to_card_full(
        task: &Task,
        schedule_status: ScheduleStatus,
        schedule_info: Option<ScheduleInfo>,
    ) -> TaskCardDto {
        let mut card = Self::task_to_card_basic(task);
        card.schedule_status = schedule_status;
        card.schedule_info = schedule_info;
        card
    }

    /// 从 TaskCardDto 和 Task 实体创建 TaskDetailDto（基础版本）
    ///
    /// 需要额外数据的字段保持默认值：
    /// - project（需要从 Project 表获取）
    /// - schedules 已通过 flatten 从 TaskCardDto 继承
    pub fn card_and_task_to_detail_basic(card: TaskCardDto, task: &Task) -> TaskDetailDto {
        TaskDetailDto {
            card,
            detail_note: task.detail_note.clone(),
            // schedules 已通过 flatten 从 TaskCardDto 继承
            project: None, // 需要后续填充
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }

    /// 从 TaskCardDto 和额外信息创建完整的 TaskDetailDto
    #[allow(dead_code)]
    pub fn card_to_detail_full(
        card: TaskCardDto,
        task: &Task,
        project: Option<ProjectSummary>,
    ) -> TaskDetailDto {
        TaskDetailDto {
            card,
            detail_note: task.detail_note.clone(),
            // schedules 已通过 flatten 从 TaskCardDto 继承
            project,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }

    /// 从 Task 实体直接创建 TaskDetailDto（基础版本）
    ///
    /// 这是一个便捷方法，内部调用 task_to_card_basic 和 card_and_task_to_detail_basic
    #[allow(dead_code)]
    pub fn task_to_detail_basic(task: &Task) -> TaskDetailDto {
        let card = Self::task_to_card_basic(task);
        Self::card_and_task_to_detail_basic(card, task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_task() -> Task {
        Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            glance_note: Some("Test note".to_string()),
            detail_note: Some("Detailed note".to_string()),
            estimated_duration: Some(60),
            subtasks: None,
            project_id: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            completed_at: None,
            archived_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_id: None,
            recurrence_original_date: None,
        }
    }

    #[test]
    fn test_task_to_card_basic() {
        let task = create_test_task();
        let card = TaskAssembler::task_to_card_basic(&task);

        assert_eq!(card.id, task.id);
        assert_eq!(card.title, task.title);
        assert_eq!(card.glance_note, task.glance_note);
        assert!(!card.is_completed);
        assert_eq!(card.schedule_status, ScheduleStatus::Staging);
        assert!(card.has_detail_note);
    }

    #[test]
    fn test_task_to_detail_basic() {
        let task = create_test_task();
        let detail = TaskAssembler::task_to_detail_basic(&task);

        assert_eq!(detail.card.id, task.id);
        assert_eq!(detail.detail_note, task.detail_note);
        assert_eq!(detail.created_at, task.created_at);
        assert_eq!(detail.updated_at, task.updated_at);
    }
}

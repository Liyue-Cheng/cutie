/// Task Schedules 表操作仓库
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    entities::{DailyOutcome, ScheduleRecord},
    infra::core::{AppError, AppResult, DbError},
};

pub struct TaskScheduleRepository;

impl TaskScheduleRepository {
    /// 检查任务是否有任何日程记录（支持事务和非事务）
    pub async fn has_any_schedule(
        executor: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        task_id: Uuid,
    ) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_schedules
            WHERE task_id = ?
        "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .fetch_one(executor)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(count > 0)
    }

    /// 检查任务在某天是否有日程
    pub async fn has_schedule_for_day_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
    ) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_schedules
            WHERE task_id = ? AND scheduled_date = ?
        "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(count > 0)
    }

    /// 检查任务在某天是否有日程记录（非事务版本）
    pub async fn has_schedule_for_day(
        pool: &SqlitePool,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
    ) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_schedules
            WHERE task_id = ? AND scheduled_date = ?
        "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(count > 0)
    }

    /// 创建日程记录
    pub async fn create_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
    ) -> AppResult<()> {
        let schedule_id = Uuid::new_v4();
        let now = Utc::now();

        let query = r#"
            INSERT INTO task_schedules (id, task_id, scheduled_date, outcome, created_at, updated_at)
            VALUES (?, ?, ?, 'PLANNED', ?, ?)
        "#;

        sqlx::query(query)
            .bind(schedule_id.to_string())
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新当天日程为已完成
    pub async fn update_today_to_completed_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        use crate::infra::core::utils::time_utils;
        // ✅ 使用本地时间确定"今天"的日期，避免时区问题
        let local_now = chrono::Local::now();
        let today = time_utils::format_date_yyyy_mm_dd(&local_now.date_naive());
        let query = r#"
            UPDATE task_schedules
            SET outcome = 'COMPLETED_ON_DAY', updated_at = ?
            WHERE task_id = ? AND scheduled_date = ?
        "#;

        sqlx::query(query)
            .bind(now.to_rfc3339())
            .bind(task_id.to_string())
            .bind(today)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新指定日期的日程为已完成
    pub async fn update_day_to_completed_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
        completed_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE task_schedules
            SET outcome = 'COMPLETED_ON_DAY', updated_at = ?
            WHERE task_id = ? AND scheduled_date = ?
        "#;

        sqlx::query(query)
            .bind(completed_at.to_rfc3339())
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 删除未来日程
    pub async fn delete_future_schedules_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        _now: DateTime<Utc>, // 保留参数但不使用，使用本地时间代替
    ) -> AppResult<()> {
        use crate::infra::core::utils::time_utils;
        // ✅ 使用本地时间确定"今天"的日期，避免时区问题
        let local_now = chrono::Local::now();
        let today = time_utils::format_date_yyyy_mm_dd(&local_now.date_naive());
        let query = r#"
            DELETE FROM task_schedules
            WHERE task_id = ? AND scheduled_date > ?
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(today)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 删除指定日期之后的所有日程
    pub async fn delete_schedules_after_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_schedules
            WHERE task_id = ? AND scheduled_date > ?
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 删除指定日期的日程
    pub async fn delete_schedule_for_day_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_schedules
            WHERE task_id = ? AND scheduled_date = ?
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 删除任务的所有日程
    pub async fn delete_all_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM task_schedules WHERE task_id = ?";
        sqlx::query(query)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 获取任务的所有日程记录
    pub async fn get_all_for_task(
        pool: &SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Vec<ScheduleRecord>> {
        let query = r#"
            SELECT scheduled_date, outcome
            FROM task_schedules
            WHERE task_id = ?
            ORDER BY scheduled_date ASC
        "#;

        let rows = sqlx::query_as::<_, (String, String)>(query)
            .bind(task_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let schedules = rows
            .into_iter()
            .filter_map(|(day_str, outcome_str)| {
                // day_str 已经是 YYYY-MM-DD 字符串，直接使用
                use crate::infra::core::utils::time_utils;
                let day = time_utils::parse_date_yyyy_mm_dd(&day_str).ok()?;

                let outcome = match outcome_str.as_str() {
                    "PLANNED" => DailyOutcome::Planned,
                    "PRESENCE_LOGGED" => DailyOutcome::PresenceLogged,
                    "COMPLETED_ON_DAY" => DailyOutcome::Completed,
                    "CARRIED_OVER" => DailyOutcome::CarriedOver,
                    _ => return None,
                };

                Some(ScheduleRecord { day, outcome })
            })
            .collect();

        Ok(schedules)
    }
}

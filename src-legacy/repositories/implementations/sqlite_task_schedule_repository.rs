/// TaskScheduleRepository的SQLite实现
///
/// 提供TaskSchedule实体的具体数据库操作实现
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::entities::{Outcome, TaskSchedule};
use crate::repositories::traits::TaskScheduleRepository;
use crate::shared::core::{AppResult, DbError};

/// 任务日程仓库的SQLite实现
#[derive(Clone)]
pub struct SqliteTaskScheduleRepository {
    pool: SqlitePool,
}

impl SqliteTaskScheduleRepository {
    /// 创建新的TaskScheduleRepository实例
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为TaskSchedule对象
    fn row_to_task_schedule(row: &sqlx::sqlite::SqliteRow) -> Result<TaskSchedule, sqlx::Error> {
        let outcome_str: String = row.try_get("outcome")?;
        let outcome = match outcome_str.as_str() {
            "PLANNED" => Outcome::Planned,
            "PRESENCE_LOGGED" => Outcome::PresenceLogged,
            "COMPLETED_ON_DAY" => Outcome::CompletedOnDay,
            "CARRIED_OVER" => Outcome::CarriedOver,
            _ => {
                return Err(sqlx::Error::ColumnDecode {
                    index: "outcome".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid outcome value",
                    )),
                })
            }
        };

        Ok(TaskSchedule {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|_| {
                sqlx::Error::ColumnDecode {
                    index: "id".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid UUID",
                    )),
                }
            })?,
            task_id: Uuid::parse_str(&row.try_get::<String, _>("task_id")?).map_err(|_| {
                sqlx::Error::ColumnDecode {
                    index: "task_id".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid UUID",
                    )),
                }
            })?,
            scheduled_day: DateTime::parse_from_rfc3339(
                &row.try_get::<String, _>("scheduled_day")?,
            )
            .map_err(|_| sqlx::Error::ColumnDecode {
                index: "scheduled_day".to_string(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid datetime",
                )),
            })?
            .with_timezone(&Utc),
            outcome,
            created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "created_at".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "updated_at".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
        })
    }

    /// 将Outcome转换为数据库字符串
    fn outcome_to_string(outcome: &Outcome) -> &'static str {
        match outcome {
            Outcome::Planned => "PLANNED",
            Outcome::PresenceLogged => "PRESENCE_LOGGED",
            Outcome::CompletedOnDay => "COMPLETED_ON_DAY",
            Outcome::CarriedOver => "CARRIED_OVER",
        }
    }
}

#[async_trait]
impl TaskScheduleRepository for SqliteTaskScheduleRepository {
    // --- 写操作 ---
    async fn create(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        schedule: &TaskSchedule,
    ) -> AppResult<TaskSchedule> {
        sqlx::query(
            r#"
            INSERT INTO task_schedule (
                id, task_id, scheduled_day, outcome, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(schedule.id.to_string())
        .bind(schedule.task_id.to_string())
        .bind(schedule.scheduled_day.to_rfc3339())
        .bind(Self::outcome_to_string(&schedule.outcome))
        .bind(schedule.created_at.to_rfc3339())
        .bind(schedule.updated_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(schedule.clone())
    }

    async fn update_outcome(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        schedule_id: Uuid,
        new_outcome: Outcome,
    ) -> AppResult<TaskSchedule> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE task_schedule SET 
                outcome = ?, 
                updated_at = ? 
            WHERE id = ?
            "#,
        )
        .bind(Self::outcome_to_string(&new_outcome))
        .bind(now.to_rfc3339())
        .bind(schedule_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "TaskSchedule",
                schedule_id.to_string(),
            ));
        }

        // 返回更新后的任务日程
        let row = sqlx::query("SELECT * FROM task_schedule WHERE id = ?")
            .bind(schedule_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::ConnectionError)?;

        Self::row_to_task_schedule(&row).map_err(|e| DbError::ConnectionError(e).into())
    }

    async fn reschedule(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        schedule_id: Uuid,
        new_day: DateTime<Utc>,
    ) -> AppResult<TaskSchedule> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE task_schedule SET 
                scheduled_day = ?, 
                outcome = ?,
                updated_at = ? 
            WHERE id = ?
            "#,
        )
        .bind(new_day.to_rfc3339())
        .bind(Self::outcome_to_string(&Outcome::Planned)) // 重置为计划状态
        .bind(now.to_rfc3339())
        .bind(schedule_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "TaskSchedule",
                schedule_id.to_string(),
            ));
        }

        // 返回更新后的任务日程
        let row = sqlx::query("SELECT * FROM task_schedule WHERE id = ?")
            .bind(schedule_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::ConnectionError)?;

        Self::row_to_task_schedule(&row).map_err(|e| DbError::ConnectionError(e).into())
    }

    async fn delete(&self, tx: &mut Transaction<'_, Sqlite>, schedule_id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM task_schedule WHERE id = ?")
            .bind(schedule_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "TaskSchedule",
                schedule_id.to_string(),
            ));
        }

        Ok(())
    }

    async fn delete_all_for_task(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM task_schedule WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(())
    }

    async fn delete_future_for_task(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        since: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM task_schedule WHERE task_id = ? AND scheduled_day > ?")
            .bind(task_id.to_string())
            .bind(since.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(())
    }

    // --- 读操作 ---
    async fn find_by_day(&self, day: DateTime<Utc>) -> AppResult<Vec<TaskSchedule>> {
        let rows = sqlx::query(
            "SELECT * FROM task_schedule WHERE scheduled_day = ? ORDER BY created_at ASC",
        )
        .bind(day.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let mut schedules = Vec::new();
        for row in rows {
            let schedule = Self::row_to_task_schedule(&row).map_err(DbError::ConnectionError)?;
            schedules.push(schedule);
        }

        Ok(schedules)
    }

    async fn find_all_for_task(&self, task_id: Uuid) -> AppResult<Vec<TaskSchedule>> {
        let rows =
            sqlx::query("SELECT * FROM task_schedule WHERE task_id = ? ORDER BY scheduled_day ASC")
                .bind(task_id.to_string())
                .fetch_all(&self.pool)
                .await
                .map_err(DbError::ConnectionError)?;

        let mut schedules = Vec::new();
        for row in rows {
            let schedule = Self::row_to_task_schedule(&row).map_err(DbError::ConnectionError)?;
            schedules.push(schedule);
        }

        Ok(schedules)
    }
}

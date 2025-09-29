use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::{ScheduleCountByOutcome, TaskScheduleRepository, Transaction};
use crate::common::error::DbError;
use crate::core::models::{Outcome, TaskSchedule};

/// TaskScheduleRepository的SQLx实现
pub struct SqlxTaskScheduleRepository {
    pool: SqlitePool,
}

impl SqlxTaskScheduleRepository {
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
                        "Invalid outcome",
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

    /// 将TaskSchedule对象转换为数据库参数
    fn task_schedule_to_params(
        schedule: &TaskSchedule,
    ) -> (String, String, String, String, String, String) {
        let outcome_str = match schedule.outcome {
            Outcome::Planned => "PLANNED",
            Outcome::PresenceLogged => "PRESENCE_LOGGED",
            Outcome::CompletedOnDay => "COMPLETED_ON_DAY",
            Outcome::CarriedOver => "CARRIED_OVER",
        };

        (
            schedule.id.to_string(),
            schedule.task_id.to_string(),
            schedule.scheduled_day.to_rfc3339(),
            outcome_str.to_string(),
            schedule.created_at.to_rfc3339(),
            schedule.updated_at.to_rfc3339(),
        )
    }
}

#[async_trait]
impl TaskScheduleRepository for SqlxTaskScheduleRepository {
    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        schedule: &TaskSchedule,
    ) -> Result<TaskSchedule, DbError> {
        let params = Self::task_schedule_to_params(schedule);

        let result = sqlx::query(
            r#"
            INSERT INTO task_schedules (id, task_id, scheduled_day, outcome, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.0)
        .bind(&params.1)
        .bind(&params.2)
        .bind(&params.3)
        .bind(&params.4)
        .bind(&params.5)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(_) => Ok(schedule.clone()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(DbError::ConstraintViolation {
                    message: format!(
                        "Task {} already has a schedule for day {}",
                        schedule.task_id, schedule.scheduled_day
                    ),
                })
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn update_outcome(
        &self,
        tx: &mut Transaction<'_>,
        schedule_id: Uuid,
        new_outcome: Outcome,
    ) -> Result<TaskSchedule, DbError> {
        let outcome_str = match new_outcome {
            Outcome::Planned => "PLANNED",
            Outcome::PresenceLogged => "PRESENCE_LOGGED",
            Outcome::CompletedOnDay => "COMPLETED_ON_DAY",
            Outcome::CarriedOver => "CARRIED_OVER",
        };

        let result =
            sqlx::query("UPDATE task_schedules SET outcome = ?, updated_at = ? WHERE id = ?")
                .bind(outcome_str)
                .bind(Utc::now().to_rfc3339())
                .bind(schedule_id.to_string())
                .execute(&mut **tx)
                .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "TaskSchedule".to_string(),
                        entity_id: schedule_id.to_string(),
                    })
                } else {
                    // 重新查询更新后的日程
                    let schedule_result = sqlx::query("SELECT * FROM task_schedules WHERE id = ?")
                        .bind(schedule_id.to_string())
                        .fetch_one(&mut **tx)
                        .await;

                    match schedule_result {
                        Ok(row) => {
                            Ok(Self::row_to_task_schedule(&row)
                                .map_err(DbError::ConnectionError)?)
                        }
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn reschedule(
        &self,
        tx: &mut Transaction<'_>,
        schedule_id: Uuid,
        new_day: DateTime<Utc>,
    ) -> Result<TaskSchedule, DbError> {
        let result = sqlx::query(
            "UPDATE task_schedules SET scheduled_day = ?, outcome = 'PLANNED', updated_at = ? WHERE id = ?"
        )
        .bind(new_day.to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .bind(schedule_id.to_string())
        .execute(&mut **tx)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "TaskSchedule".to_string(),
                        entity_id: schedule_id.to_string(),
                    })
                } else {
                    // 重新查询更新后的日程
                    let schedule_result = sqlx::query("SELECT * FROM task_schedules WHERE id = ?")
                        .bind(schedule_id.to_string())
                        .fetch_one(&mut **tx)
                        .await;

                    match schedule_result {
                        Ok(row) => {
                            Ok(Self::row_to_task_schedule(&row)
                                .map_err(DbError::ConnectionError)?)
                        }
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn delete(&self, tx: &mut Transaction<'_>, schedule_id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM task_schedules WHERE id = ?")
            .bind(schedule_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn delete_all_for_task(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
    ) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM task_schedules WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn delete_future_for_task(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
        since: DateTime<Utc>,
    ) -> Result<(), DbError> {
        let result =
            sqlx::query("DELETE FROM task_schedules WHERE task_id = ? AND scheduled_day > ?")
                .bind(task_id.to_string())
                .bind(since.to_rfc3339())
                .execute(&mut **tx)
                .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_day(&self, day: DateTime<Utc>) -> Result<Vec<TaskSchedule>, DbError> {
        let result = sqlx::query(
            "SELECT * FROM task_schedules WHERE scheduled_day = ? ORDER BY updated_at DESC",
        )
        .bind(day.to_rfc3339())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let schedules: Result<Vec<TaskSchedule>, _> = rows
                    .iter()
                    .map(|row| Self::row_to_task_schedule(row))
                    .collect();
                schedules.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_all_for_task(&self, task_id: Uuid) -> Result<Vec<TaskSchedule>, DbError> {
        let result = sqlx::query(
            "SELECT * FROM task_schedules WHERE task_id = ? ORDER BY scheduled_day ASC",
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let schedules: Result<Vec<TaskSchedule>, _> = rows
                    .iter()
                    .map(|row| Self::row_to_task_schedule(row))
                    .collect();
                schedules.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TaskSchedule>, DbError> {
        let result = sqlx::query(
            "SELECT * FROM task_schedules WHERE scheduled_day >= ? AND scheduled_day <= ? ORDER BY scheduled_day ASC"
        )
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let schedules: Result<Vec<TaskSchedule>, _> = rows
                    .iter()
                    .map(|row| Self::row_to_task_schedule(row))
                    .collect();
                schedules.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_outcome(
        &self,
        outcome: Outcome,
        limit: Option<i64>,
    ) -> Result<Vec<TaskSchedule>, DbError> {
        let outcome_str = match outcome {
            Outcome::Planned => "PLANNED",
            Outcome::PresenceLogged => "PRESENCE_LOGGED",
            Outcome::CompletedOnDay => "COMPLETED_ON_DAY",
            Outcome::CarriedOver => "CARRIED_OVER",
        };

        let limit = limit.unwrap_or(100);

        let result = sqlx::query(
            "SELECT * FROM task_schedules WHERE outcome = ? ORDER BY scheduled_day DESC LIMIT ?",
        )
        .bind(outcome_str)
        .bind(limit)
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let schedules: Result<Vec<TaskSchedule>, _> = rows
                    .iter()
                    .map(|row| Self::row_to_task_schedule(row))
                    .collect();
                schedules.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn count_by_outcome(&self) -> Result<ScheduleCountByOutcome, DbError> {
        let result = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN outcome = 'PLANNED' THEN 1 END) as planned,
                COUNT(CASE WHEN outcome = 'PRESENCE_LOGGED' THEN 1 END) as presence_logged,
                COUNT(CASE WHEN outcome = 'COMPLETED_ON_DAY' THEN 1 END) as completed_on_day,
                COUNT(CASE WHEN outcome = 'CARRIED_OVER' THEN 1 END) as carried_over
            FROM task_schedules
            "#,
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => Ok(ScheduleCountByOutcome {
                total: row.try_get("total").map_err(DbError::ConnectionError)?,
                planned: row.try_get("planned").map_err(DbError::ConnectionError)?,
                presence_logged: row
                    .try_get("presence_logged")
                    .map_err(DbError::ConnectionError)?,
                completed_on_day: row
                    .try_get("completed_on_day")
                    .map_err(DbError::ConnectionError)?,
                carried_over: row
                    .try_get("carried_over")
                    .map_err(DbError::ConnectionError)?,
            }),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }
}

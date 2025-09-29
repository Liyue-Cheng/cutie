/// 日程数据访问层
///
/// 实现日程的数据库操作

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::shared::{
    core::{AppResult, DbError, Outcome, TaskSchedule},
    database::{Repository, TaskScheduleRepository},
};

/// 任务日程仓库的SQLx实现
#[derive(Clone)]
pub struct SqlxTaskScheduleRepository {
    pool: SqlitePool,
}

impl SqlxTaskScheduleRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为TaskSchedule对象
    fn row_to_schedule(row: &sqlx::sqlite::SqliteRow) -> Result<TaskSchedule, sqlx::Error> {
        let outcome_str: String = row.try_get("outcome")?;
        let outcome = match outcome_str.as_str() {
            "PLANNED" => Outcome::Planned,
            "PRESENCE_LOGGED" => Outcome::PresenceLogged,
            "COMPLETED_ON_DAY" => Outcome::CompletedOnDay,
            "CARRIED_OVER" => Outcome::CarriedOver,
            _ => Outcome::Planned, // 默认值
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
            scheduled_day: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("scheduled_day")?)
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
    fn schedule_to_params(schedule: &TaskSchedule) -> (String, String, String, String, String, String) {
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
impl Repository<TaskSchedule> for SqlxTaskScheduleRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<TaskSchedule>> {
        let row = sqlx::query("SELECT * FROM task_schedules WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        match row {
            Some(row) => {
                let schedule = Self::row_to_schedule(&row).map_err(DbError::ConnectionError)?;
                Ok(Some(schedule))
            }
            None => Ok(None),
        }
    }

    async fn create(&self, schedule: &TaskSchedule) -> AppResult<TaskSchedule> {
        let params = Self::schedule_to_params(schedule);

        sqlx::query(
            r#"
            INSERT INTO task_schedules (id, task_id, scheduled_day, outcome, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.0) // id
        .bind(&params.1) // task_id
        .bind(&params.2) // scheduled_day
        .bind(&params.3) // outcome
        .bind(&params.4) // created_at
        .bind(&params.5) // updated_at
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(schedule.clone())
    }

    async fn update(&self, schedule: &TaskSchedule) -> AppResult<TaskSchedule> {
        let params = Self::schedule_to_params(schedule);

        let result = sqlx::query(
            r#"
            UPDATE task_schedules SET
                scheduled_day = ?, outcome = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&params.2) // scheduled_day
        .bind(&params.3) // outcome
        .bind(&params.5) // updated_at
        .bind(&params.0) // id
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "TaskSchedule",
                schedule.id.to_string(),
            ));
        }

        Ok(schedule.clone())
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM task_schedules WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "TaskSchedule",
                id.to_string(),
            ));
        }

        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<TaskSchedule>> {
        let rows = sqlx::query("SELECT * FROM task_schedules ORDER BY scheduled_day DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let schedules = rows
            .iter()
            .map(Self::row_to_schedule)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(schedules)
    }
}

#[async_trait]
impl TaskScheduleRepository for SqlxTaskScheduleRepository {
    async fn find_by_task_id(&self, task_id: Uuid) -> AppResult<Vec<TaskSchedule>> {
        let rows = sqlx::query(
            "SELECT * FROM task_schedules WHERE task_id = ? ORDER BY scheduled_day DESC",
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let schedules = rows
            .iter()
            .map(Self::row_to_schedule)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(schedules)
    }

    async fn find_by_date(&self, date: DateTime<Utc>) -> AppResult<Vec<TaskSchedule>> {
        let normalized_date = crate::shared::core::normalize_to_day_start(date);

        let rows = sqlx::query(
            "SELECT * FROM task_schedules WHERE scheduled_day = ? ORDER BY created_at ASC",
        )
        .bind(normalized_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let schedules = rows
            .iter()
            .map(Self::row_to_schedule)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(schedules)
    }

    async fn find_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<Vec<TaskSchedule>> {
        let normalized_start = crate::shared::core::normalize_to_day_start(start_date);
        let normalized_end = crate::shared::core::normalize_to_day_start(end_date);

        let rows = sqlx::query(
            r#"
            SELECT * FROM task_schedules 
            WHERE scheduled_day >= ? AND scheduled_day <= ?
            ORDER BY scheduled_day ASC, created_at ASC
            "#,
        )
        .bind(normalized_start.to_rfc3339())
        .bind(normalized_end.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let schedules = rows
            .iter()
            .map(Self::row_to_schedule)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(schedules)
    }

    async fn delete_by_task_id(&self, task_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM task_schedules WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::database::connection::create_test_database;

    #[tokio::test]
    async fn test_schedule_crud_operations() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTaskScheduleRepository::new(pool);

        // 创建测试日程
        let schedule = TaskSchedule::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            crate::shared::core::normalize_to_day_start(Utc::now()),
            Utc::now(),
        );

        // 测试创建
        let created_schedule = repo.create(&schedule).await.unwrap();
        assert_eq!(created_schedule.task_id, schedule.task_id);

        // 测试查找
        let found_schedule = repo.find_by_id(schedule.id).await.unwrap().unwrap();
        assert_eq!(found_schedule.id, schedule.id);

        // 测试更新
        let mut updated_schedule = found_schedule.clone();
        updated_schedule.update_outcome(Outcome::PresenceLogged, Utc::now());

        let updated = repo.update(&updated_schedule).await.unwrap();
        assert_eq!(updated.outcome, Outcome::PresenceLogged);

        // 测试删除
        repo.delete(schedule.id).await.unwrap();
        let deleted_schedule = repo.find_by_id(schedule.id).await.unwrap();
        assert!(deleted_schedule.is_none());
    }

    #[tokio::test]
    async fn test_find_by_task_id() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTaskScheduleRepository::new(pool);

        let task_id = Uuid::new_v4();
        
        // 创建多个日程
        for i in 0..3 {
            let schedule = TaskSchedule::new(
                Uuid::new_v4(),
                task_id,
                crate::shared::core::normalize_to_day_start(Utc::now()) + chrono::Duration::days(i),
                Utc::now(),
            );
            repo.create(&schedule).await.unwrap();
        }

        // 测试按任务ID查找
        let schedules = repo.find_by_task_id(task_id).await.unwrap();
        assert_eq!(schedules.len(), 3);
        assert!(schedules.iter().all(|s| s.task_id == task_id));
    }

    #[tokio::test]
    async fn test_find_by_date() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTaskScheduleRepository::new(pool);

        let target_date = crate::shared::core::normalize_to_day_start(Utc::now());
        
        // 创建同一天的多个日程
        for _ in 0..2 {
            let schedule = TaskSchedule::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                target_date,
                Utc::now(),
            );
            repo.create(&schedule).await.unwrap();
        }

        // 创建不同天的日程
        let other_date = target_date + chrono::Duration::days(1);
        let other_schedule = TaskSchedule::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            other_date,
            Utc::now(),
        );
        repo.create(&other_schedule).await.unwrap();

        // 测试按日期查找
        let schedules = repo.find_by_date(target_date).await.unwrap();
        assert_eq!(schedules.len(), 2);
        assert!(schedules.iter().all(|s| s.scheduled_day == target_date));
    }
}

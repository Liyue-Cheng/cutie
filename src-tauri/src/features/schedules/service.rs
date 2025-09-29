/// 日程业务逻辑层
///
/// 实现日程的业务逻辑和规则

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shared::{
    core::{AppError, AppResult, Outcome, TaskSchedule},
    database::TaskScheduleRepository,
};

use super::payloads::{ScheduleMode, ScheduleStatsResponse, ScheduleTaskPayload};

/// 日程服务
pub struct ScheduleService<R: TaskScheduleRepository> {
    repository: R,
}

impl<R: TaskScheduleRepository> ScheduleService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 安排任务到指定日期
    pub async fn schedule_task(&self, payload: ScheduleTaskPayload) -> AppResult<TaskSchedule> {
        let now = Utc::now();
        let normalized_day = crate::shared::core::normalize_to_day_start(payload.target_day);

        match payload.mode {
            ScheduleMode::Link => {
                // 链接模式：创建新的日程记录
                let schedule_id = Uuid::new_v4();
                let schedule = TaskSchedule::new(schedule_id, payload.task_id, normalized_day, now);

                self.repository.create(&schedule).await
            }
            ScheduleMode::Move => {
                // 移动模式：更新现有日程记录
                let source_id = payload.source_schedule_id.ok_or_else(|| {
                    AppError::validation_error(
                        "source_schedule_id",
                        "移动模式下必须提供源日程ID",
                        "SOURCE_SCHEDULE_ID_REQUIRED",
                    )
                })?;

                let mut schedule = self
                    .repository
                    .find_by_id(source_id)
                    .await?
                    .ok_or_else(|| AppError::not_found("TaskSchedule", source_id.to_string()))?;

                schedule.reschedule(normalized_day, now);
                self.repository.update(&schedule).await
            }
        }
    }

    /// 根据ID获取日程
    pub async fn get_schedule(&self, id: Uuid) -> AppResult<TaskSchedule> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("TaskSchedule", id.to_string()))
    }

    /// 删除日程
    pub async fn delete_schedule(&self, id: Uuid) -> AppResult<()> {
        // 检查日程是否存在
        self.get_schedule(id).await?;

        // 删除日程
        self.repository.delete(id).await?;

        // TODO: 如果任务没有其他日程，自动回归Staging区
        // 这里需要检查任务是否还有其他日程安排

        Ok(())
    }

    /// 记录努力
    pub async fn log_presence(&self, schedule_id: Uuid, note: Option<String>) -> AppResult<TaskSchedule> {
        let mut schedule = self.get_schedule(schedule_id).await?;
        let now = Utc::now();

        // 验证日程状态
        if schedule.outcome == Outcome::CompletedOnDay {
            return Err(AppError::conflict("不能为已完成的日程记录努力"));
        }

        schedule.log_presence(now)?;
        self.repository.update(&schedule).await
    }

    /// 取消任务的所有日程
    pub async fn unschedule_task_completely(&self, task_id: Uuid) -> AppResult<()> {
        self.repository.delete_by_task_id(task_id).await?;

        // TODO: 在适当的上下文中重新创建排序记录
        // 这里需要调用排序服务来处理排序逻辑

        Ok(())
    }

    /// 根据条件获取日程列表
    pub async fn get_schedules(
        &self,
        date: Option<chrono::DateTime<Utc>>,
        start_date: Option<chrono::DateTime<Utc>>,
        end_date: Option<chrono::DateTime<Utc>>,
        task_id: Option<Uuid>,
    ) -> AppResult<Vec<TaskSchedule>> {
        // 根据查询类型选择不同的查询方法
        if let Some(date) = date {
            // 单日查询
            let mut schedules = self.repository.find_by_date(date).await?;

            // 如果指定了任务ID，进一步过滤
            if let Some(task_id) = task_id {
                schedules.retain(|s| s.task_id == task_id);
            }

            Ok(schedules)
        } else if let (Some(start), Some(end)) = (start_date, end_date) {
            // 日期范围查询
            let mut schedules = self.repository.find_by_date_range(start, end).await?;

            // 如果指定了任务ID，进一步过滤
            if let Some(task_id) = task_id {
                schedules.retain(|s| s.task_id == task_id);
            }

            Ok(schedules)
        } else if let Some(task_id) = task_id {
            // 按任务ID查询
            self.repository.find_by_task_id(task_id).await
        } else {
            // 默认返回所有日程
            self.repository.find_all().await
        }
    }

    /// 获取日程统计
    pub async fn get_schedule_stats(&self) -> AppResult<ScheduleStatsResponse> {
        let all_schedules = self.repository.find_all().await?;

        let total_count = all_schedules.len() as i64;
        let planned_count = all_schedules
            .iter()
            .filter(|s| s.outcome == Outcome::Planned)
            .count() as i64;
        let presence_logged_count = all_schedules
            .iter()
            .filter(|s| s.outcome == Outcome::PresenceLogged)
            .count() as i64;
        let completed_on_day_count = all_schedules
            .iter()
            .filter(|s| s.outcome == Outcome::CompletedOnDay)
            .count() as i64;
        let carried_over_count = all_schedules
            .iter()
            .filter(|s| s.outcome == Outcome::CarriedOver)
            .count() as i64;

        // 计算完成率
        let completion_rate = if total_count > 0 {
            (completed_on_day_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        // TODO: 实现本周、本月统计
        let this_week_count = 0;
        let this_month_count = 0;

        Ok(ScheduleStatsResponse {
            total_count,
            planned_count,
            presence_logged_count,
            completed_on_day_count,
            carried_over_count,
            this_week_count,
            this_month_count,
            completion_rate,
        })
    }

    /// 批量删除日程
    pub async fn bulk_delete_schedules(&self, schedule_ids: Vec<Uuid>) -> AppResult<usize> {
        let mut deleted_count = 0;

        for schedule_id in schedule_ids {
            if self.repository.find_by_id(schedule_id).await?.is_some() {
                self.repository.delete(schedule_id).await?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// 批量记录努力
    pub async fn bulk_log_presence(&self, schedule_ids: Vec<Uuid>) -> AppResult<usize> {
        let mut logged_count = 0;
        let now = Utc::now();

        for schedule_id in schedule_ids {
            if let Some(mut schedule) = self.repository.find_by_id(schedule_id).await? {
                if schedule.outcome != Outcome::CompletedOnDay {
                    if schedule.log_presence(now).is_ok() {
                        self.repository.update(&schedule).await?;
                        logged_count += 1;
                    }
                }
            }
        }

        Ok(logged_count)
    }

    /// 批量标记为延期
    pub async fn bulk_mark_carried_over(&self, schedule_ids: Vec<Uuid>) -> AppResult<usize> {
        let mut marked_count = 0;
        let now = Utc::now();

        for schedule_id in schedule_ids {
            if let Some(mut schedule) = self.repository.find_by_id(schedule_id).await? {
                schedule.mark_carried_over(now);
                self.repository.update(&schedule).await?;
                marked_count += 1;
            }
        }

        Ok(marked_count)
    }

    /// 批量移动到新日期
    pub async fn bulk_move_to_date(
        &self,
        schedule_ids: Vec<Uuid>,
        target_date: DateTime<Utc>,
    ) -> AppResult<usize> {
        let mut moved_count = 0;
        let now = Utc::now();
        let normalized_date = crate::shared::core::normalize_to_day_start(target_date);

        for schedule_id in schedule_ids {
            if let Some(mut schedule) = self.repository.find_by_id(schedule_id).await? {
                schedule.reschedule(normalized_date, now);
                self.repository.update(&schedule).await?;
                moved_count += 1;
            }
        }

        Ok(moved_count)
    }

    /// 验证日程业务规则
    fn validate_schedule_business_rules(&self, schedule: &TaskSchedule) -> AppResult<()> {
        // 验证日期必须是当天的零点
        let normalized_day = crate::shared::core::normalize_to_day_start(schedule.scheduled_day);
        if schedule.scheduled_day != normalized_day {
            return Err(AppError::validation_error(
                "scheduled_day",
                "安排日期必须是当天的零点时间戳",
                "INVALID_DAY_TIMESTAMP",
            ));
        }

        // 验证不能为未来日期记录努力
        if schedule.outcome == Outcome::PresenceLogged && schedule.scheduled_day > Utc::now() {
            return Err(AppError::validation_error(
                "outcome",
                "不能为未来日期记录努力",
                "FUTURE_PRESENCE_LOG",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::schedules::repository::SqlxTaskScheduleRepository,
        shared::database::connection::create_test_database,
    };

    #[tokio::test]
    async fn test_schedule_task_link_mode() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTaskScheduleRepository::new(pool);
        let service = ScheduleService::new(repository);

        let payload = ScheduleTaskPayload {
            task_id: Uuid::new_v4(),
            target_day: crate::shared::core::normalize_to_day_start(Utc::now()),
            mode: ScheduleMode::Link,
            source_schedule_id: None,
        };

        let schedule = service.schedule_task(payload).await.unwrap();
        assert_eq!(schedule.outcome, Outcome::Planned);
    }

    #[tokio::test]
    async fn test_log_presence() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTaskScheduleRepository::new(pool);
        let service = ScheduleService::new(repository);

        // 创建日程
        let schedule = TaskSchedule::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            crate::shared::core::normalize_to_day_start(Utc::now()),
            Utc::now(),
        );
        let created_schedule = service.repository.create(&schedule).await.unwrap();

        // 记录努力
        let updated_schedule = service
            .log_presence(created_schedule.id, Some("Worked on this task".to_string()))
            .await
            .unwrap();

        assert_eq!(updated_schedule.outcome, Outcome::PresenceLogged);
    }

    #[tokio::test]
    async fn test_bulk_operations() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTaskScheduleRepository::new(pool);
        let service = ScheduleService::new(repository);

        // 创建多个日程
        let mut schedule_ids = Vec::new();
        for i in 0..3 {
            let schedule = TaskSchedule::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                crate::shared::core::normalize_to_day_start(Utc::now()) + chrono::Duration::days(i),
                Utc::now(),
            );
            let created = service.repository.create(&schedule).await.unwrap();
            schedule_ids.push(created.id);
        }

        // 批量记录努力
        let logged_count = service.bulk_log_presence(schedule_ids.clone()).await.unwrap();
        assert_eq!(logged_count, 3);

        // 批量标记为延期
        let marked_count = service
            .bulk_mark_carried_over(schedule_ids.clone())
            .await
            .unwrap();
        assert_eq!(marked_count, 3);

        // 批量删除
        let deleted_count = service.bulk_delete_schedules(schedule_ids).await.unwrap();
        assert_eq!(deleted_count, 3);
    }
}

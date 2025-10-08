/// 内存测试仓库实现
///
/// 这些实现用于单元测试，提供快速的内存存储，不依赖真实数据库
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::*;
use crate::common::error::DbError;
use crate::core::models::*;

/// 内存中的"事务"，实际上只是一个标记
pub struct MemoryTransaction;

/// 内存任务仓库
#[derive(Debug, Clone)]
pub struct MemoryTaskRepository {
    tasks: Arc<RwLock<HashMap<Uuid, Task>>>,
}

impl MemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn with_test_data() -> Self {
        let repo = Self::new();
        let now = Utc::now();

        // 添加一些测试数据
        let task1 = Task::new(Uuid::new_v4(), "测试任务1".to_string(), now);
        let task2 = Task::new(Uuid::new_v4(), "测试任务2".to_string(), now);

        {
            let mut tasks = repo.tasks.write().await;
            tasks.insert(task1.id, task1);
            tasks.insert(task2.id, task2);
        }

        repo
    }
}

impl Default for MemoryTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskRepository for MemoryTaskRepository {
    async fn begin_transaction(&self) -> Result<Transaction<'_>, DbError> {
        // 内存实现不需要真实的事务，返回一个虚拟的
        // 注意：这里我们返回一个生命周期问题，实际使用中需要更复杂的处理
        Err(DbError::ConnectionError(sqlx::Error::Configuration(
            "Memory repository doesn't support real transactions".into(),
        )))
    }

    async fn create(&self, _tx: &mut Transaction<'_>, task: &Task) -> Result<Task, DbError> {
        let mut tasks = self.tasks.write().await;

        if tasks.contains_key(&task.id) {
            return Err(DbError::ConstraintViolation {
                message: format!("Task with id {} already exists", task.id),
            });
        }

        tasks.insert(task.id, task.clone());
        Ok(task.clone())
    }

    async fn update(&self, _tx: &mut Transaction<'_>, task: &Task) -> Result<Task, DbError> {
        let mut tasks = self.tasks.write().await;

        if !tasks.contains_key(&task.id) {
            return Err(DbError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task.id.to_string(),
            });
        }

        tasks.insert(task.id, task.clone());
        Ok(task.clone())
    }

    async fn set_completed(
        &self,
        _tx: &mut Transaction<'_>,
        task_id: Uuid,
        completion_time: DateTime<Utc>,
    ) -> Result<Task, DbError> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            if !task.is_deleted {
                task.completed_at = Some(completion_time);
                task.updated_at = completion_time;
                Ok(task.clone())
            } else {
                Err(DbError::NotFound {
                    entity_type: "Task".to_string(),
                    entity_id: task_id.to_string(),
                })
            }
        } else {
            Err(DbError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task_id.to_string(),
            })
        }
    }

    async fn reopen(&self, _tx: &mut Transaction<'_>, task_id: Uuid) -> Result<Task, DbError> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            if !task.is_deleted {
                task.completed_at = None;
                task.updated_at = Utc::now();
                Ok(task.clone())
            } else {
                Err(DbError::NotFound {
                    entity_type: "Task".to_string(),
                    entity_id: task_id.to_string(),
                })
            }
        } else {
            Err(DbError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task_id.to_string(),
            })
        }
    }

    async fn find_by_id(&self, task_id: Uuid) -> Result<Option<Task>, DbError> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(&task_id).filter(|t| !t.is_deleted).cloned())
    }

    async fn find_many_by_ids(&self, task_ids: &[Uuid]) -> Result<Vec<Task>, DbError> {
        let tasks = self.tasks.read().await;
        let mut result = Vec::new();

        for &task_id in task_ids {
            if let Some(task) = tasks.get(&task_id) {
                if !task.is_deleted {
                    result.push(task.clone());
                }
            }
        }

        Ok(result)
    }

    async fn find_unscheduled(&self) -> Result<Vec<Task>, DbError> {
        let tasks = self.tasks.read().await;
        // 简化实现：返回所有未删除的任务
        // 在真实实现中需要检查task_schedules表
        Ok(tasks.values().filter(|t| !t.is_deleted).cloned().collect())
    }

    async fn find_by_project_id(&self, project_id: Uuid) -> Result<Vec<Task>, DbError> {
        let tasks = self.tasks.read().await;
        Ok(tasks
            .values()
            .filter(|t| !t.is_deleted && t.project_id == Some(project_id))
            .cloned()
            .collect())
    }

    async fn find_by_area_id(&self, area_id: Uuid) -> Result<Vec<Task>, DbError> {
        let tasks = self.tasks.read().await;
        Ok(tasks
            .values()
            .filter(|t| !t.is_deleted && t.area_id == Some(area_id))
            .cloned()
            .collect())
    }

    async fn find_completed(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Task>, DbError> {
        let tasks = self.tasks.read().await;
        let mut completed: Vec<_> = tasks
            .values()
            .filter(|t| !t.is_deleted && t.completed_at.is_some())
            .cloned()
            .collect();

        completed.sort_by(|a, b| b.completed_at.cmp(&a.completed_at));

        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(100) as usize;

        Ok(completed.into_iter().skip(offset).take(limit).collect())
    }

    async fn soft_delete(&self, _tx: &mut Transaction<'_>, task_id: Uuid) -> Result<(), DbError> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            task.is_deleted = true;
            task.updated_at = Utc::now();
            Ok(())
        } else {
            Ok(()) // 幂等操作
        }
    }

    async fn restore(&self, _tx: &mut Transaction<'_>, task_id: Uuid) -> Result<Task, DbError> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            task.deleted_at IS NULL;
            task.updated_at = Utc::now();
            Ok(task.clone())
        } else {
            Err(DbError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task_id.to_string(),
            })
        }
    }

    async fn search(&self, query: &str, limit: Option<i64>) -> Result<Vec<Task>, DbError> {
        let tasks = self.tasks.read().await;
        let query_lower = query.to_lowercase();
        let limit = limit.unwrap_or(50) as usize;

        let mut results: Vec<_> = tasks
            .values()
            .filter(|t| !t.is_deleted)
            .filter(|t| {
                t.title.to_lowercase().contains(&query_lower)
                    || t.glance_note
                        .as_ref()
                        .map_or(false, |note| note.to_lowercase().contains(&query_lower))
                    || t.detail_note
                        .as_ref()
                        .map_or(false, |note| note.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results.truncate(limit);

        Ok(results)
    }

    async fn count_by_status(&self) -> Result<TaskCountByStatus, DbError> {
        let tasks = self.tasks.read().await;
        let active_tasks: Vec<_> = tasks.values().filter(|t| !t.is_deleted).collect();

        let total = active_tasks.len() as i64;
        let completed = active_tasks
            .iter()
            .filter(|t| t.completed_at.is_some())
            .count() as i64;
        let pending = total - completed;

        Ok(TaskCountByStatus {
            total,
            completed,
            pending,
            scheduled: 0,         // 简化实现
            unscheduled: pending, // 简化实现
        })
    }
}

// 为了简化，我们只实现TaskRepository的内存版本
// 其他仓库的内存实现可以按需添加

/// 内存任务日程仓库
#[derive(Debug, Clone)]
pub struct MemoryTaskScheduleRepository {
    schedules: Arc<RwLock<HashMap<Uuid, TaskSchedule>>>,
}

impl MemoryTaskScheduleRepository {
    pub fn new() -> Self {
        Self {
            schedules: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryTaskScheduleRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskScheduleRepository for MemoryTaskScheduleRepository {
    async fn create(
        &self,
        _tx: &mut Transaction<'_>,
        schedule: &TaskSchedule,
    ) -> Result<TaskSchedule, DbError> {
        let mut schedules = self.schedules.write().await;

        // 检查是否已存在相同任务在同一天的日程
        let exists = schedules
            .values()
            .any(|s| s.task_id == schedule.task_id && s.scheduled_day == schedule.scheduled_day);

        if exists {
            return Err(DbError::ConstraintViolation {
                message: format!(
                    "Task {} already has a schedule for day {}",
                    schedule.task_id, schedule.scheduled_day
                ),
            });
        }

        schedules.insert(schedule.id, schedule.clone());
        Ok(schedule.clone())
    }

    async fn update_outcome(
        &self,
        _tx: &mut Transaction<'_>,
        schedule_id: Uuid,
        new_outcome: Outcome,
    ) -> Result<TaskSchedule, DbError> {
        let mut schedules = self.schedules.write().await;

        if let Some(schedule) = schedules.get_mut(&schedule_id) {
            schedule.outcome = new_outcome;
            schedule.updated_at = Utc::now();
            Ok(schedule.clone())
        } else {
            Err(DbError::NotFound {
                entity_type: "TaskSchedule".to_string(),
                entity_id: schedule_id.to_string(),
            })
        }
    }

    async fn reschedule(
        &self,
        _tx: &mut Transaction<'_>,
        schedule_id: Uuid,
        new_day: DateTime<Utc>,
    ) -> Result<TaskSchedule, DbError> {
        let mut schedules = self.schedules.write().await;

        if let Some(schedule) = schedules.get_mut(&schedule_id) {
            schedule.scheduled_day = new_day;
            schedule.outcome = Outcome::Planned;
            schedule.updated_at = Utc::now();
            Ok(schedule.clone())
        } else {
            Err(DbError::NotFound {
                entity_type: "TaskSchedule".to_string(),
                entity_id: schedule_id.to_string(),
            })
        }
    }

    async fn delete(&self, _tx: &mut Transaction<'_>, schedule_id: Uuid) -> Result<(), DbError> {
        let mut schedules = self.schedules.write().await;
        schedules.remove(&schedule_id);
        Ok(())
    }

    async fn delete_all_for_task(
        &self,
        _tx: &mut Transaction<'_>,
        task_id: Uuid,
    ) -> Result<(), DbError> {
        let mut schedules = self.schedules.write().await;
        schedules.retain(|_, schedule| schedule.task_id != task_id);
        Ok(())
    }

    async fn delete_future_for_task(
        &self,
        _tx: &mut Transaction<'_>,
        task_id: Uuid,
        since: DateTime<Utc>,
    ) -> Result<(), DbError> {
        let mut schedules = self.schedules.write().await;
        schedules
            .retain(|_, schedule| !(schedule.task_id == task_id && schedule.scheduled_day > since));
        Ok(())
    }

    async fn find_by_day(&self, day: DateTime<Utc>) -> Result<Vec<TaskSchedule>, DbError> {
        let schedules = self.schedules.read().await;
        Ok(schedules
            .values()
            .filter(|s| s.scheduled_day == day)
            .cloned()
            .collect())
    }

    async fn find_all_for_task(&self, task_id: Uuid) -> Result<Vec<TaskSchedule>, DbError> {
        let schedules = self.schedules.read().await;
        let mut result: Vec<_> = schedules
            .values()
            .filter(|s| s.task_id == task_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| a.scheduled_day.cmp(&b.scheduled_day));
        Ok(result)
    }

    async fn find_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TaskSchedule>, DbError> {
        let schedules = self.schedules.read().await;
        let mut result: Vec<_> = schedules
            .values()
            .filter(|s| s.scheduled_day >= start_date && s.scheduled_day <= end_date)
            .cloned()
            .collect();
        result.sort_by(|a, b| a.scheduled_day.cmp(&b.scheduled_day));
        Ok(result)
    }

    async fn find_by_outcome(
        &self,
        outcome: Outcome,
        limit: Option<i64>,
    ) -> Result<Vec<TaskSchedule>, DbError> {
        let schedules = self.schedules.read().await;
        let limit = limit.unwrap_or(100) as usize;

        let mut result: Vec<_> = schedules
            .values()
            .filter(|s| s.outcome == outcome)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.scheduled_day.cmp(&a.scheduled_day));
        result.truncate(limit);

        Ok(result)
    }

    async fn count_by_outcome(&self) -> Result<ScheduleCountByOutcome, DbError> {
        let schedules = self.schedules.read().await;
        let all_schedules: Vec<_> = schedules.values().collect();

        let total = all_schedules.len() as i64;
        let planned = all_schedules
            .iter()
            .filter(|s| s.outcome == Outcome::Planned)
            .count() as i64;
        let presence_logged = all_schedules
            .iter()
            .filter(|s| s.outcome == Outcome::PresenceLogged)
            .count() as i64;
        let completed_on_day = all_schedules
            .iter()
            .filter(|s| s.outcome == Outcome::CompletedOnDay)
            .count() as i64;
        let carried_over = all_schedules
            .iter()
            .filter(|s| s.outcome == Outcome::CarriedOver)
            .count() as i64;

        Ok(ScheduleCountByOutcome {
            total,
            planned,
            presence_logged,
            completed_on_day,
            carried_over,
        })
    }
}

// 其他内存仓库实现可以按需添加...
// 由于篇幅限制，这里只实现了Task和TaskSchedule的内存版本作为示例

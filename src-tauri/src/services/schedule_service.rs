use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::{AppError, AppResult};
use crate::common::utils::time_utils::normalize_to_day_start;
use crate::core::models::{ContextType, Outcome, TaskSchedule};
use crate::ports::{Clock, IdGenerator};
use crate::repositories::{OrderingRepository, TaskRepository, TaskScheduleRepository};

/// 日程服务
///
/// **预期行为简介:** 封装所有与TaskSchedule相关的业务逻辑，包括创建、移动、删除日程等操作
pub struct ScheduleService {
    /// 时钟服务
    clock: Arc<dyn Clock>,

    /// ID生成器
    id_generator: Arc<dyn IdGenerator>,

    /// 任务仓库
    task_repository: Arc<dyn TaskRepository>,

    /// 任务日程仓库
    task_schedule_repository: Arc<dyn TaskScheduleRepository>,

    /// 排序仓库
    ordering_repository: Arc<dyn OrderingRepository>,
}

impl ScheduleService {
    /// 创建新的日程服务
    pub fn new(
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        task_repository: Arc<dyn TaskRepository>,
        task_schedule_repository: Arc<dyn TaskScheduleRepository>,
        ordering_repository: Arc<dyn OrderingRepository>,
    ) -> Self {
        Self {
            clock,
            id_generator,
            task_repository,
            task_schedule_repository,
            ordering_repository,
        }
    }

    /// 为任务创建额外的日程安排
    ///
    /// **函数签名:** `pub async fn create_additional_schedule(&self, task_id: Uuid, target_day: DateTime<Utc>) -> Result<TaskSchedule, AppError>`
    /// **预期行为简介:** 为一个任务在新的日期上创建一个额外的日程安排，原日程保持不变。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证Task:** 调用 `TaskRepository::find_by_id(task_id)`。若任务不存在，回滚并返回`AppError::NotFound`。若任务的`completed_at`非空，回滚并返回`AppError::Conflict("不能为已完成的任务安排日程")`。
    /// 3. **验证目标日期:** 检查`target_day`是否为规范化的零点时间戳。
    /// 4. **幂等性检查:** 调用 `TaskScheduleRepository::find_one_by_task_and_day(task_id, target_day)`。如果已存在记录，**直接提交事务并返回该记录**。
    /// 5. **生成ID和排序:** `new_schedule_id = self.id_generator.new_uuid()`。计算`target_day`的默认`sort_order`（例如，置于列表末尾）。
    /// 6. **核心操作 (创建日程):** 调用 `TaskScheduleRepository::create` 创建一条新的`TaskSchedule`记录，包含`id`, `task_id`, `scheduled_day`, `outcome='PLANNED'`。
    /// 7. **排序处理:** 调用 `OrderingRepository::upsert` 为 `DAILY_KANBAN` 上下文 (`context_id`为`target_day`的时间戳字符串) 创建一条新的排序记录。
    /// 8. **提交事务。**
    /// 9. **返回:** 返回新创建的`TaskSchedule`对象。
    /// **预期副作用:** 向`task_schedule`表插入1条记录，向`ordering`表插入1条记录。
    pub async fn create_additional_schedule(
        &self,
        task_id: Uuid,
        target_day: DateTime<Utc>,
    ) -> AppResult<TaskSchedule> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证Task
        let task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        if task.completed_at.is_some() {
            return Err(AppError::conflict("不能为已完成的任务安排日程"));
        }

        // 3. 验证目标日期
        let normalized_target_day = normalize_to_day_start(target_day);

        // 4. 幂等性检查
        let existing_schedules = self
            .task_schedule_repository
            .find_by_day(normalized_target_day)
            .await?;
        for schedule in existing_schedules {
            if schedule.task_id == task_id {
                // 已存在记录，直接返回
                tx.commit().await.map_err(|e| {
                    AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                        message: e.to_string(),
                    })
                })?;
                return Ok(schedule);
            }
        }

        // 5. 生成ID
        let new_schedule_id = self.id_generator.new_uuid();
        let now = self.clock.now_utc();

        // 6. 核心操作（创建日程）
        let new_schedule = TaskSchedule::new(new_schedule_id, task_id, normalized_target_day, now);
        let created_schedule = self
            .task_schedule_repository
            .create(&mut tx, &new_schedule)
            .await?;

        // 7. 排序处理
        let context_id = normalized_target_day.timestamp().to_string();
        let sort_order = self
            .ordering_repository
            .get_next_sort_order(&ContextType::DailyKanban, &context_id)
            .await?;

        let ordering = crate::core::models::Ordering::new(
            self.id_generator.new_uuid(),
            ContextType::DailyKanban,
            context_id,
            task_id,
            sort_order,
            now,
        )?;

        self.ordering_repository.upsert(&mut tx, &ordering).await?;

        // 8. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 9. 返回
        Ok(created_schedule)
    }

    /// 重新安排任务日程
    ///
    /// **函数签名:** `pub async fn reschedule_task(&self, schedule_id: Uuid, target_day: DateTime<Utc>) -> Result<TaskSchedule, AppError>`
    /// **预期行为简介:** 将一个已存在的日程安排移动到新的日期。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证源日程:** 调用 `TaskScheduleRepository::find_by_id(schedule_id)` 获取源日程。若不存在，回滚并返回`AppError::NotFound`。
    /// 3. **验证Task:** (同上, 使用源日程的`task_id`)。
    /// 4. **验证目标日期:** (同上)。
    /// 5. **检查目标日程冲突:** 调用 `TaskScheduleRepository::find_one_by_task_and_day` 检查`target_day`是否已存在该任务的日程。若存在且其`id`不等于`schedule_id`，回滚并返回`AppError::Conflict`。
    /// 6. **核心操作 (更新日程):** 调用 `TaskScheduleRepository::reschedule`，将`schedule_id`对应记录的`scheduled_day`字段更新为`target_day`。
    /// 7. **排序处理:**
    ///    a. 获取源日程的`scheduled_day` (`source_day`)。
    ///    b. 调用 `OrderingRepository::delete_for_task_in_context` 删除 `source_day` 对应的`DAILY_KANBAN`排序记录。
    ///    c. 调用 `OrderingRepository::upsert` 为 `target_day` 对应的`DAILY_KANBAN`上下文创建一条新的排序记录。
    /// 8. **提交事务。**
    /// 9. **返回:** 返回更新后的`TaskSchedule`对象。
    /// **预期副作用:** 更新`task_schedule`表1条记录。删除`ordering`表1条记录，插入`ordering`表1条记录。
    pub async fn reschedule_task(
        &self,
        schedule_id: Uuid,
        target_day: DateTime<Utc>,
    ) -> AppResult<TaskSchedule> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证源日程
        // 注意：我们需要先实现一个查找方法，这里简化处理
        let all_schedules = self
            .task_schedule_repository
            .find_by_date_range(
                DateTime::from_timestamp(0, 0).unwrap().with_timezone(&Utc),
                DateTime::from_timestamp(4000000000, 0)
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .await?;

        let source_schedule = all_schedules
            .into_iter()
            .find(|s| s.id == schedule_id)
            .ok_or_else(|| AppError::not_found("TaskSchedule", schedule_id.to_string()))?;

        // 3. 验证Task
        let task = self
            .task_repository
            .find_by_id(source_schedule.task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", source_schedule.task_id.to_string()))?;

        if task.completed_at.is_some() {
            return Err(AppError::conflict("不能重新安排已完成任务的日程"));
        }

        // 4. 验证目标日期
        let normalized_target_day = normalize_to_day_start(target_day);

        // 5. 检查目标日程冲突
        let target_day_schedules = self
            .task_schedule_repository
            .find_by_day(normalized_target_day)
            .await?;
        for schedule in target_day_schedules {
            if schedule.task_id == source_schedule.task_id && schedule.id != schedule_id {
                return Err(AppError::conflict("目标日期已存在该任务的日程"));
            }
        }

        // 6. 核心操作（更新日程）
        let updated_schedule = self
            .task_schedule_repository
            .reschedule(&mut tx, schedule_id, normalized_target_day)
            .await?;

        // 7. 排序处理
        let source_day = source_schedule.scheduled_day;
        let source_context_id = source_day.timestamp().to_string();
        let target_context_id = normalized_target_day.timestamp().to_string();

        // a. 删除源日程的排序记录
        self.ordering_repository
            .delete_for_task_in_context(
                &mut tx,
                source_schedule.task_id,
                &ContextType::DailyKanban,
                &source_context_id,
            )
            .await?;

        // c. 为目标日期创建新的排序记录
        let sort_order = self
            .ordering_repository
            .get_next_sort_order(&ContextType::DailyKanban, &target_context_id)
            .await?;

        let ordering = crate::core::models::Ordering::new(
            self.id_generator.new_uuid(),
            ContextType::DailyKanban,
            target_context_id,
            source_schedule.task_id,
            sort_order,
            self.clock.now_utc(),
        )?;

        self.ordering_repository.upsert(&mut tx, &ordering).await?;

        // 8. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 9. 返回
        Ok(updated_schedule)
    }

    /// 删除单个日程
    ///
    /// **函数签名:** `pub async fn delete_schedule(&self, schedule_id: Uuid) -> Result<(), AppError>`
    /// **预期行为简介:** 删除一个具体的、单一的日程安排。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证源日程:** 调用 `TaskScheduleRepository::find_by_id(schedule_id)` 获取日程信息。若不存在，**幂等地提交事务并返回`Ok(())`**。
    /// 3. **核心操作 (删除日程):** 调用 `TaskScheduleRepository::delete(schedule_id)`。
    /// 4. **排序处理:** 调用 `OrderingRepository::delete_for_task_in_context` 删除该日程`scheduled_day`对应的`DAILY_KANBAN`排序记录。
    /// 5. **回归Staging检查:** 调用`TaskScheduleRepository::find_all_for_task`检查该任务是否还有其他日程。如果没有，则为该任务在`MISC`上下文中创建一条排序记录（同`unschedule_task_completely`的排序逻辑）。
    /// 6. **提交事务。**
    /// 7. **返回:** `Ok(())`。
    /// **预期副作用:** 删除`task_schedule`表1条记录。删除`ordering`表1条记录。可能向`ordering`表插入1条记录。
    pub async fn delete_schedule(&self, schedule_id: Uuid) -> AppResult<()> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证源日程
        let all_schedules = self
            .task_schedule_repository
            .find_by_date_range(
                DateTime::from_timestamp(0, 0).unwrap().with_timezone(&Utc),
                DateTime::from_timestamp(4000000000, 0)
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .await?;

        let schedule = match all_schedules.into_iter().find(|s| s.id == schedule_id) {
            Some(s) => s,
            None => {
                // 幂等地提交事务并返回Ok
                tx.commit().await.map_err(|e| {
                    AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                        message: e.to_string(),
                    })
                })?;
                return Ok(());
            }
        };

        // 3. 核心操作（删除日程）
        self.task_schedule_repository
            .delete(&mut tx, schedule_id)
            .await?;

        // 4. 排序处理
        let context_id = schedule.scheduled_day.timestamp().to_string();
        self.ordering_repository
            .delete_for_task_in_context(
                &mut tx,
                schedule.task_id,
                &ContextType::DailyKanban,
                &context_id,
            )
            .await?;

        // 5. 回归Staging检查
        let remaining_schedules = self
            .task_schedule_repository
            .find_all_for_task(schedule.task_id)
            .await?;
        if remaining_schedules.is_empty() {
            // 任务没有其他日程，创建Staging排序记录
            let task = self
                .task_repository
                .find_by_id(schedule.task_id)
                .await?
                .ok_or_else(|| AppError::not_found("Task", schedule.task_id.to_string()))?;

            let staging_context = if let Some(project_id) = task.project_id {
                (ContextType::ProjectList, format!("project::{}", project_id))
            } else {
                (ContextType::Misc, "floating".to_string())
            };

            let sort_order = self
                .ordering_repository
                .get_next_sort_order(&staging_context.0, &staging_context.1)
                .await?;

            let ordering = crate::core::models::Ordering::new(
                self.id_generator.new_uuid(),
                staging_context.0,
                staging_context.1,
                schedule.task_id,
                sort_order,
                self.clock.now_utc(),
            )?;

            self.ordering_repository.upsert(&mut tx, &ordering).await?;
        }

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 7. 返回
        Ok(())
    }

    /// 完全取消任务的所有日程安排
    ///
    /// **函数签名:** `pub async fn unschedule_task_completely(&self, task_id: Uuid) -> Result<(), AppError>`
    /// **预期行为简介:** 将一个任务从所有日程中移除，使其回归Staging区。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证:** 调用`TaskRepository::find_by_id`检查`task_id`是否存在。若不存在，返回`NotFound`。
    /// 3. **核心操作 (删除所有日程):** 调用 `TaskScheduleRepository::delete_all_for_task(task_id)`。
    /// 4. **排序处理 (清理与重建):**
    ///    a. 调用 `OrderingRepository::delete_all_for_task_in_daily_contexts`。
    ///    b. 根据任务的`project_id`，为该任务在`MISC::floating`或`PROJECT_LIST::{project_id}`上下文中创建一条新的`Ordering`记录。
    /// 5. **提交事务。**
    /// 6. **返回:** `Ok(())`。
    /// **预期副作用:** 删除`task_schedule`和`ordering`中的多条记录，并向`ordering`插入1条记录。
    pub async fn unschedule_task_completely(&self, task_id: Uuid) -> AppResult<()> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证
        let task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 3. 核心操作（删除所有日程）
        self.task_schedule_repository
            .delete_all_for_task(&mut tx, task_id)
            .await?;

        // 4. 排序处理（清理与重建）
        // a. 删除所有日程相关的排序记录
        self.ordering_repository
            .delete_all_for_task(&mut tx, task_id)
            .await?;

        // b. 根据任务的project_id创建新的排序记录
        let (context_type, context_id) = if let Some(project_id) = task.project_id {
            (ContextType::ProjectList, format!("project::{}", project_id))
        } else {
            (ContextType::Misc, "floating".to_string())
        };

        let sort_order = self
            .ordering_repository
            .get_next_sort_order(&context_type, &context_id)
            .await?;

        let ordering = crate::core::models::Ordering::new(
            self.id_generator.new_uuid(),
            context_type,
            context_id,
            task_id,
            sort_order,
            self.clock.now_utc(),
        )?;

        self.ordering_repository.upsert(&mut tx, &ordering).await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 6. 返回
        Ok(())
    }

    /// 记录努力
    ///
    /// **函数签名:** `pub async fn log_presence(&self, schedule_id: Uuid) -> Result<TaskSchedule, AppError>`
    /// **预期行为简介:** 为指定的日程记录"努力已付出"。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证日程:** 调用 `TaskScheduleRepository::find_by_id(schedule_id)`。若不存在，返回`NotFound`。
    /// 3. **验证状态:** 检查日程的`outcome`是否为`COMPLETED_ON_DAY`。若是，返回`Conflict`。
    /// 4. **核心操作:** 调用 `TaskScheduleRepository::update_outcome`，将`outcome`更新为`PRESENCE_LOGGED`。
    /// 5. **提交事务。**
    /// 6. **返回:** 返回更新后的`TaskSchedule`对象。
    /// **预期副作用:** 修改`task_schedule`表中的一条记录。
    pub async fn log_presence(&self, schedule_id: Uuid) -> AppResult<TaskSchedule> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证日程
        let all_schedules = self
            .task_schedule_repository
            .find_by_date_range(
                DateTime::from_timestamp(0, 0).unwrap().with_timezone(&Utc),
                DateTime::from_timestamp(4000000000, 0)
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .await?;

        let schedule = all_schedules
            .into_iter()
            .find(|s| s.id == schedule_id)
            .ok_or_else(|| AppError::not_found("TaskSchedule", schedule_id.to_string()))?;

        // 3. 验证状态
        if schedule.outcome == Outcome::CompletedOnDay {
            return Err(AppError::conflict("无法为已完成的日程记录努力"));
        }

        // 4. 核心操作
        let updated_schedule = self
            .task_schedule_repository
            .update_outcome(&mut tx, schedule_id, Outcome::PresenceLogged)
            .await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 6. 返回
        Ok(updated_schedule)
    }

    /// 获取指定日期的任务日程
    pub async fn get_schedules_for_day(&self, day: DateTime<Utc>) -> AppResult<Vec<TaskSchedule>> {
        let normalized_day = normalize_to_day_start(day);
        self.task_schedule_repository
            .find_by_day(normalized_day)
            .await
            .map_err(AppError::from)
    }

    /// 获取任务的所有日程
    pub async fn get_task_schedules(&self, task_id: Uuid) -> AppResult<Vec<TaskSchedule>> {
        self.task_schedule_repository
            .find_all_for_task(task_id)
            .await
            .map_err(AppError::from)
    }

    /// 获取日程统计
    pub async fn get_schedule_statistics(
        &self,
    ) -> AppResult<crate::repositories::ScheduleCountByOutcome> {
        self.task_schedule_repository
            .count_by_outcome()
            .await
            .map_err(AppError::from)
    }
}

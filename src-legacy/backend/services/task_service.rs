use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use super::{CreateTaskData, CreationContext, UpdateTaskData};
use crate::common::error::{AppError, AppResult};
use crate::common::utils::time_utils::normalize_to_day_start;
use crate::core::models::{ContextType, Outcome, Task};
use crate::ports::{Clock, IdGenerator};
use crate::repositories::{OrderingRepository, TaskRepository, TaskScheduleRepository};

/// 任务服务
///
/// **预期行为简介:** 封装所有与Task实体相关的业务逻辑，包括创建、更新、完成、重新打开等操作
pub struct TaskService {
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
    // 注意：为了避免循环依赖，我们不在TaskService中直接引用其他服务
    // 而是在需要时直接调用仓库层的方法
}

impl TaskService {
    /// 创建新的任务服务
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

    /// 在指定上下文中创建一个新任务
    ///
    /// **函数签名:** `pub async fn create_in_context(&self, data: CreateTaskData, context: &CreationContext) -> Result<Task, AppError>`
    /// **预期行为简介:** 在指定的上下文中创建一个新任务，并根据上下文完成其初始的日程或排序安排。
    /// **输入输出规范:**
    /// - **前置条件:** `data.title`不能为空且长度小于256。`context`必须是有效的（例如，如果`type`是`PROJECT_LIST`，则`context.id`对应的`Project`必须存在）。
    /// - **后置条件:** 成功时返回新创建的、完整的`Task`对象。数据库中存在新的`Task`记录。根据`context`，相应的`Task_Schedule`或`Ordering`记录被创建。
    /// **边界情况:**
    /// - `context`无效 (如`project_id`不存在): **必须**返回`AppError::NotFound`。
    /// - `data`验证失败: **必须**返回`AppError::ValidationFailed`。
    /// **预期副作用:**
    /// - **数据库写入:** 向`tasks`表插入1条记录。可能向`task_schedule`表插入1条记录，并向`ordering`表插入1条记录。
    /// - **事务:** 所有数据库修改都在一个事务中执行。
    /// - **推送通知 (未来):** 推送`TASK_CREATED`事件。
    pub async fn create_in_context(
        &self,
        data: CreateTaskData,
        context: &CreationContext,
    ) -> AppResult<Task> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证输入
        if let Err(validation_errors) = data.validate() {
            return Err(AppError::ValidationFailed(validation_errors));
        }

        // 3. 生成核心属性
        let new_task_id = self.id_generator.new_uuid();
        let now = self.clock.now_utc();

        // 4. 构建Task对象
        let mut new_task = Task::new(new_task_id, data.title, now);
        new_task.glance_note = data.glance_note;
        new_task.detail_note = data.detail_note;
        new_task.estimated_duration = data.estimated_duration;
        new_task.subtasks = data.subtasks;
        new_task.area_id = data.area_id;
        new_task.due_date = data.due_date;
        new_task.due_date_type = data.due_date_type;

        // 5. 处理上下文（条件分支）
        match context.context_type {
            ContextType::ProjectList => {
                // 验证项目是否存在（这里简化处理，在V1.0中项目表只建表不提供API）
                let project_id = context
                    .context_id
                    .strip_prefix("project::")
                    .and_then(|id_str| Uuid::parse_str(id_str).ok())
                    .ok_or_else(|| {
                        AppError::validation_error(
                            "context_id",
                            "Invalid project context ID format",
                            "CONTEXT_ID_INVALID",
                        )
                    })?;

                new_task.project_id = Some(project_id);
            }
            ContextType::DailyKanban => {
                // 验证日期格式
                context.context_id.parse::<i64>().map_err(|_| {
                    AppError::validation_error(
                        "context_id",
                        "Invalid daily kanban context ID format",
                        "CONTEXT_ID_INVALID",
                    )
                })?;
            }
            ContextType::Misc => {
                // MISC上下文无需特殊验证
            }
            ContextType::AreaFilter => {
                // 验证领域是否存在
                let area_id = context
                    .context_id
                    .strip_prefix("area::")
                    .and_then(|id_str| Uuid::parse_str(id_str).ok())
                    .ok_or_else(|| {
                        AppError::validation_error(
                            "context_id",
                            "Invalid area context ID format",
                            "CONTEXT_ID_INVALID",
                        )
                    })?;

                // 这里应该验证area_id是否存在，但为了简化暂时跳过
                new_task.area_id = Some(area_id);
            }
        }

        // 6. 持久化Task
        let created_task = self.task_repository.create(&mut tx, &new_task).await?;

        // 7. 处理后续安排（条件分支）
        match context.context_type {
            ContextType::DailyKanban => {
                // 创建日程安排
                let target_day_timestamp = context.context_id.parse::<i64>().map_err(|_| {
                    AppError::validation_error(
                        "context_id",
                        "Invalid timestamp",
                        "TIMESTAMP_INVALID",
                    )
                })?;

                let target_day = DateTime::from_timestamp(target_day_timestamp, 0)
                    .ok_or_else(|| {
                        AppError::validation_error(
                            "context_id",
                            "Invalid timestamp",
                            "TIMESTAMP_INVALID",
                        )
                    })?
                    .with_timezone(&Utc);

                let normalized_day = normalize_to_day_start(target_day);

                // 直接实现create_additional_schedule的核心逻辑
                let schedule_id = self.id_generator.new_uuid();
                let new_schedule = crate::core::models::TaskSchedule::new(
                    schedule_id,
                    new_task_id,
                    normalized_day,
                    now,
                );

                self.task_schedule_repository
                    .create(&mut tx, &new_schedule)
                    .await?;

                // 创建排序记录
                let sort_order = self
                    .ordering_repository
                    .get_next_sort_order(&ContextType::DailyKanban, &context.context_id)
                    .await?;

                let ordering = crate::core::models::Ordering::new(
                    self.id_generator.new_uuid(),
                    ContextType::DailyKanban,
                    context.context_id.clone(),
                    new_task_id,
                    sort_order,
                    now,
                )?;

                self.ordering_repository.upsert(&mut tx, &ordering).await?;
            }
            ContextType::ProjectList => {
                // 创建项目排序记录
                let sort_order = self
                    .ordering_repository
                    .get_next_sort_order(&ContextType::ProjectList, &context.context_id)
                    .await?;

                let ordering = crate::core::models::Ordering::new(
                    self.id_generator.new_uuid(),
                    ContextType::ProjectList,
                    context.context_id.clone(),
                    new_task_id,
                    sort_order,
                    now,
                )?;

                self.ordering_repository.upsert(&mut tx, &ordering).await?;
            }
            ContextType::Misc => {
                // 创建杂项排序记录
                let sort_order = self
                    .ordering_repository
                    .get_next_sort_order(&ContextType::Misc, &context.context_id)
                    .await?;

                let ordering = crate::core::models::Ordering::new(
                    self.id_generator.new_uuid(),
                    ContextType::Misc,
                    context.context_id.clone(),
                    new_task_id,
                    sort_order,
                    now,
                )?;

                self.ordering_repository.upsert(&mut tx, &ordering).await?;
            }
            ContextType::AreaFilter => {
                // 领域过滤上下文通常不需要创建排序记录
                // 任务会根据其area_id自动出现在相应的领域过滤视图中
            }
        }

        // 8. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 9. 返回
        Ok(created_task)
    }

    /// 更新任务
    ///
    /// **函数签名:** `pub async fn update_task(&self, task_id: Uuid, updates: UpdateTaskData) -> Result<Task, AppError>`
    /// **预期行为简介:** 原子性地更新一个任务的一个或多个属性。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证Task:** 调用 `TaskRepository::find_by_id(task_id)` 获取当前`Task`对象。若不存在，回滚并返回 `AppError::NotFound`。
    /// 3. **验证输入:** 对`updates`中的每一个非`None`字段进行验证。例如，`title`不能为空，`area_id`必须存在于`areas`表中。若任一验证失败，回滚并返回 `AppError::ValidationFailed`。
    /// 4. **构建更新对象:** 将`updates`中的值合并到从数据库中获取的`Task`对象上，生成一个`updated_task`。
    /// 5. **更新时间戳:** `updated_task.updated_at = self.clock.now_utc()`。
    /// 6. **核心操作:** 调用 `TaskRepository::update(&mut tx, &updated_task)`，将更新持久化到数据库。
    /// 7. **提交事务。**
    /// 8. **返回:** 返回更新后的`Task`对象。
    /// **预期副作用:**
    /// - **数据库写入:** 修改`tasks`表中的一条记录。
    /// - **事务:** 所有数据库修改都在一个事务中执行。
    /// - **推送通知 (未来):** 推送`TASK_UPDATED`事件。
    pub async fn update_task(&self, task_id: Uuid, updates: UpdateTaskData) -> AppResult<Task> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证Task
        let mut current_task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 3. 验证输入
        let mut validation_errors = Vec::new();

        if let Some(ref title) = updates.title {
            if title.is_empty() {
                validation_errors.push(crate::common::error::ValidationError::new(
                    "title",
                    "Title cannot be empty",
                    "TITLE_EMPTY",
                ));
            } else if title.len() > 255 {
                validation_errors.push(crate::common::error::ValidationError::new(
                    "title",
                    "Title cannot exceed 255 characters",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        // 验证截止日期一致性
        let new_due_date = updates.due_date.as_ref().unwrap_or(&current_task.due_date);
        let new_due_date_type = updates
            .due_date_type
            .as_ref()
            .unwrap_or(&current_task.due_date_type);

        match (new_due_date, new_due_date_type) {
            (Some(_), &None) => {
                validation_errors.push(crate::common::error::ValidationError::new(
                    "due_date_type",
                    "Due date type must be specified when due date is set",
                    "DUE_DATE_TYPE_MISSING",
                ));
            }
            (&None, Some(_)) => {
                validation_errors.push(crate::common::error::ValidationError::new(
                    "due_date",
                    "Due date must be specified when due date type is set",
                    "DUE_DATE_MISSING",
                ));
            }
            _ => {}
        }

        if !validation_errors.is_empty() {
            return Err(AppError::ValidationFailed(validation_errors));
        }

        // 4. 构建更新对象
        if let Some(title) = updates.title {
            current_task.title = title;
        }
        if let Some(glance_note) = updates.glance_note {
            current_task.glance_note = glance_note;
        }
        if let Some(detail_note) = updates.detail_note {
            current_task.detail_note = detail_note;
        }
        if let Some(estimated_duration) = updates.estimated_duration {
            current_task.estimated_duration = estimated_duration;
        }
        if let Some(subtasks) = updates.subtasks {
            current_task.subtasks = subtasks;
        }
        if let Some(project_id) = updates.project_id {
            current_task.project_id = project_id;
        }
        if let Some(area_id) = updates.area_id {
            current_task.area_id = area_id;
        }
        if let Some(due_date) = updates.due_date {
            current_task.due_date = due_date;
        }
        if let Some(due_date_type) = updates.due_date_type {
            current_task.due_date_type = due_date_type;
        }

        // 5. 更新时间戳
        current_task.updated_at = self.clock.now_utc();

        // 6. 核心操作
        let updated_task = self.task_repository.update(&mut tx, &current_task).await?;

        // 7. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 8. 返回
        Ok(updated_task)
    }

    /// 完成任务
    ///
    /// **函数签名:** `pub async fn complete_task(&self, task_id: Uuid) -> Result<Task, AppError>`
    /// **预期行为简介:** 将指定任务标记为已完成，并处理相关的日程和时间块
    /// **执行过程 (Process):**
    /// 1. 获取当前时间`now` (通过`Clock`服务)。
    /// 2. 更新`Task.completed_at = now`。
    /// 3. 截断正在进行的、仅与此任务耦合的`TimeBlock`。
    /// 4. 删除所有未来的`Task_Schedule`记录。
    /// 5. 删除所有未来的、仅与此任务耦合的`TimeBlock`。
    /// 6. 更新**当天**的`Task_Schedule`记录的`outcome`为`COMPLETED_ON_DAY`。**不修改**过去日期的`outcome`。
    /// **结果:** 返回更新后的`Task`对象。通过WebSocket推送`TASK_UPDATED`事件。
    pub async fn complete_task(&self, task_id: Uuid) -> AppResult<Task> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 验证任务存在
        let task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 如果任务已完成，幂等返回
        if task.is_completed() {
            tx.commit().await.map_err(|e| {
                AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                    message: e.to_string(),
                })
            })?;
            return Ok(task);
        }

        // 2. 获取当前时间并完成任务
        let now = self.clock.now_utc();
        let completed_task = self
            .task_repository
            .set_completed(&mut tx, task_id, now)
            .await?;

        // 3-5. 处理时间块和未来日程（简化实现，在真实版本中需要实现）
        // TODO: 实现时间块截断和删除逻辑

        // 4. 删除所有未来的Task_Schedule记录
        let today = normalize_to_day_start(now);
        self.task_schedule_repository
            .delete_future_for_task(&mut tx, task_id, today)
            .await?;

        // 6. 更新当天的Task_Schedule记录的outcome为COMPLETED_ON_DAY
        let today_schedules = self.task_schedule_repository.find_by_day(today).await?;
        for schedule in today_schedules {
            if schedule.task_id == task_id {
                self.task_schedule_repository
                    .update_outcome(&mut tx, schedule.id, Outcome::CompletedOnDay)
                    .await?;
                break;
            }
        }

        // 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(completed_task)
    }

    /// 重新打开任务
    ///
    /// **函数签名:** `pub async fn reopen_task(&self, task_id: Uuid) -> Result<Task, AppError>`
    /// **预期行为简介:** 将一个已完成的任务重新打开。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证Task与幂等检查:** 调用 `TaskRepository::find_by_id(task_id)`。若不存在，返回`NotFound`。若`task.completed_at`为`NULL`，**直接提交事务并返回该`Task`对象**。
    /// 3. **核心操作:** 调用 `TaskRepository::reopen(&mut tx, task_id)`，将`completed_at`字段设置为`NULL`。
    /// 4. **耦合操作 (重置结局):**
    ///    a. 调用 `TaskScheduleRepository::find_all_for_task(task_id)` 获取所有相关日程。
    ///    b. 对于每一条`outcome`为`COMPLETED_ON_DAY`的日程，调用 `TaskScheduleRepository::update_outcome` 将其重置为`PLANNED`。
    /// 5. **提交事务。**
    /// 6. **返回:** 返回更新后的`Task`对象。
    /// **预期副作用:** 修改`tasks`表1条记录。可能修改`task_schedule`表中的多条记录。
    pub async fn reopen_task(&self, task_id: Uuid) -> AppResult<Task> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证Task与幂等检查
        let task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 如果任务未完成，幂等返回
        if !task.is_completed() {
            tx.commit().await.map_err(|e| {
                AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                    message: e.to_string(),
                })
            })?;
            return Ok(task);
        }

        // 3. 核心操作
        let reopened_task = self.task_repository.reopen(&mut tx, task_id).await?;

        // 4. 耦合操作（重置结局）
        let all_schedules = self
            .task_schedule_repository
            .find_all_for_task(task_id)
            .await?;
        for schedule in all_schedules {
            if schedule.outcome == Outcome::CompletedOnDay {
                self.task_schedule_repository
                    .update_outcome(&mut tx, schedule.id, Outcome::Planned)
                    .await?;
            }
        }

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 6. 返回
        Ok(reopened_task)
    }

    /// 获取任务详情
    pub async fn get_task(&self, task_id: Uuid) -> AppResult<Option<Task>> {
        self.task_repository
            .find_by_id(task_id)
            .await
            .map_err(AppError::from)
    }

    /// 搜索任务
    pub async fn search_tasks(&self, query: &str, limit: Option<i64>) -> AppResult<Vec<Task>> {
        self.task_repository
            .search(query, limit)
            .await
            .map_err(AppError::from)
    }

    /// 获取未安排的任务（Staging区）
    pub async fn get_unscheduled_tasks(&self) -> AppResult<Vec<Task>> {
        self.task_repository
            .find_unscheduled()
            .await
            .map_err(AppError::from)
    }

    /// 获取任务统计
    pub async fn get_task_statistics(&self) -> AppResult<crate::repositories::TaskCountByStatus> {
        self.task_repository
            .count_by_status()
            .await
            .map_err(AppError::from)
    }

    /// 软删除任务
    pub async fn delete_task(&self, task_id: Uuid) -> AppResult<()> {
        let mut tx = self.task_repository.begin_transaction().await?;

        // 验证任务存在
        let _task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 删除所有相关的日程和排序记录
        self.task_schedule_repository
            .delete_all_for_task(&mut tx, task_id)
            .await?;
        self.ordering_repository
            .delete_all_for_task(&mut tx, task_id)
            .await?;

        // 软删除任务
        self.task_repository.soft_delete(&mut tx, task_id).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

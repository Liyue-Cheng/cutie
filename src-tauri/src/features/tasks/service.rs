/// 任务业务逻辑层
///
/// 实现任务的业务逻辑和规则
use chrono::Utc;
use uuid::Uuid;

use crate::shared::{
    core::{AppError, AppResult, Task},
    database::TaskRepository,
};

use super::payloads::{CreateTaskPayload, TaskStatsResponse, UpdateTaskPayload};

/// 任务服务
pub struct TaskService<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> TaskService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 创建新任务
    pub async fn create_task(&self, payload: CreateTaskPayload) -> AppResult<Task> {
        let now = Utc::now();
        let task_id = Uuid::new_v4();

        let mut task = Task::new(task_id, payload.title, now);

        // 设置可选字段
        if let Some(glance_note) = payload.glance_note {
            task.glance_note = Some(glance_note);
        }
        if let Some(detail_note) = payload.detail_note {
            task.detail_note = Some(detail_note);
        }
        if let Some(duration) = payload.estimated_duration {
            task.estimated_duration = Some(duration);
        }
        if let Some(subtasks) = payload.subtasks {
            task.subtasks = Some(subtasks);
        }
        if let Some(area_id) = payload.area_id {
            task.area_id = Some(area_id);
        }
        if let Some(due_date) = payload.due_date {
            task.due_date = Some(due_date);
            task.due_date_type = payload.due_date_type;
        }

        // 创建任务
        let created_task = self.repository.create(&task).await?;

        // TODO: 在相应的上下文中创建排序记录
        // 这里需要调用排序服务来处理排序逻辑

        Ok(created_task)
    }

    /// 根据ID获取任务
    pub async fn get_task(&self, id: Uuid) -> AppResult<Task> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", id.to_string()))
    }

    /// 更新任务
    pub async fn update_task(&self, id: Uuid, payload: UpdateTaskPayload) -> AppResult<Task> {
        let mut task = self.get_task(id).await?;
        let now = Utc::now();

        // 更新字段
        if let Some(title) = payload.title {
            task.title = title;
        }
        if let Some(glance_note) = payload.glance_note {
            task.glance_note = glance_note;
        }
        if let Some(detail_note) = payload.detail_note {
            task.detail_note = detail_note;
        }
        if let Some(estimated_duration) = payload.estimated_duration {
            task.estimated_duration = estimated_duration;
        }
        if let Some(subtasks) = payload.subtasks {
            task.subtasks = subtasks;
        }
        if let Some(project_id) = payload.project_id {
            task.project_id = project_id;
        }
        if let Some(area_id) = payload.area_id {
            task.area_id = area_id;
        }
        if let Some(due_date) = payload.due_date {
            task.due_date = due_date;
        }
        if let Some(due_date_type) = payload.due_date_type {
            task.due_date_type = due_date_type;
        }

        task.updated_at = now;

        self.repository.update(&task).await
    }

    /// 删除任务
    pub async fn delete_task(&self, id: Uuid) -> AppResult<()> {
        // 检查任务是否存在
        self.get_task(id).await?;

        // 软删除任务
        self.repository.delete(id).await?;

        // TODO: 清理相关的日程和排序记录
        // 这里需要调用其他服务来清理关联数据

        Ok(())
    }

    /// 完成任务
    pub async fn complete_task(&self, id: Uuid) -> AppResult<Task> {
        let mut task = self.get_task(id).await?;
        let now = Utc::now();

        if task.is_completed() {
            return Err(AppError::conflict("任务已经完成"));
        }

        task.complete(now);
        self.repository.update(&task).await
    }

    /// 重新打开任务
    pub async fn reopen_task(&self, id: Uuid) -> AppResult<Task> {
        let mut task = self.get_task(id).await?;
        let now = Utc::now();

        if !task.is_completed() {
            return Err(AppError::conflict("任务尚未完成"));
        }

        task.reopen(now);
        self.repository.update(&task).await
    }

    /// 搜索任务
    pub async fn search_tasks(
        &self,
        query: Option<String>,
        limit: Option<usize>,
    ) -> AppResult<Vec<Task>> {
        match query {
            Some(q) if !q.trim().is_empty() => self.repository.search(q.trim(), limit).await,
            _ => {
                // 如果没有搜索词，返回未安排的任务
                self.repository.find_unscheduled().await
            }
        }
    }

    /// 获取未安排的任务
    pub async fn get_unscheduled_tasks(&self) -> AppResult<Vec<Task>> {
        self.repository.find_unscheduled().await
    }

    /// 根据项目ID获取任务
    pub async fn get_tasks_by_project(&self, project_id: Uuid) -> AppResult<Vec<Task>> {
        self.repository.find_by_project_id(project_id).await
    }

    /// 根据领域ID获取任务
    pub async fn get_tasks_by_area(&self, area_id: Uuid) -> AppResult<Vec<Task>> {
        self.repository.find_by_area_id(area_id).await
    }

    /// 获取已完成的任务
    pub async fn get_completed_tasks(&self) -> AppResult<Vec<Task>> {
        self.repository.find_completed().await
    }

    /// 获取任务统计
    pub async fn get_task_stats(&self) -> AppResult<TaskStatsResponse> {
        let stats = self.repository.get_stats().await?;

        // TODO: 实现更详细的统计逻辑
        // 这里可以添加今日、本周、本月的任务统计

        Ok(TaskStatsResponse {
            total_count: stats.total_count,
            completed_count: stats.completed_count,
            pending_count: stats.pending_count,
            overdue_count: stats.overdue_count,
            today_count: 0,      // TODO: 实现今日任务统计
            this_week_count: 0,  // TODO: 实现本周任务统计
            this_month_count: 0, // TODO: 实现本月任务统计
        })
    }

    /// 批量删除任务
    pub async fn bulk_delete_tasks(&self, task_ids: Vec<Uuid>) -> AppResult<usize> {
        let mut deleted_count = 0;

        for task_id in task_ids {
            if self.repository.find_by_id(task_id).await?.is_some() {
                self.repository.delete(task_id).await?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// 批量完成任务
    pub async fn bulk_complete_tasks(&self, task_ids: Vec<Uuid>) -> AppResult<usize> {
        let mut completed_count = 0;
        let now = Utc::now();

        for task_id in task_ids {
            if let Some(mut task) = self.repository.find_by_id(task_id).await? {
                if !task.is_completed() {
                    task.complete(now);
                    self.repository.update(&task).await?;
                    completed_count += 1;
                }
            }
        }

        Ok(completed_count)
    }

    /// 批量重新打开任务
    pub async fn bulk_reopen_tasks(&self, task_ids: Vec<Uuid>) -> AppResult<usize> {
        let mut reopened_count = 0;
        let now = Utc::now();

        for task_id in task_ids {
            if let Some(mut task) = self.repository.find_by_id(task_id).await? {
                if task.is_completed() {
                    task.reopen(now);
                    self.repository.update(&task).await?;
                    reopened_count += 1;
                }
            }
        }

        Ok(reopened_count)
    }

    /// 验证任务数据的业务规则
    fn validate_task_business_rules(&self, task: &Task) -> AppResult<()> {
        // 验证标题长度
        if task.title.len() > 255 {
            return Err(AppError::validation_error(
                "title",
                "任务标题不能超过255个字符",
                "TITLE_TOO_LONG",
            ));
        }

        // 验证预估时长
        if let Some(duration) = task.estimated_duration {
            if duration < 0 {
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
            if duration > 24 * 60 * 7 {
                // 一周的分钟数
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能超过一周",
                    "DURATION_TOO_LONG",
                ));
            }
        }

        // 验证截止日期
        if let Some(due_date) = task.due_date {
            if due_date < task.created_at {
                return Err(AppError::validation_error(
                    "due_date",
                    "截止日期不能早于创建时间",
                    "DUE_DATE_TOO_EARLY",
                ));
            }
        }

        // 验证子任务数量
        if let Some(subtasks) = &task.subtasks {
            if subtasks.len() > 50 {
                return Err(AppError::validation_error(
                    "subtasks",
                    "子任务数量不能超过50个",
                    "TOO_MANY_SUBTASKS",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::tasks::{payloads::CreationContextPayload, repository::SqlxTaskRepository},
        shared::{core::ContextType, database::connection::create_test_database},
    };

    #[tokio::test]
    async fn test_create_task() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTaskRepository::new(pool);
        let service = TaskService::new(repository);

        let payload = CreateTaskPayload {
            title: "Test Task".to_string(),
            glance_note: Some("Test note".to_string()),
            detail_note: None,
            estimated_duration: Some(60),
            subtasks: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            context: CreationContextPayload {
                context_type: ContextType::Misc,
                context_id: "floating".to_string(),
            },
        };

        let task = service.create_task(payload).await.unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.glance_note, Some("Test note".to_string()));
        assert_eq!(task.estimated_duration, Some(60));
    }

    #[tokio::test]
    async fn test_complete_and_reopen_task() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTaskRepository::new(pool);
        let service = TaskService::new(repository);

        // 创建任务
        let payload = CreateTaskPayload {
            title: "Test Task".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            subtasks: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            context: CreationContextPayload {
                context_type: ContextType::Misc,
                context_id: "floating".to_string(),
            },
        };

        let task = service.create_task(payload).await.unwrap();
        assert!(!task.is_completed());

        // 完成任务
        let completed_task = service.complete_task(task.id).await.unwrap();
        assert!(completed_task.is_completed());

        // 重新打开任务
        let reopened_task = service.reopen_task(task.id).await.unwrap();
        assert!(!reopened_task.is_completed());
    }

    #[tokio::test]
    async fn test_bulk_operations() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTaskRepository::new(pool);
        let service = TaskService::new(repository);

        // 创建多个任务
        let mut task_ids = Vec::new();
        for i in 0..3 {
            let payload = CreateTaskPayload {
                title: format!("Test Task {}", i),
                glance_note: None,
                detail_note: None,
                estimated_duration: None,
                subtasks: None,
                area_id: None,
                due_date: None,
                due_date_type: None,
                context: CreationContextPayload {
                    context_type: ContextType::Misc,
                    context_id: "floating".to_string(),
                },
            };

            let task = service.create_task(payload).await.unwrap();
            task_ids.push(task.id);
        }

        // 批量完成任务
        let completed_count = service.bulk_complete_tasks(task_ids.clone()).await.unwrap();
        assert_eq!(completed_count, 3);

        // 批量重新打开任务
        let reopened_count = service.bulk_reopen_tasks(task_ids.clone()).await.unwrap();
        assert_eq!(reopened_count, 3);

        // 批量删除任务
        let deleted_count = service.bulk_delete_tasks(task_ids).await.unwrap();
        assert_eq!(deleted_count, 3);
    }
}

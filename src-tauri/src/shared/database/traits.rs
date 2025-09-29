use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::core::{AppResult, Task, Area, TaskSchedule, Template, TimeBlock, Ordering};

/// 通用仓库trait
/// 
/// 定义了所有实体仓库的基本CRUD操作
#[async_trait]
pub trait Repository<T> {
    /// 根据ID查找实体
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<T>>;
    
    /// 创建新实体
    async fn create(&self, entity: &T) -> AppResult<T>;
    
    /// 更新实体
    async fn update(&self, entity: &T) -> AppResult<T>;
    
    /// 删除实体（软删除）
    async fn delete(&self, id: Uuid) -> AppResult<()>;
    
    /// 获取所有实体
    async fn find_all(&self) -> AppResult<Vec<T>>;
    
    /// 检查实体是否存在
    async fn exists(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}

/// 任务仓库trait
#[async_trait]
pub trait TaskRepository: Repository<Task> {
    /// 搜索任务
    async fn search(&self, query: &str, limit: Option<usize>) -> AppResult<Vec<Task>>;
    
    /// 获取未安排的任务
    async fn find_unscheduled(&self) -> AppResult<Vec<Task>>;
    
    /// 根据项目ID查找任务
    async fn find_by_project_id(&self, project_id: Uuid) -> AppResult<Vec<Task>>;
    
    /// 根据领域ID查找任务
    async fn find_by_area_id(&self, area_id: Uuid) -> AppResult<Vec<Task>>;
    
    /// 获取已完成的任务
    async fn find_completed(&self) -> AppResult<Vec<Task>>;
    
    /// 获取任务统计
    async fn get_stats(&self) -> AppResult<TaskStats>;
}

/// 任务统计
#[derive(Debug, Clone)]
pub struct TaskStats {
    pub total_count: i64,
    pub completed_count: i64,
    pub pending_count: i64,
    pub overdue_count: i64,
}

/// 领域仓库trait
#[async_trait]
pub trait AreaRepository: Repository<Area> {
    /// 获取根领域
    async fn find_roots(&self) -> AppResult<Vec<Area>>;
    
    /// 根据父ID查找子领域
    async fn find_by_parent_id(&self, parent_id: Option<Uuid>) -> AppResult<Vec<Area>>;
    
    /// 获取领域路径（从根到指定领域）
    async fn get_path(&self, area_id: Uuid) -> AppResult<Vec<Area>>;
    
    /// 获取所有后代领域
    async fn get_descendants(&self, area_id: Uuid) -> AppResult<Vec<Area>>;
    
    /// 检查是否可以删除（没有被使用）
    async fn can_delete(&self, area_id: Uuid) -> AppResult<bool>;
}

/// 任务日程仓库trait
#[async_trait]
pub trait TaskScheduleRepository: Repository<TaskSchedule> {
    /// 根据任务ID查找日程
    async fn find_by_task_id(&self, task_id: Uuid) -> AppResult<Vec<TaskSchedule>>;
    
    /// 根据日期查找日程
    async fn find_by_date(&self, date: chrono::DateTime<chrono::Utc>) -> AppResult<Vec<TaskSchedule>>;
    
    /// 根据日期范围查找日程
    async fn find_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<Vec<TaskSchedule>>;
    
    /// 删除任务的所有日程
    async fn delete_by_task_id(&self, task_id: Uuid) -> AppResult<()>;
}

/// 模板仓库trait
#[async_trait]
pub trait TemplateRepository: Repository<Template> {
    /// 根据名称搜索模板
    async fn search_by_name(&self, query: &str) -> AppResult<Vec<Template>>;
    
    /// 根据领域ID查找模板
    async fn find_by_area_id(&self, area_id: Uuid) -> AppResult<Vec<Template>>;
    
    /// 查找包含特定变量的模板
    async fn find_with_variable(&self, variable_name: &str) -> AppResult<Vec<Template>>;
}

/// 时间块仓库trait
#[async_trait]
pub trait TimeBlockRepository: Repository<TimeBlock> {
    /// 根据日期查找时间块
    async fn find_by_date(&self, date: chrono::DateTime<chrono::Utc>) -> AppResult<Vec<TimeBlock>>;
    
    /// 根据日期范围查找时间块
    async fn find_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<Vec<TimeBlock>>;
    
    /// 根据任务ID查找时间块
    async fn find_by_task_id(&self, task_id: Uuid) -> AppResult<Vec<TimeBlock>>;
    
    /// 根据领域ID查找时间块
    async fn find_by_area_id(&self, area_id: Uuid) -> AppResult<Vec<TimeBlock>>;
    
    /// 检查时间冲突
    async fn check_conflict(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        exclude_id: Option<Uuid>,
    ) -> AppResult<bool>;
    
    /// 查找空闲时间段
    async fn find_free_slots(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        min_duration_minutes: i32,
    ) -> AppResult<Vec<FreeTimeSlot>>;
}

/// 空闲时间段
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FreeTimeSlot {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration_minutes: i32,
}

/// 排序仓库trait
#[async_trait]
pub trait OrderingRepository: Repository<Ordering> {
    /// 根据上下文查找排序记录
    async fn find_by_context(
        &self,
        context_type: &crate::shared::core::ContextType,
        context_id: &str,
    ) -> AppResult<Vec<Ordering>>;
    
    /// 根据任务ID查找排序记录
    async fn find_by_task_id(&self, task_id: Uuid) -> AppResult<Vec<Ordering>>;
    
    /// 更新排序记录
    async fn update_sort_order(
        &self,
        context_type: &crate::shared::core::ContextType,
        context_id: &str,
        task_id: Uuid,
        new_sort_order: &str,
    ) -> AppResult<()>;
    
    /// 批量更新排序记录
    async fn batch_update(&self, orderings: &[Ordering]) -> AppResult<()>;
    
    /// 清理上下文中的排序记录
    async fn clear_context(
        &self,
        context_type: &crate::shared::core::ContextType,
        context_id: &str,
    ) -> AppResult<()>;
    
    /// 删除任务的所有排序记录
    async fn delete_by_task_id(&self, task_id: Uuid) -> AppResult<()>;
}

/// 时间块任务关联仓库trait
#[async_trait]
pub trait TimeBlockTaskRepository {
    /// 链接任务到时间块
    async fn link_task(&self, time_block_id: Uuid, task_id: Uuid) -> AppResult<()>;
    
    /// 取消任务与时间块的关联
    async fn unlink_task(&self, time_block_id: Uuid, task_id: Uuid) -> AppResult<()>;
    
    /// 获取时间块关联的任务
    async fn get_tasks_for_block(&self, time_block_id: Uuid) -> AppResult<Vec<Task>>;
    
    /// 获取任务关联的时间块
    async fn get_blocks_for_task(&self, task_id: Uuid) -> AppResult<Vec<TimeBlock>>;
    
    /// 清理时间块的所有任务关联
    async fn clear_block_tasks(&self, time_block_id: Uuid) -> AppResult<()>;
    
    /// 清理任务的所有时间块关联
    async fn clear_task_blocks(&self, task_id: Uuid) -> AppResult<()>;
}

/// 事务trait
/// 
/// 提供事务支持的仓库操作
#[async_trait]
pub trait TransactionalRepository {
    type Transaction;
    
    /// 开始事务
    async fn begin_transaction(&self) -> AppResult<Self::Transaction>;
    
    /// 提交事务
    async fn commit_transaction(&self, tx: Self::Transaction) -> AppResult<()>;
    
    /// 回滚事务
    async fn rollback_transaction(&self, tx: Self::Transaction) -> AppResult<()>;
}

/// 批量操作trait
#[async_trait]
pub trait BulkOperations<T> {
    /// 批量创建
    async fn bulk_create(&self, entities: &[T]) -> AppResult<Vec<T>>;
    
    /// 批量更新
    async fn bulk_update(&self, entities: &[T]) -> AppResult<Vec<T>>;
    
    /// 批量删除
    async fn bulk_delete(&self, ids: &[Uuid]) -> AppResult<()>;
}

/// 分页查询trait
#[async_trait]
pub trait PaginatedQuery<T> {
    /// 分页查询
    async fn find_paginated(
        &self,
        page: u32,
        page_size: u32,
    ) -> AppResult<crate::shared::database::PaginatedResult<T>>;
    
    /// 带条件的分页查询
    async fn find_paginated_with_filter(
        &self,
        page: u32,
        page_size: u32,
        filter: &str,
    ) -> AppResult<crate::shared::database::PaginatedResult<T>>;
}


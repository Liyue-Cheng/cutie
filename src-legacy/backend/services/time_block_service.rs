use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use super::{CreateTimeBlockData, UpdateTimeBlockData};
use crate::common::error::{AppError, AppResult};
use crate::core::models::TimeBlock;
use crate::ports::{Clock, IdGenerator};
use crate::repositories::{TaskRepository, TimeBlockRepository};

/// 时间块服务
///
/// **预期行为简介:** 封装所有与TimeBlock相关的业务逻辑，包括创建、更新、删除时间块以及任务关联管理
pub struct TimeBlockService {
    /// 时钟服务
    clock: Arc<dyn Clock>,

    /// ID生成器
    id_generator: Arc<dyn IdGenerator>,

    /// 任务仓库
    task_repository: Arc<dyn TaskRepository>,

    /// 时间块仓库
    time_block_repository: Arc<dyn TimeBlockRepository>,
}

impl TimeBlockService {
    /// 创建新的时间块服务
    pub fn new(
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        task_repository: Arc<dyn TaskRepository>,
        time_block_repository: Arc<dyn TimeBlockRepository>,
    ) -> Self {
        Self {
            clock,
            id_generator,
            task_repository,
            time_block_repository,
        }
    }

    /// 创建时间块
    ///
    /// **函数签名:** `pub async fn create_time_block(&self, data: CreateTimeBlockData) -> Result<TimeBlock, AppError>`
    /// **预期行为简介:** 创建一个新的时间块，并选择性地链接一个或多个任务。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证输入:** 检查`data.start_time`必须小于等于`data.end_time`。验证`data.area_id`和所有`data.task_ids`的有效性。若失败，返回`ValidationFailed`或`NotFound`。
    /// 3. **生成ID和时间:** `new_block_id = self.id_generator.new_uuid()`，`now = self.clock.now_utc()`。
    /// 4. **核心操作 (创建TimeBlock):** 调用`TimeBlockRepository::create`创建新的`TimeBlock`记录。
    /// 5. **耦合操作 (链接Task):**
    ///    a. 遍历`data.task_ids`。
    ///    b. 对于每一个`task_id`，调用`self.link_task_to_block`的核心逻辑，在`task_time_block_link`表中创建一条链接记录。
    /// 6. **提交事务。**
    /// 7. **返回:** 返回新创建的`TimeBlock`对象。
    /// **预期副作用:** 向`time_blocks`表插入1条记录。可能向`task_time_block_link`表插入多条记录。
    pub async fn create_time_block(&self, data: CreateTimeBlockData) -> AppResult<TimeBlock> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证输入
        if let Err(validation_errors) = data.validate() {
            return Err(AppError::ValidationFailed(validation_errors));
        }

        // 验证所有task_ids的有效性
        for &task_id in &data.task_ids {
            let _task = self
                .task_repository
                .find_by_id(task_id)
                .await?
                .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        }

        // 3. 生成ID和时间
        let new_block_id = self.id_generator.new_uuid();
        let now = self.clock.now_utc();

        // 4. 核心操作（创建TimeBlock）
        let new_time_block = TimeBlock::new(new_block_id, data.start_time, data.end_time, now)?;
        let mut time_block = new_time_block;
        time_block.title = data.title;
        time_block.glance_note = data.glance_note;
        time_block.detail_note = data.detail_note;
        time_block.area_id = data.area_id;

        let created_time_block = self
            .time_block_repository
            .create(&mut tx, &time_block)
            .await?;

        // 5. 耦合操作（链接Task）
        for &task_id in &data.task_ids {
            self.time_block_repository
                .link_task(&mut tx, new_block_id, task_id)
                .await?;
        }

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 7. 返回
        Ok(created_time_block)
    }

    /// 更新时间块
    ///
    /// **函数签名:** `pub async fn update_time_block(&self, block_id: Uuid, updates: UpdateTimeBlockData) -> Result<TimeBlock, AppError>`
    /// **预期行为简介:** 更新一个时间块的属性，如起止时间、标题、笔记、Area等。
    /// **执行过程 (Process):** (同`TaskService::update_task`的逻辑)
    /// 1. 启动事务。
    /// 2. 查找并验证`TimeBlock`存在。
    /// 3. 验证`updates`数据（如`start_time <= end_time`）。
    /// 4. 合并更新，设置`updated_at`。
    /// 5. 调用`TimeBlockRepository::update`。
    /// 6. 提交事务。
    /// 7. 返回更新后的`TimeBlock`。
    /// **预期副作用:** 修改`time_blocks`表中的一条记录。
    pub async fn update_time_block(
        &self,
        block_id: Uuid,
        updates: UpdateTimeBlockData,
    ) -> AppResult<TimeBlock> {
        // 1. 启动事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 查找并验证TimeBlock存在
        let mut current_block = self
            .time_block_repository
            .find_by_id(block_id)
            .await?
            .ok_or_else(|| AppError::not_found("TimeBlock", block_id.to_string()))?;

        // 3. 验证updates数据
        let mut validation_errors = Vec::new();

        let new_start_time = updates.start_time.unwrap_or(current_block.start_time);
        let new_end_time = updates.end_time.unwrap_or(current_block.end_time);

        if new_start_time >= new_end_time {
            validation_errors.push(crate::common::error::ValidationError::new(
                "time_range",
                "Start time must be before end time",
                "TIME_RANGE_INVALID",
            ));
        }

        if !validation_errors.is_empty() {
            return Err(AppError::ValidationFailed(validation_errors));
        }

        // 4. 合并更新，设置updated_at
        if let Some(title) = updates.title {
            current_block.title = title;
        }
        if let Some(glance_note) = updates.glance_note {
            current_block.glance_note = glance_note;
        }
        if let Some(detail_note) = updates.detail_note {
            current_block.detail_note = detail_note;
        }
        if let Some(start_time) = updates.start_time {
            current_block.start_time = start_time;
        }
        if let Some(end_time) = updates.end_time {
            current_block.end_time = end_time;
        }
        if let Some(area_id) = updates.area_id {
            current_block.area_id = area_id;
        }

        current_block.updated_at = self.clock.now_utc();

        // 5. 调用TimeBlockRepository::update
        let updated_block = self
            .time_block_repository
            .update(&mut tx, &current_block)
            .await?;

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 7. 返回更新后的TimeBlock
        Ok(updated_block)
    }

    /// 删除时间块
    ///
    /// **函数签名:** `pub async fn delete_time_block(&self, block_id: Uuid) -> Result<(), AppError>`
    /// **预期行为简介:** 删除一个时间块及其所有关联。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证:** 检查`block_id`是否存在。若不存在，幂等地返回成功。
    /// 3. **核心操作:** 调用 `TimeBlockRepository::delete(block_id)` (软删除)。
    /// 4. **耦合操作:** 调用`TimeBlockRepository::delete_all_links(block_id)`，删除`task_time_block_link`表中所有与该`block_id`相关的记录。
    /// 5. **提交事务。**
    /// 6. **返回:** `Ok(())`。
    /// **预期副作用:** 修改`time_blocks`表1条记录（`is_deleted=true`）。删除`task_time_block_link`表中的多条记录。
    pub async fn delete_time_block(&self, block_id: Uuid) -> AppResult<()> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证
        let time_block = self.time_block_repository.find_by_id(block_id).await?;
        if time_block.is_none() {
            // 幂等地返回成功
            tx.commit().await.map_err(|e| {
                AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                    message: e.to_string(),
                })
            })?;
            return Ok(());
        }

        // 4. 耦合操作：删除所有任务链接
        self.time_block_repository
            .unlink_all_tasks(&mut tx, block_id)
            .await?;

        // 3. 核心操作：软删除时间块
        self.time_block_repository
            .soft_delete(&mut tx, block_id)
            .await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 6. 返回
        Ok(())
    }

    /// 链接任务到时间块
    ///
    /// **函数签名:** `pub async fn link_task_to_block(&self, block_id: Uuid, task_id: Uuid) -> Result<(), AppError>`
    /// **执行过程:**
    /// 1. 启动事务。
    /// 2. 验证`block_id`和`task_id`都存在。
    /// 3. 幂等检查：检查`task_time_block_link`中是否已存在该链接。若存在，直接成功返回。
    /// 4. 核心操作：在`task_time_block_link`表中插入新记录。
    /// 5. **AI副作用触发 (异步):**
    ///    a. 获取该`TimeBlock`及其所有关联`Task`的`Area`信息。
    ///    b. 如果Area不唯一，则异步调用`AiService::deduce_time_block_area`。
    /// 6. 提交事务。返回`Ok(())`。
    pub async fn link_task_to_block(&self, block_id: Uuid, task_id: Uuid) -> AppResult<()> {
        // 1. 启动事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证block_id和task_id都存在
        let _time_block = self
            .time_block_repository
            .find_by_id(block_id)
            .await?
            .ok_or_else(|| AppError::not_found("TimeBlock", block_id.to_string()))?;

        let _task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 3. 幂等检查：检查链接是否已存在
        let existing_links = self.time_block_repository.find_by_task_id(task_id).await?;
        if existing_links.iter().any(|tb| tb.id == block_id) {
            // 链接已存在，直接成功返回
            tx.commit().await.map_err(|e| {
                AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                    message: e.to_string(),
                })
            })?;
            return Ok(());
        }

        // 4. 核心操作：创建链接记录
        self.time_block_repository
            .link_task(&mut tx, block_id, task_id)
            .await?;

        // 5. AI副作用触发（简化实现，在V1.0中暂时跳过AI功能）
        // TODO: 实现AI Area推导逻辑

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }

    /// 取消任务与时间块的关联
    ///
    /// **函数签名:** `pub async fn unlink_task_from_block(&self, block_id: Uuid, task_id: Uuid) -> Result<(), AppError>`
    /// **执行过程:**
    /// 1. 启动事务。
    /// 2. 核心操作：从`task_time_block_link`表中删除对应的链接记录。
    /// 3. 提交事务。返回`Ok(())`。
    pub async fn unlink_task_from_block(&self, block_id: Uuid, task_id: Uuid) -> AppResult<()> {
        // 1. 启动事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 核心操作：删除链接记录
        self.time_block_repository
            .unlink_task(&mut tx, block_id, task_id)
            .await?;

        // 3. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }

    /// 获取时间块详情
    pub async fn get_time_block(&self, block_id: Uuid) -> AppResult<Option<TimeBlock>> {
        self.time_block_repository
            .find_by_id(block_id)
            .await
            .map_err(AppError::from)
    }

    /// 获取指定日期的时间块
    pub async fn get_time_blocks_for_date(&self, date: DateTime<Utc>) -> AppResult<Vec<TimeBlock>> {
        self.time_block_repository
            .find_by_date(date)
            .await
            .map_err(AppError::from)
    }

    /// 获取日期范围内的时间块
    pub async fn get_time_blocks_for_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<Vec<TimeBlock>> {
        self.time_block_repository
            .find_by_date_range(start_date, end_date)
            .await
            .map_err(AppError::from)
    }

    /// 获取与任务关联的时间块
    pub async fn get_time_blocks_for_task(&self, task_id: Uuid) -> AppResult<Vec<TimeBlock>> {
        self.time_block_repository
            .find_by_task_id(task_id)
            .await
            .map_err(AppError::from)
    }

    /// 检查时间冲突
    pub async fn check_time_conflict(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        exclude_id: Option<Uuid>,
    ) -> AppResult<bool> {
        self.time_block_repository
            .has_time_conflict(start_time, end_time, exclude_id)
            .await
            .map_err(AppError::from)
    }

    /// 查找空闲时间段
    pub async fn find_free_time_slots(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        min_duration_minutes: i32,
    ) -> AppResult<Vec<crate::repositories::FreeTimeSlot>> {
        self.time_block_repository
            .find_free_time_slots(start_time, end_time, min_duration_minutes)
            .await
            .map_err(AppError::from)
    }

    /// 截断时间块
    pub async fn truncate_time_block(
        &self,
        block_id: Uuid,
        truncate_at: DateTime<Utc>,
    ) -> AppResult<TimeBlock> {
        let mut tx = self.task_repository.begin_transaction().await?;

        let truncated_block = self
            .time_block_repository
            .truncate_at(&mut tx, block_id, truncate_at)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(truncated_block)
    }

    /// 扩展时间块
    pub async fn extend_time_block(
        &self,
        block_id: Uuid,
        new_end_time: DateTime<Utc>,
    ) -> AppResult<TimeBlock> {
        let mut tx = self.task_repository.begin_transaction().await?;

        let extended_block = self
            .time_block_repository
            .extend_to(&mut tx, block_id, new_end_time)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(extended_block)
    }

    /// 分割时间块
    pub async fn split_time_block(
        &self,
        block_id: Uuid,
        split_at: DateTime<Utc>,
    ) -> AppResult<(TimeBlock, TimeBlock)> {
        let mut tx = self.task_repository.begin_transaction().await?;

        let (first_block, second_block) = self
            .time_block_repository
            .split_at(&mut tx, block_id, split_at)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok((first_block, second_block))
    }

    /// 获取时间块使用统计
    pub async fn get_usage_statistics(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<crate::repositories::TimeBlockUsageStats> {
        self.time_block_repository
            .get_usage_statistics(start_date, end_date)
            .await
            .map_err(AppError::from)
    }
}

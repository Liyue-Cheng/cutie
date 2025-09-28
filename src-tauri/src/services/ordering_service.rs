use std::sync::Arc;
use uuid::Uuid;

use super::UpdateOrderCommand;
use crate::common::error::{AppError, AppResult};
use crate::core::models::{ContextType, Ordering};
use crate::ports::{Clock, IdGenerator};
use crate::repositories::{OrderingRepository, TaskRepository};

/// 排序服务
///
/// **预期行为简介:** 封装所有与Ordering相关的业务逻辑，管理任务在各种上下文中的排序
pub struct OrderingService {
    /// 时钟服务
    clock: Arc<dyn Clock>,

    /// ID生成器
    id_generator: Arc<dyn IdGenerator>,

    /// 任务仓库
    task_repository: Arc<dyn TaskRepository>,

    /// 排序仓库
    ordering_repository: Arc<dyn OrderingRepository>,
}

impl OrderingService {
    /// 创建新的排序服务
    pub fn new(
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        task_repository: Arc<dyn TaskRepository>,
        ordering_repository: Arc<dyn OrderingRepository>,
    ) -> Self {
        Self {
            clock,
            id_generator,
            task_repository,
            ordering_repository,
        }
    }

    /// 更新任务排序
    ///
    /// **函数签名:** `pub async fn update_order(&self, command: UpdateOrderCommand) -> Result<(), AppError>`
    /// **预期行为简介:** 更新一个任务在一个特定上下文中的排序位置。
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证输入:** 检查`command`中的`context`和`task_id`是否有效。`new_sort_order`必须是合法的排序字符串。
    /// 3. **核心操作:** 调用`OrderingRepository::upsert`，在`ordering`表中创建或更新对应的排序记录。
    /// 4. **提交事务。**
    /// 5. **返回:** `Ok(())`。
    /// **预期副作用:** 创建或更新`ordering`表中的一条记录。
    pub async fn update_order(&self, command: UpdateOrderCommand) -> AppResult<()> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证输入
        // 验证任务存在
        let _task = self
            .task_repository
            .find_by_id(command.task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", command.task_id.to_string()))?;

        // 验证上下文ID格式
        Ordering::validate_context_id(&command.context_type, &command.context_id)
            .map_err(|e| AppError::validation_error("context_id", e, "CONTEXT_ID_INVALID"))?;

        // 验证排序字符串
        if !crate::common::utils::sort_order_utils::is_valid_sort_order(&command.new_sort_order) {
            return Err(AppError::validation_error(
                "new_sort_order",
                "Invalid sort order string",
                "SORT_ORDER_INVALID",
            ));
        }

        // 3. 核心操作
        let ordering = Ordering::new(
            self.id_generator.new_uuid(),
            command.context_type,
            command.context_id,
            command.task_id,
            command.new_sort_order,
            self.clock.now_utc(),
        )?;

        self.ordering_repository.upsert(&mut tx, &ordering).await?;

        // 4. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 5. 返回
        Ok(())
    }

    /// 获取上下文中的任务排序
    pub async fn get_context_ordering(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<Vec<Ordering>> {
        self.ordering_repository
            .find_for_context(context_type, context_id)
            .await
            .map_err(AppError::from)
    }

    /// 批量更新排序
    pub async fn batch_update_order(&self, orderings: Vec<Ordering>) -> AppResult<()> {
        let mut tx = self.task_repository.begin_transaction().await?;

        // 验证所有排序记录
        for ordering in &orderings {
            // 验证任务存在
            let _task = self
                .task_repository
                .find_by_id(ordering.task_id)
                .await?
                .ok_or_else(|| AppError::not_found("Task", ordering.task_id.to_string()))?;

            // 验证上下文ID格式
            Ordering::validate_context_id(&ordering.context_type, &ordering.context_id)
                .map_err(|e| AppError::validation_error("context_id", e, "CONTEXT_ID_INVALID"))?;
        }

        // 批量更新
        self.ordering_repository
            .batch_upsert(&mut tx, &orderings)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }

    /// 获取任务的所有排序记录
    pub async fn get_task_orderings(&self, task_id: Uuid) -> AppResult<Vec<Ordering>> {
        self.ordering_repository
            .find_for_task(task_id)
            .await
            .map_err(AppError::from)
    }

    /// 清理上下文中的所有排序
    pub async fn clear_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<()> {
        let mut tx = self.task_repository.begin_transaction().await?;

        self.ordering_repository
            .clear_context(&mut tx, context_type, context_id)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }

    /// 计算两个排序位置之间的新位置
    pub async fn get_sort_order_between(
        &self,
        context_type: &ContextType,
        context_id: &str,
        prev_sort_order: Option<&str>,
        next_sort_order: Option<&str>,
    ) -> AppResult<String> {
        self.ordering_repository
            .get_sort_order_between(context_type, context_id, prev_sort_order, next_sort_order)
            .await
            .map_err(AppError::from)
    }
}

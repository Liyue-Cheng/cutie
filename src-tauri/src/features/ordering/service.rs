/// 排序业务逻辑层
///
/// 实现排序的业务逻辑和规则

use chrono::Utc;
use uuid::Uuid;

use crate::shared::{
    core::{
        get_mid_lexo_rank, get_rank_after, get_rank_before, generate_initial_sort_order,
        AppError, AppResult, ContextType, Ordering,
    },
    database::OrderingRepository,
};

use super::payloads::{
    CalculateSortOrderResponse, OrderingStatsResponse, UpdateOrderPayload,
};

/// 排序服务
pub struct OrderingService<R: OrderingRepository> {
    repository: R,
}

impl<R: OrderingRepository> OrderingService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 更新排序
    pub async fn update_order(&self, payload: UpdateOrderPayload) -> AppResult<()> {
        // 验证上下文和排序值
        Ordering::validate_context_id(&payload.context_type, &payload.context_id)?;

        if !crate::shared::core::is_valid_sort_order(&payload.new_sort_order) {
            return Err(AppError::validation_error(
                "new_sort_order",
                "无效的排序值格式",
                "INVALID_SORT_ORDER",
            ));
        }

        // 更新排序记录
        self.repository
            .update_sort_order(
                &payload.context_type,
                &payload.context_id,
                payload.task_id,
                &payload.new_sort_order,
            )
            .await
    }

    /// 获取上下文中的排序记录
    pub async fn get_context_ordering(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<Vec<Ordering>> {
        // 验证上下文ID格式
        Ordering::validate_context_id(context_type, context_id)?;

        self.repository.find_by_context(context_type, context_id).await
    }

    /// 清理上下文中的排序记录
    pub async fn clear_context_ordering(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<()> {
        // 验证上下文ID格式
        Ordering::validate_context_id(context_type, context_id)?;

        self.repository.clear_context(context_type, context_id).await
    }

    /// 批量更新排序记录
    pub async fn batch_update_ordering(&self, orderings: Vec<Ordering>) -> AppResult<()> {
        // 验证所有排序记录
        for ordering in &orderings {
            Ordering::validate_context_id(&ordering.context_type, &ordering.context_id)?;

            if !crate::shared::core::is_valid_sort_order(&ordering.sort_order) {
                return Err(AppError::validation_error(
                    "sort_order",
                    "无效的排序值格式",
                    "INVALID_SORT_ORDER",
                ));
            }
        }

        self.repository.batch_update(&orderings).await
    }

    /// 计算排序位置
    pub async fn calculate_sort_order(
        &self,
        context_type: &ContextType,
        context_id: &str,
        prev_sort_order: Option<&str>,
        next_sort_order: Option<&str>,
    ) -> AppResult<CalculateSortOrderResponse> {
        // 验证上下文ID格式
        Ordering::validate_context_id(context_type, context_id)?;

        // 验证排序值格式
        if let Some(prev) = prev_sort_order {
            if !crate::shared::core::is_valid_sort_order(prev) {
                return Err(AppError::validation_error(
                    "prev_sort_order",
                    "无效的前置排序值格式",
                    "INVALID_PREV_SORT_ORDER",
                ));
            }
        }

        if let Some(next) = next_sort_order {
            if !crate::shared::core::is_valid_sort_order(next) {
                return Err(AppError::validation_error(
                    "next_sort_order",
                    "无效的后置排序值格式",
                    "INVALID_NEXT_SORT_ORDER",
                ));
            }
        }

        // 计算新的排序值
        let new_sort_order = match (prev_sort_order, next_sort_order) {
            (Some(prev), Some(next)) => {
                if prev >= next {
                    return Err(AppError::validation_error(
                        "sort_order_range",
                        "前置排序值必须小于后置排序值",
                        "INVALID_SORT_ORDER_RANGE",
                    ));
                }
                get_mid_lexo_rank(prev, next)
            }
            (Some(prev), None) => get_rank_after(prev),
            (None, Some(next)) => get_rank_before(next),
            (None, None) => generate_initial_sort_order(),
        };

        Ok(CalculateSortOrderResponse {
            sort_order: new_sort_order,
            context_type: context_type.clone(),
            context_id: context_id.to_string(),
        })
    }

    /// 获取任务的排序记录
    pub async fn get_task_orderings(&self, task_id: Uuid) -> AppResult<Vec<Ordering>> {
        self.repository.find_by_task_id(task_id).await
    }

    /// 获取排序统计
    pub async fn get_ordering_stats(&self) -> AppResult<OrderingStatsResponse> {
        let all_orderings = self.repository.find_all().await?;

        let total_count = all_orderings.len() as i64;

        // 按上下文类型分组统计
        let mut by_context_type = std::collections::HashMap::new();
        for ordering in &all_orderings {
            let context_type_str = match ordering.context_type {
                ContextType::DailyKanban => "DAILY_KANBAN",
                ContextType::ProjectList => "PROJECT_LIST",
                ContextType::AreaFilter => "AREA_FILTER",
                ContextType::Misc => "MISC",
            };

            *by_context_type.entry(context_type_str.to_string()).or_insert(0) += 1;
        }

        // 计算活跃上下文数量
        let mut active_contexts = std::collections::HashSet::new();
        for ordering in &all_orderings {
            let context_key = format!("{}::{}", 
                match ordering.context_type {
                    ContextType::DailyKanban => "DAILY_KANBAN",
                    ContextType::ProjectList => "PROJECT_LIST", 
                    ContextType::AreaFilter => "AREA_FILTER",
                    ContextType::Misc => "MISC",
                },
                ordering.context_id
            );
            active_contexts.insert(context_key);
        }

        let active_contexts_count = active_contexts.len() as i64;

        // 计算平均每个上下文的排序记录数
        let avg_records_per_context = if active_contexts_count > 0 {
            total_count as f64 / active_contexts_count as f64
        } else {
            0.0
        };

        Ok(OrderingStatsResponse {
            total_count,
            by_context_type,
            active_contexts_count,
            avg_records_per_context,
        })
    }

    /// 为任务创建初始排序记录
    pub async fn create_initial_ordering(
        &self,
        context_type: &ContextType,
        context_id: &str,
        task_id: Uuid,
    ) -> AppResult<Ordering> {
        // 验证上下文ID格式
        Ordering::validate_context_id(context_type, context_id)?;

        // 获取上下文中现有的排序记录
        let existing_orderings = self.repository.find_by_context(context_type, context_id).await?;

        // 计算新的排序值（放在末尾）
        let new_sort_order = if existing_orderings.is_empty() {
            generate_initial_sort_order()
        } else {
            let last_sort_order = &existing_orderings.last().unwrap().sort_order;
            get_rank_after(last_sort_order)
        };

        // 创建排序记录
        let ordering = Ordering::new(
            Uuid::new_v4(),
            context_type.clone(),
            context_id.to_string(),
            task_id,
            new_sort_order,
            Utc::now(),
        )?;

        self.repository.create(&ordering).await
    }

    /// 重新排序上下文中的所有记录
    pub async fn reorder_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
        task_ids: Vec<Uuid>,
    ) -> AppResult<()> {
        // 验证上下文ID格式
        Ordering::validate_context_id(context_type, context_id)?;

        // 清理现有排序记录
        self.repository.clear_context(context_type, context_id).await?;

        // 为每个任务创建新的排序记录
        let mut new_orderings: Vec<Ordering> = Vec::new();
        for (index, task_id) in task_ids.iter().enumerate() {
            let sort_order = if index == 0 {
                generate_initial_sort_order()
            } else {
                let prev_sort_order = &new_orderings[index - 1].sort_order;
                get_rank_after(prev_sort_order)
            };

            let ordering = Ordering::new(
                Uuid::new_v4(),
                context_type.clone(),
                context_id.to_string(),
                *task_id,
                sort_order,
                Utc::now(),
            )?;

            new_orderings.push(ordering);
        }

        // 批量创建新的排序记录
        self.repository.batch_update(&new_orderings).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::ordering::repository::SqlxOrderingRepository,
        shared::database::connection::create_test_database,
    };

    #[tokio::test]
    async fn test_update_order() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxOrderingRepository::new(pool);
        let service = OrderingService::new(repository);

        let payload = UpdateOrderPayload {
            context_type: ContextType::Misc,
            context_id: "floating".to_string(),
            task_id: Uuid::new_v4(),
            new_sort_order: "n".to_string(),
        };

        // 应该成功创建新的排序记录
        service.update_order(payload).await.unwrap();
    }

    #[tokio::test]
    async fn test_calculate_sort_order() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxOrderingRepository::new(pool);
        let service = OrderingService::new(repository);

        // 测试在两个值之间计算
        let response = service
            .calculate_sort_order(
                &ContextType::Misc,
                "floating",
                Some("a"),
                Some("z"),
            )
            .await
            .unwrap();

        assert!(response.sort_order > "a");
        assert!(response.sort_order < "z");
        assert_eq!(response.context_type, ContextType::Misc);
        assert_eq!(response.context_id, "floating");
    }

    #[tokio::test]
    async fn test_create_initial_ordering() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxOrderingRepository::new(pool);
        let service = OrderingService::new(repository);

        let task_id = Uuid::new_v4();
        let ordering = service
            .create_initial_ordering(&ContextType::Misc, "floating", task_id)
            .await
            .unwrap();

        assert_eq!(ordering.task_id, task_id);
        assert_eq!(ordering.context_type, ContextType::Misc);
        assert_eq!(ordering.context_id, "floating");
    }

    #[tokio::test]
    async fn test_reorder_context() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxOrderingRepository::new(pool);
        let service = OrderingService::new(repository);

        let task_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

        // 重新排序上下文
        service
            .reorder_context(&ContextType::Misc, "floating", task_ids.clone())
            .await
            .unwrap();

        // 验证排序记录已创建
        let orderings = service
            .get_context_ordering(&ContextType::Misc, "floating")
            .await
            .unwrap();

        assert_eq!(orderings.len(), 3);

        // 验证任务ID顺序
        for (index, ordering) in orderings.iter().enumerate() {
            assert_eq!(ordering.task_id, task_ids[index]);
        }

        // 验证排序值是递增的
        for i in 1..orderings.len() {
            assert!(orderings[i - 1].sort_order < orderings[i].sort_order);
        }
    }
}

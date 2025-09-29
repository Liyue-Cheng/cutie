/// 排序功能模块的HTTP载荷定义
///
/// 定义排序相关API端点的请求体和响应体结构

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::{
    core::{ContextType, Ordering, ValidationError},
    http::extractors::Validate,
};

/// 更新排序请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderPayload {
    /// 上下文类型
    pub context_type: ContextType,

    /// 上下文ID
    pub context_id: String,

    /// 任务ID
    pub task_id: Uuid,

    /// 新的排序值
    pub new_sort_order: String,
}

impl Validate for UpdateOrderPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证上下文ID格式
        if let Err(err) = Ordering::validate_context_id(&self.context_type, &self.context_id) {
            errors.push(ValidationError::new(
                "context_id",
                &err,
                "INVALID_CONTEXT_ID",
            ));
        }

        // 验证排序值
        if !crate::shared::core::is_valid_sort_order(&self.new_sort_order) {
            errors.push(ValidationError::new(
                "new_sort_order",
                "无效的排序值格式",
                "INVALID_SORT_ORDER",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 获取上下文排序查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOrderingQuery {
    /// 上下文类型
    pub context_type: ContextType,

    /// 上下文ID
    pub context_id: String,
}

impl Validate for ContextOrderingQuery {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证上下文ID格式
        if let Err(err) = Ordering::validate_context_id(&self.context_type, &self.context_id) {
            errors.push(ValidationError::new(
                "context_id",
                &err,
                "INVALID_CONTEXT_ID",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 计算排序位置查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateSortOrderQuery {
    /// 上下文类型
    pub context_type: ContextType,

    /// 上下文ID
    pub context_id: String,

    /// 前一个排序位置（可选）
    pub prev_sort_order: Option<String>,

    /// 后一个排序位置（可选）
    pub next_sort_order: Option<String>,
}

impl Validate for CalculateSortOrderQuery {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证上下文ID格式
        if let Err(err) = Ordering::validate_context_id(&self.context_type, &self.context_id) {
            errors.push(ValidationError::new(
                "context_id",
                &err,
                "INVALID_CONTEXT_ID",
            ));
        }

        // 验证排序值格式
        if let Some(prev) = &self.prev_sort_order {
            if !crate::shared::core::is_valid_sort_order(prev) {
                errors.push(ValidationError::new(
                    "prev_sort_order",
                    "无效的前置排序值格式",
                    "INVALID_PREV_SORT_ORDER",
                ));
            }
        }

        if let Some(next) = &self.next_sort_order {
            if !crate::shared::core::is_valid_sort_order(next) {
                errors.push(ValidationError::new(
                    "next_sort_order",
                    "无效的后置排序值格式",
                    "INVALID_NEXT_SORT_ORDER",
                ));
            }
        }

        // 验证排序值的逻辑关系
        if let (Some(prev), Some(next)) = (&self.prev_sort_order, &self.next_sort_order) {
            if prev >= next {
                errors.push(ValidationError::new(
                    "sort_order_range",
                    "前置排序值必须小于后置排序值",
                    "INVALID_SORT_ORDER_RANGE",
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 计算排序位置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateSortOrderResponse {
    /// 计算出的排序值
    pub sort_order: String,

    /// 上下文信息
    pub context_type: ContextType,

    /// 上下文ID
    pub context_id: String,
}

/// 批量排序操作载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateOrderingPayload {
    /// 排序记录列表
    pub orderings: Vec<Ordering>,
}

impl Validate for BatchUpdateOrderingPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.orderings.is_empty() {
            errors.push(ValidationError::new(
                "orderings",
                "排序记录列表不能为空",
                "ORDERINGS_EMPTY",
            ));
        }

        if self.orderings.len() > 100 {
            errors.push(ValidationError::new(
                "orderings",
                "批量操作不能超过100个排序记录",
                "TOO_MANY_ORDERINGS",
            ));
        }

        // 验证每个排序记录
        for (index, ordering) in self.orderings.iter().enumerate() {
            if let Err(err) = Ordering::validate_context_id(&ordering.context_type, &ordering.context_id) {
                errors.push(ValidationError::new(
                    &format!("orderings[{}].context_id", index),
                    &err,
                    "INVALID_CONTEXT_ID",
                ));
            }

            if !crate::shared::core::is_valid_sort_order(&ordering.sort_order) {
                errors.push(ValidationError::new(
                    &format!("orderings[{}].sort_order", index),
                    "无效的排序值格式",
                    "INVALID_SORT_ORDER",
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 清理上下文排序查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearContextQuery {
    /// 上下文类型
    pub context_type: ContextType,

    /// 上下文ID
    pub context_id: String,
}

impl Validate for ClearContextQuery {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证上下文ID格式
        if let Err(err) = Ordering::validate_context_id(&self.context_type, &self.context_id) {
            errors.push(ValidationError::new(
                "context_id",
                &err,
                "INVALID_CONTEXT_ID",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 排序统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderingStatsResponse {
    /// 总排序记录数
    pub total_count: i64,

    /// 按上下文类型分组的统计
    pub by_context_type: std::collections::HashMap<String, i64>,

    /// 活跃上下文数量
    pub active_contexts_count: i64,

    /// 平均每个上下文的排序记录数
    pub avg_records_per_context: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_update_order_payload_validation() {
        let valid_payload = UpdateOrderPayload {
            context_type: ContextType::Misc,
            context_id: "floating".to_string(),
            task_id: Uuid::new_v4(),
            new_sort_order: "n".to_string(),
        };

        assert!(valid_payload.validate().is_ok());

        // 测试无效的上下文ID
        let invalid_payload = UpdateOrderPayload {
            context_type: ContextType::DailyKanban,
            context_id: "invalid-timestamp".to_string(),
            task_id: Uuid::new_v4(),
            new_sort_order: "n".to_string(),
        };

        assert!(invalid_payload.validate().is_err());
    }

    #[test]
    fn test_calculate_sort_order_query_validation() {
        let valid_query = CalculateSortOrderQuery {
            context_type: ContextType::Misc,
            context_id: "floating".to_string(),
            prev_sort_order: Some("a".to_string()),
            next_sort_order: Some("z".to_string()),
        };

        assert!(valid_query.validate().is_ok());

        // 测试无效的排序范围
        let invalid_query = CalculateSortOrderQuery {
            context_type: ContextType::Misc,
            context_id: "floating".to_string(),
            prev_sort_order: Some("z".to_string()),
            next_sort_order: Some("a".to_string()),
        };

        assert!(invalid_query.validate().is_err());
    }

    #[test]
    fn test_batch_update_ordering_validation() {
        let ordering = Ordering::new(
            Uuid::new_v4(),
            ContextType::Misc,
            "floating".to_string(),
            Uuid::new_v4(),
            "n".to_string(),
            Utc::now(),
        ).unwrap();

        let valid_payload = BatchUpdateOrderingPayload {
            orderings: vec![ordering],
        };

        assert!(valid_payload.validate().is_ok());

        // 测试空列表
        let invalid_payload = BatchUpdateOrderingPayload {
            orderings: vec![],
        };

        assert!(invalid_payload.validate().is_err());
    }
}

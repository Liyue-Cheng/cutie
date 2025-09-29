use async_trait::async_trait;
use uuid::Uuid;

use super::Transaction;
use crate::common::error::DbError;
use crate::core::models::{ContextType, Ordering};

/// 排序仓库接口定义
///
/// **预期行为简介:** 提供对Ordering记录的持久化操作，管理所有自定义排序
#[async_trait]
pub trait OrderingRepository: Send + Sync {
    /// 创建或更新排序记录
    ///
    /// **预期行为简介:** 根据context_type, context_id, task_id的组合键，创建或更新一条排序记录
    /// **输入输出规范:**
    /// - **前置条件:** 输入的Ordering对象必须完整
    /// - **后置条件:** 数据库中存在或更新了对应的排序记录，其sort_order值与输入对象一致
    /// **预期副作用:** 创建或更新ordering表中的一条记录
    async fn upsert(
        &self,
        tx: &mut Transaction<'_>,
        ordering: &Ordering,
    ) -> Result<Ordering, DbError>;

    /// 删除任务在特定上下文中的排序
    ///
    /// **预期行为简介:** 删除指定任务在特定上下文中的排序记录
    /// **输入输出规范:**
    /// - **前置条件:** task_id, context_type, context_id必须有效
    /// - **后置条件:** 对应的排序记录被删除
    /// **边界情况:** 如果记录不存在，幂等地返回成功
    /// **预期副作用:** 从ordering表中删除零条或一条记录
    async fn delete_for_task_in_context(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<(), DbError>;

    /// 查找上下文中的所有排序
    ///
    /// **预期行为简介:** 查找指定上下文中的所有排序记录
    /// **输入输出规范:**
    /// - **前置条件:** context_type和context_id必须有效
    /// - **后置条件:** 返回该上下文中的所有Ordering记录，按sort_order排序
    /// **边界情况:** 如果上下文中没有排序，返回空列表
    /// **预期副作用:** 无
    async fn find_for_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<Vec<Ordering>, DbError>;

    /// 批量更新排序
    ///
    /// **预期行为简介:** 批量更新多个任务的排序位置
    /// **输入输出规范:**
    /// - **前置条件:** orderings列表必须都属于同一个上下文
    /// - **后置条件:** 所有排序记录都被更新
    /// **预期副作用:** 在一个事务中更新多条ordering记录
    async fn batch_upsert(
        &self,
        tx: &mut Transaction<'_>,
        orderings: &[Ordering],
    ) -> Result<Vec<Ordering>, DbError>;

    /// 删除任务的所有排序
    ///
    /// **预期行为简介:** 删除指定任务在所有上下文中的排序记录
    /// **输入输出规范:**
    /// - **前置条件:** task_id必须有效
    /// - **后置条件:** 该任务的所有排序记录都被删除
    /// **边界情况:** 如果任务没有排序记录，幂等地返回成功
    /// **预期副作用:** 可能删除ordering表中的零条或多条记录
    async fn delete_all_for_task(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
    ) -> Result<(), DbError>;

    /// 清理上下文中的排序
    ///
    /// **预期行为简介:** 删除指定上下文中的所有排序记录
    async fn clear_context(
        &self,
        tx: &mut Transaction<'_>,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<(), DbError>;

    /// 查找任务的所有排序
    ///
    /// **预期行为简介:** 查找指定任务在所有上下文中的排序记录
    async fn find_for_task(&self, task_id: Uuid) -> Result<Vec<Ordering>, DbError>;

    /// 获取上下文中的下一个排序位置
    ///
    /// **预期行为简介:** 计算在指定上下文末尾插入新任务的排序字符串
    async fn get_next_sort_order(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<String, DbError>;

    /// 获取两个位置之间的排序位置
    ///
    /// **预期行为简介:** 计算在两个现有排序位置之间插入新任务的排序字符串
    async fn get_sort_order_between(
        &self,
        context_type: &ContextType,
        context_id: &str,
        prev_sort_order: Option<&str>,
        next_sort_order: Option<&str>,
    ) -> Result<String, DbError>;

    /// 统计上下文中的任务数量
    ///
    /// **预期行为简介:** 统计指定上下文中的任务数量
    async fn count_tasks_in_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<i64, DbError>;
}

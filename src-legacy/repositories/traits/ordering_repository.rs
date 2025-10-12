/// OrderingRepository trait定义
///
/// 提供对Ordering实体的所有持久化操作接口
use async_trait::async_trait;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::infra::core::AppResult;
use crate::entities::{ContextType, Ordering};

/// 排序仓库接口
///
/// 注意：事务管理在服务层进行，本层的方法接受事务句柄作为参数
#[async_trait]
pub trait OrderingRepository: Send + Sync {
    // --- 写操作 ---
    /// 创建或更新排序记录
    ///
    /// # 参数
    /// - `tx`: 数据库事务
    /// - `ordering`: 排序对象
    ///
    /// # 行为
    /// 根据context_type, context_id, task_id的组合键，创建或更新一条排序记录
    ///
    /// # 前置条件
    /// 输入的Ordering对象必须完整
    ///
    /// # 后置条件
    /// 数据库中存在或更新了对应的排序记录，其sort_order值与输入对象一致
    async fn upsert(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        ordering: &Ordering,
    ) -> AppResult<Ordering>;

    /// 删除指定上下文中某个任务的排序记录
    async fn delete_for_task_in_context(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<()>;

    // --- 读操作 ---
    /// 查找指定上下文的所有排序记录
    async fn find_for_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<Vec<Ordering>>;

    /// 为新任务在指定上下文中创建排序记录（放在末尾）
    ///
    /// # 参数
    /// - `tx`: 数据库事务
    /// - `context_type`: 上下文类型
    /// - `context_id`: 上下文ID
    /// - `task_id`: 任务ID
    /// - `created_at`: 创建时间
    ///
    /// # 行为
    /// 自动计算新的排序位置（放在当前上下文的末尾），然后创建排序记录
    async fn create_for_new_task(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        context_type: &ContextType,
        context_id: &str,
        task_id: Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<Ordering>;
}

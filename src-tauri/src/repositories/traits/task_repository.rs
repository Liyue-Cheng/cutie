/// TaskRepository trait定义
///
/// 提供对Task实体的所有持久化操作接口
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::entities::Task;
use crate::shared::core::AppResult;

/// 任务仓库接口
///
/// 注意：事务管理在服务层进行，本层的方法接受事务句柄作为参数
#[async_trait]
pub trait TaskRepository: Send + Sync {
    // --- 写操作 ---
    /// 创建新任务
    ///
    /// # 参数
    /// - `tx`: 数据库事务
    /// - `task`: 要创建的任务对象
    ///
    /// # 返回
    /// 创建成功后返回包含最新信息的Task对象
    async fn create(&self, tx: &mut Transaction<'_, Sqlite>, task: &Task) -> AppResult<Task>;

    /// 更新任务
    ///
    /// # 参数
    /// - `tx`: 数据库事务
    /// - `task`: 要更新的任务对象（必须包含有效的id）
    ///
    /// # 前置条件
    /// - task.id必须存在于数据库中
    /// - 传入的task对象必须是完整的
    ///
    /// # 后置条件
    /// - 成功后返回更新后的、包含最新updated_at的Task对象
    /// - 数据库中的记录与返回对象的状态完全一致
    async fn update(&self, tx: &mut Transaction<'_, Sqlite>, task: &Task) -> AppResult<Task>;

    /// 设置任务为已完成状态
    ///
    /// # 参数
    /// - `tx`: 数据库事务
    /// - `task_id`: 任务ID
    /// - `completion_time`: 完成时间
    ///
    /// # 返回
    /// 更新后的Task对象
    async fn set_completed(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completion_time: DateTime<Utc>,
    ) -> AppResult<Task>;

    /// 重新打开已完成的任务
    ///
    /// # 参数
    /// - `tx`: 数据库事务
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    /// 更新后的Task对象
    async fn reopen(&self, tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<Task>;

    // --- 读操作 ---
    /// 根据ID查找任务（独立连接）
    ///
    /// # 参数
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    /// - Some(Task): 找到任务
    /// - None: 任务不存在或已删除
    async fn find_by_id(&self, task_id: Uuid) -> AppResult<Option<Task>>;

    /// 根据ID查找任务（在事务中）
    ///
    /// # 参数
    /// - `tx`: 数据库事务
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    /// - Some(Task): 找到任务
    /// - None: 任务不存在或已删除
    async fn find_by_id_in_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<Task>>;

    /// 根据多个ID批量查找任务
    ///
    /// # 参数
    /// - `task_ids`: 任务ID列表
    ///
    /// # 返回
    /// 找到的任务列表（不包含已删除的任务）
    async fn find_many_by_ids(&self, task_ids: &[Uuid]) -> AppResult<Vec<Task>>;

    /// 查找所有未安排日程的任务
    ///
    /// # 返回
    /// 未安排日程的任务列表，用于Staging区显示
    async fn find_unscheduled(&self) -> AppResult<Vec<Task>>;

    /// 检查任务是否存在（轻量级检查）
    ///
    /// # 参数
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    /// true表示任务存在且未删除，false表示不存在或已删除
    async fn exists(&self, task_id: Uuid) -> AppResult<bool>;
}

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::error::DbError;
use crate::core::models::Task;

/// 数据库事务类型别名
pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;

/// 任务仓库接口定义
///
/// **预期行为简介:** 提供对Task实体的所有持久化操作
///
/// ## 不变量
/// - 本层代码是唯一允许包含SQL语句的地方
/// - 本层方法应尽可能地原子化，即一个方法对应一个或一组紧密相关的数据库查询
/// - 所有需要跨多个仓库操作的业务逻辑，其事务管理必须在服务层进行
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// 开始数据库事务
    ///
    /// **预期行为简介:** 创建一个新的数据库事务用于跨操作的一致性保证
    /// **输入输出规范:**
    /// - **前置条件:** 数据库连接池可用
    /// - **后置条件:** 返回一个可用的事务句柄
    /// **边界情况:** 如果数据库不可用，返回DbError::ConnectionError
    /// **预期副作用:** 在数据库中开始一个新事务
    async fn begin_transaction(&self) -> Result<Transaction<'_>, DbError>;

    /// 创建新任务
    ///
    /// **预期行为简介:** 在数据库中插入一个新的任务记录
    /// **输入输出规范:**
    /// - **前置条件:** task对象必须完整且有效，tx必须是活跃的事务
    /// - **后置条件:** 成功时返回插入的Task对象，数据库中存在新记录
    /// **边界情况:** 如果task.id已存在，返回DbError::ConstraintViolation
    /// **预期副作用:** 向tasks表插入一条新记录
    async fn create(&self, tx: &mut Transaction<'_>, task: &Task) -> Result<Task, DbError>;

    /// 更新任务
    ///
    /// **预期行为简介:** 根据传入的Task对象的id，更新数据库中对应记录的所有可变字段
    /// **输入输出规范:**
    /// - **前置条件:** task.id必须存在于数据库中。传入的task对象必须是完整的
    /// - **后置条件:** 成功后返回更新后的、包含最新updated_at的Task对象。数据库中的记录与返回对象的状态完全一致
    /// **边界情况:** 如果task.id不存在，应返回DbError::NotFound
    /// **预期副作用:** 修改tasks表中的一条记录。updated_at字段被更新
    async fn update(&self, tx: &mut Transaction<'_>, task: &Task) -> Result<Task, DbError>;

    /// 设置任务为已完成
    ///
    /// **预期行为简介:** 设置指定任务的completed_at字段为给定时间
    /// **输入输出规范:**
    /// - **前置条件:** task_id必须存在，completion_time必须有效
    /// - **后置条件:** 返回更新后的Task对象，其completed_at字段被设置
    /// **边界情况:** 如果任务已完成，幂等地返回当前状态
    /// **预期副作用:** 更新tasks表中的completed_at和updated_at字段
    async fn set_completed(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
        completion_time: DateTime<Utc>,
    ) -> Result<Task, DbError>;

    /// 重新打开任务
    ///
    /// **预期行为简介:** 将已完成的任务重新设置为未完成状态
    /// **输入输出规范:**
    /// - **前置条件:** task_id必须存在
    /// - **后置条件:** 返回更新后的Task对象，其completed_at字段为NULL
    /// **边界情况:** 如果任务本就未完成，幂等地返回当前状态
    /// **预期副作用:** 更新tasks表中的completed_at和updated_at字段
    async fn reopen(&self, tx: &mut Transaction<'_>, task_id: Uuid) -> Result<Task, DbError>;

    /// 根据ID查找任务
    ///
    /// **预期行为简介:** 根据UUID查找单个任务
    /// **输入输出规范:**
    /// - **前置条件:** task_id必须是有效的UUID
    /// - **后置条件:** 如果找到返回Some(Task)，否则返回None
    /// **边界情况:** 如果任务被逻辑删除(is_deleted=true)，返回None
    /// **预期副作用:** 无
    async fn find_by_id(&self, task_id: Uuid) -> Result<Option<Task>, DbError>;

    /// 批量查找任务
    ///
    /// **预期行为简介:** 根据UUID列表批量查找任务
    /// **输入输出规范:**
    /// - **前置条件:** task_ids必须是有效的UUID列表
    /// - **后置条件:** 返回找到的Task对象列表，顺序可能与输入不同
    /// **边界情况:** 如果某些ID不存在，只返回存在的任务
    /// **预期副作用:** 无
    async fn find_many_by_ids(&self, task_ids: &[Uuid]) -> Result<Vec<Task>, DbError>;

    /// 查找未安排的任务（Staging区）
    ///
    /// **预期行为简介:** 查找所有不在任何日程中的任务，用于Staging区显示
    /// **输入输出规范:**
    /// - **前置条件:** 无
    /// - **后置条件:** 返回所有未被安排到任何日程的任务列表
    /// **边界情况:** 如果所有任务都已安排，返回空列表
    /// **预期副作用:** 无
    async fn find_unscheduled(&self) -> Result<Vec<Task>, DbError>;

    /// 根据项目ID查找任务
    ///
    /// **预期行为简介:** 查找属于指定项目的所有任务
    async fn find_by_project_id(&self, project_id: Uuid) -> Result<Vec<Task>, DbError>;

    /// 根据领域ID查找任务
    ///
    /// **预期行为简介:** 查找属于指定领域的所有任务
    async fn find_by_area_id(&self, area_id: Uuid) -> Result<Vec<Task>, DbError>;

    /// 查找已完成的任务
    ///
    /// **预期行为简介:** 查找所有已完成的任务，支持分页
    async fn find_completed(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Task>, DbError>;

    /// 软删除任务
    ///
    /// **预期行为简介:** 将任务标记为已删除而不是物理删除
    /// **输入输出规范:**
    /// - **前置条件:** task_id必须存在
    /// - **后置条件:** 任务的is_deleted字段被设置为true
    /// **边界情况:** 如果任务已被删除，幂等地返回成功
    /// **预期副作用:** 更新tasks表中的is_deleted和updated_at字段
    async fn soft_delete(&self, tx: &mut Transaction<'_>, task_id: Uuid) -> Result<(), DbError>;

    /// 恢复已删除的任务
    ///
    /// **预期行为简介:** 将软删除的任务恢复为可见状态
    async fn restore(&self, tx: &mut Transaction<'_>, task_id: Uuid) -> Result<Task, DbError>;

    /// 搜索任务
    ///
    /// **预期行为简介:** 根据关键词搜索任务标题和笔记
    async fn search(&self, query: &str, limit: Option<i64>) -> Result<Vec<Task>, DbError>;

    /// 统计任务数量
    ///
    /// **预期行为简介:** 统计各种状态的任务数量
    async fn count_by_status(&self) -> Result<TaskCountByStatus, DbError>;
}

/// 任务状态统计结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskCountByStatus {
    pub total: i64,
    pub completed: i64,
    pub pending: i64,
    pub scheduled: i64,
    pub unscheduled: i64,
}

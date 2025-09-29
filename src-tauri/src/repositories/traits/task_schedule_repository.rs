/// TaskScheduleRepository trait定义
///
/// 提供对TaskSchedule实体的所有持久化操作接口

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::shared::core::AppResult;
use crate::entities::{TaskSchedule, Outcome};

/// 任务日程仓库接口
/// 
/// 注意：事务管理在服务层进行，本层的方法接受事务句柄作为参数
#[async_trait]
pub trait TaskScheduleRepository: Send + Sync {
    // --- 写操作 ---
    /// 创建新的任务日程
    async fn create(&self, tx: &mut Transaction<'_, Sqlite>, schedule: &TaskSchedule) -> AppResult<TaskSchedule>;

    /// 更新任务日程的结果
    async fn update_outcome(&self, tx: &mut Transaction<'_, Sqlite>, schedule_id: Uuid, new_outcome: Outcome) -> AppResult<TaskSchedule>;

    /// 重新安排任务日程到新的日期
    async fn reschedule(&self, tx: &mut Transaction<'_, Sqlite>, schedule_id: Uuid, new_day: DateTime<Utc>) -> AppResult<TaskSchedule>;

    /// 删除指定的任务日程
    async fn delete(&self, tx: &mut Transaction<'_, Sqlite>, schedule_id: Uuid) -> AppResult<()>;

    /// 删除某个任务的所有日程安排
    async fn delete_all_for_task(&self, tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<()>;

    /// 删除某个任务在指定日期之后的所有日程安排
    /// 
    /// # 参数
    /// - `tx`: 数据库事务
    /// - `task_id`: 任务ID
    /// - `since`: 起始时间，删除此时间之后的日程
    /// 
    /// # 边界情况
    /// 如果没有未来的日程，操作应直接成功返回，不产生任何影响
    async fn delete_future_for_task(&self, tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, since: DateTime<Utc>) -> AppResult<()>;

    // --- 读操作 ---
    /// 查找指定日期的所有任务日程
    async fn find_by_day(&self, day: DateTime<Utc>) -> AppResult<Vec<TaskSchedule>>;

    /// 查找某个任务的所有日程安排
    async fn find_all_for_task(&self, task_id: Uuid) -> AppResult<Vec<TaskSchedule>>;
}

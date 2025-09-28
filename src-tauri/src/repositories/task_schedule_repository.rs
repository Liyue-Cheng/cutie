use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Transaction;
use crate::common::error::DbError;
use crate::core::models::{Outcome, TaskSchedule};

/// 任务日程仓库接口定义
///
/// **预期行为简介:** 提供对Task_Schedule关联记录的所有持久化操作
#[async_trait]
pub trait TaskScheduleRepository: Send + Sync {
    /// 创建任务日程
    ///
    /// **预期行为简介:** 在数据库中创建一个新的任务日程记录
    /// **输入输出规范:**
    /// - **前置条件:** schedule对象必须完整且有效，task_id必须存在
    /// - **后置条件:** 成功时返回创建的TaskSchedule对象
    /// **边界情况:** 如果同一任务在同一天已有日程，返回DbError::ConstraintViolation
    /// **预期副作用:** 向task_schedules表插入一条新记录
    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        schedule: &TaskSchedule,
    ) -> Result<TaskSchedule, DbError>;

    /// 更新日程结局
    ///
    /// **预期行为简介:** 更新指定日程的结局状态
    /// **输入输出规范:**
    /// - **前置条件:** schedule_id必须存在，new_outcome必须有效
    /// - **后置条件:** 返回更新后的TaskSchedule对象
    /// **边界情况:** 如果日程不存在，返回DbError::NotFound
    /// **预期副作用:** 修改task_schedules表中的outcome和updated_at字段
    async fn update_outcome(
        &self,
        tx: &mut Transaction<'_>,
        schedule_id: Uuid,
        new_outcome: Outcome,
    ) -> Result<TaskSchedule, DbError>;

    /// 重新安排日程
    ///
    /// **预期行为简介:** 将日程移动到新的日期
    /// **输入输出规范:**
    /// - **前置条件:** schedule_id必须存在，new_day必须是规范化的零点时间戳
    /// - **后置条件:** 返回更新后的TaskSchedule对象，outcome重置为PLANNED
    /// **边界情况:** 如果目标日期已有该任务的日程，返回DbError::ConstraintViolation
    /// **预期副作用:** 修改task_schedules表中的scheduled_day、outcome和updated_at字段
    async fn reschedule(
        &self,
        tx: &mut Transaction<'_>,
        schedule_id: Uuid,
        new_day: DateTime<Utc>,
    ) -> Result<TaskSchedule, DbError>;

    /// 删除日程
    ///
    /// **预期行为简介:** 删除指定的任务日程记录
    /// **输入输出规范:**
    /// - **前置条件:** schedule_id必须存在
    /// - **后置条件:** 成功时不返回内容，数据库中对应记录被删除
    /// **边界情况:** 如果日程不存在，幂等地返回成功
    /// **预期副作用:** 从task_schedules表中删除一条记录
    async fn delete(&self, tx: &mut Transaction<'_>, schedule_id: Uuid) -> Result<(), DbError>;

    /// 删除任务的所有日程
    ///
    /// **预期行为简介:** 删除指定任务的所有日程安排
    /// **输入输出规范:**
    /// - **前置条件:** task_id必须有效
    /// - **后置条件:** 该任务的所有日程记录都被删除
    /// **边界情况:** 如果任务没有日程，幂等地返回成功
    /// **预期副作用:** 可能删除task_schedules表中的零条或多条记录
    async fn delete_all_for_task(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
    ) -> Result<(), DbError>;

    /// 删除任务的未来日程
    ///
    /// **预期行为简介:** 删除某个任务在指定日期之后的所有日程安排
    /// **输入输出规范:**
    /// - **前置条件:** task_id和since时间戳必须有效
    /// - **后置条件:** task_schedule表中所有task_id匹配且scheduled_day大于since的记录都将被删除
    /// **边界情况:** 如果没有未来的日程，操作应直接成功返回，不产生任何影响
    /// **预期副作用:** 可能会删除task_schedule表中的零条或多条记录
    async fn delete_future_for_task(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
        since: DateTime<Utc>,
    ) -> Result<(), DbError>;

    /// 根据日期查找日程
    ///
    /// **预期行为简介:** 查找指定日期的所有任务日程
    /// **输入输出规范:**
    /// - **前置条件:** day必须是规范化的零点时间戳
    /// - **后置条件:** 返回该日期的所有TaskSchedule记录
    /// **边界情况:** 如果该日期没有日程，返回空列表
    /// **预期副作用:** 无
    async fn find_by_day(&self, day: DateTime<Utc>) -> Result<Vec<TaskSchedule>, DbError>;

    /// 查找任务的所有日程
    ///
    /// **预期行为简介:** 查找指定任务的所有日程安排
    /// **输入输出规范:**
    /// - **前置条件:** task_id必须有效
    /// - **后置条件:** 返回该任务的所有TaskSchedule记录，按日期排序
    /// **边界情况:** 如果任务没有日程，返回空列表
    /// **预期副作用:** 无
    async fn find_all_for_task(&self, task_id: Uuid) -> Result<Vec<TaskSchedule>, DbError>;

    /// 查找日期范围内的日程
    ///
    /// **预期行为简介:** 查找指定日期范围内的所有日程
    async fn find_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TaskSchedule>, DbError>;

    /// 查找特定结局的日程
    ///
    /// **预期行为简介:** 查找具有特定结局状态的日程
    async fn find_by_outcome(
        &self,
        outcome: Outcome,
        limit: Option<i64>,
    ) -> Result<Vec<TaskSchedule>, DbError>;

    /// 统计日程数量
    ///
    /// **预期行为简介:** 统计各种结局状态的日程数量
    async fn count_by_outcome(&self) -> Result<ScheduleCountByOutcome, DbError>;
}

/// 日程结局统计结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScheduleCountByOutcome {
    pub total: i64,
    pub planned: i64,
    pub presence_logged: i64,
    pub completed_on_day: i64,
    pub carried_over: i64,
}

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Transaction;
use crate::common::error::DbError;
use crate::core::models::TimeBlock;

/// 时间块仓库接口定义
///
/// **预期行为简介:** 提供对TimeBlock实体的所有持久化操作
#[async_trait]
pub trait TimeBlockRepository: Send + Sync {
    /// 创建新时间块
    ///
    /// **预期行为简介:** 在数据库中插入一个新的时间块记录
    /// **输入输出规范:**
    /// - **前置条件:** time_block对象必须完整且有效，start_time <= end_time
    /// - **后置条件:** 成功时返回插入的TimeBlock对象
    /// **边界情况:** 如果time_block.id已存在，返回DbError::ConstraintViolation
    /// **预期副作用:** 向time_blocks表插入一条新记录
    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        time_block: &TimeBlock,
    ) -> Result<TimeBlock, DbError>;

    /// 更新时间块
    ///
    /// **预期行为简介:** 更新指定时间块的信息
    /// **输入输出规范:**
    /// - **前置条件:** time_block.id必须存在，time_block对象必须完整
    /// - **后置条件:** 返回更新后的TimeBlock对象
    /// **边界情况:** 如果time_block.id不存在，返回DbError::NotFound
    /// **预期副作用:** 修改time_blocks表中的一条记录，updated_at字段被更新
    async fn update(
        &self,
        tx: &mut Transaction<'_>,
        time_block: &TimeBlock,
    ) -> Result<TimeBlock, DbError>;

    /// 根据ID查找时间块
    ///
    /// **预期行为简介:** 根据UUID查找单个时间块
    /// **输入输出规范:**
    /// - **前置条件:** time_block_id必须是有效的UUID
    /// - **后置条件:** 如果找到返回Some(TimeBlock)，否则返回None
    /// **边界情况:** 如果时间块被逻辑删除，返回None
    /// **预期副作用:** 无
    async fn find_by_id(&self, time_block_id: Uuid) -> Result<Option<TimeBlock>, DbError>;

    /// 查找时间范围内的时间块
    ///
    /// **预期行为简介:** 查找与指定时间范围有重叠的所有时间块
    /// **输入输出规范:**
    /// - **前置条件:** start_time <= end_time
    /// - **后置条件:** 返回与时间范围有重叠的TimeBlock记录列表
    /// **边界情况:** 如果没有重叠的时间块，返回空列表
    /// **预期副作用:** 无
    async fn find_overlapping(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TimeBlock>, DbError>;

    /// 查找指定日期的时间块
    ///
    /// **预期行为简介:** 查找指定日期内的所有时间块
    /// **输入输出规范:**
    /// - **前置条件:** date必须是规范化的零点时间戳
    /// - **后置条件:** 返回该日期内的TimeBlock记录列表，按开始时间排序
    /// **边界情况:** 如果该日期没有时间块，返回空列表
    /// **预期副作用:** 无
    async fn find_by_date(&self, date: DateTime<Utc>) -> Result<Vec<TimeBlock>, DbError>;

    /// 查找日期范围内的时间块
    ///
    /// **预期行为简介:** 查找指定日期范围内的所有时间块
    /// **输入输出规范:**
    /// - **前置条件:** start_date <= end_date，都必须是规范化的零点时间戳
    /// - **后置条件:** 返回日期范围内的TimeBlock记录列表
    /// **边界情况:** 如果范围内没有时间块，返回空列表
    /// **预期副作用:** 无
    async fn find_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimeBlock>, DbError>;

    /// 根据领域查找时间块
    ///
    /// **预期行为简介:** 查找属于指定领域的所有时间块
    /// **输入输出规范:**
    /// - **前置条件:** area_id必须是有效的UUID
    /// - **后置条件:** 返回area_id匹配的TimeBlock记录列表
    /// **边界情况:** 如果没有该领域的时间块，返回空列表
    /// **预期副作用:** 无
    async fn find_by_area_id(&self, area_id: Uuid) -> Result<Vec<TimeBlock>, DbError>;

    /// 查找与任务关联的时间块
    ///
    /// **预期行为简介:** 查找与指定任务关联的所有时间块
    /// **输入输出规范:**
    /// - **前置条件:** task_id必须是有效的UUID
    /// - **后置条件:** 返回通过task_time_block_links表关联的TimeBlock记录列表
    /// **边界情况:** 如果任务没有关联的时间块，返回空列表
    /// **预期副作用:** 无
    async fn find_by_task_id(&self, task_id: Uuid) -> Result<Vec<TimeBlock>, DbError>;

    /// 软删除时间块
    ///
    /// **预期行为简介:** 将时间块标记为已删除
    /// **输入输出规范:**
    /// - **前置条件:** time_block_id必须存在
    /// - **后置条件:** 时间块的is_deleted字段被设置为true
    /// **边界情况:** 如果时间块已被删除，幂等地返回成功
    /// **预期副作用:** 更新time_blocks表中的is_deleted和updated_at字段
    async fn soft_delete(
        &self,
        tx: &mut Transaction<'_>,
        time_block_id: Uuid,
    ) -> Result<(), DbError>;

    /// 恢复已删除的时间块
    ///
    /// **预期行为简介:** 将软删除的时间块恢复为可见状态
    async fn restore(
        &self,
        tx: &mut Transaction<'_>,
        time_block_id: Uuid,
    ) -> Result<TimeBlock, DbError>;

    /// 关联任务到时间块
    ///
    /// **预期行为简介:** 在task_time_block_links表中创建关联记录
    /// **输入输出规范:**
    /// - **前置条件:** task_id和time_block_id都必须存在
    /// - **后置条件:** 任务和时间块建立关联
    /// **边界情况:** 如果关联已存在，幂等地返回成功
    /// **预期副作用:** 向task_time_block_links表插入一条记录
    async fn link_task(
        &self,
        tx: &mut Transaction<'_>,
        time_block_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), DbError>;

    /// 取消任务与时间块的关联
    ///
    /// **预期行为简介:** 删除task_time_block_links表中的关联记录
    /// **输入输出规范:**
    /// - **前置条件:** task_id和time_block_id必须有效
    /// - **后置条件:** 任务和时间块的关联被移除
    /// **边界情况:** 如果关联不存在，幂等地返回成功
    /// **预期副作用:** 从task_time_block_links表中删除一条记录
    async fn unlink_task(
        &self,
        tx: &mut Transaction<'_>,
        time_block_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), DbError>;

    /// 取消时间块的所有任务关联
    ///
    /// **预期行为简介:** 删除指定时间块的所有任务关联
    async fn unlink_all_tasks(
        &self,
        tx: &mut Transaction<'_>,
        time_block_id: Uuid,
    ) -> Result<(), DbError>;

    /// 检查时间冲突
    ///
    /// **预期行为简介:** 检查新的时间块是否与现有时间块冲突
    /// **输入输出规范:**
    /// - **前置条件:** start_time <= end_time
    /// - **后置条件:** 如果有冲突返回true，否则返回false
    /// **预期副作用:** 无
    async fn has_time_conflict(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        exclude_id: Option<Uuid>,
    ) -> Result<bool, DbError>;

    /// 查找空闲时间段
    ///
    /// **预期行为简介:** 在指定时间范围内查找未被时间块占用的空闲时间段
    async fn find_free_time_slots(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        min_duration_minutes: i32,
    ) -> Result<Vec<FreeTimeSlot>, DbError>;

    /// 统计时间块使用情况
    ///
    /// **预期行为简介:** 统计时间块的各种使用指标
    async fn get_usage_statistics(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<TimeBlockUsageStats, DbError>;

    /// 查找重复时间块
    ///
    /// **预期行为简介:** 查找基于重复规则生成的时间块
    async fn find_recurring_blocks(&self, parent_id: Uuid) -> Result<Vec<TimeBlock>, DbError>;

    /// 截断时间块
    ///
    /// **预期行为简介:** 将正在进行的时间块截断到指定时间
    /// **输入输出规范:**
    /// - **前置条件:** time_block_id必须存在，truncate_at必须在时间块的时间范围内
    /// - **后置条件:** 时间块的end_time被更新为truncate_at
    /// **预期副作用:** 更新time_blocks表中的end_time和updated_at字段
    async fn truncate_at(
        &self,
        tx: &mut Transaction<'_>,
        time_block_id: Uuid,
        truncate_at: DateTime<Utc>,
    ) -> Result<TimeBlock, DbError>;

    /// 扩展时间块
    ///
    /// **预期行为简介:** 扩展时间块的结束时间
    async fn extend_to(
        &self,
        tx: &mut Transaction<'_>,
        time_block_id: Uuid,
        new_end_time: DateTime<Utc>,
    ) -> Result<TimeBlock, DbError>;

    /// 分割时间块
    ///
    /// **预期行为简介:** 在指定时间点将时间块分割为两个
    async fn split_at(
        &self,
        tx: &mut Transaction<'_>,
        time_block_id: Uuid,
        split_at: DateTime<Utc>,
    ) -> Result<(TimeBlock, TimeBlock), DbError>;
}

/// 空闲时间段
#[derive(Debug, Clone, serde::Serialize)]
pub struct FreeTimeSlot {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_minutes: i64,
}

/// 时间块使用统计
#[derive(Debug, Clone)]
pub struct TimeBlockUsageStats {
    pub total_blocks: i64,
    pub total_duration_minutes: i64,
    pub average_duration_minutes: f64,
    pub blocks_by_area: std::collections::HashMap<Uuid, i64>,
    pub busiest_hour: Option<u32>,
    pub utilization_by_day: Vec<DayUtilization>,
}

/// 日使用率
#[derive(Debug, Clone)]
pub struct DayUtilization {
    pub date: DateTime<Utc>,
    pub scheduled_minutes: i64,
    pub utilization_percentage: f64,
}

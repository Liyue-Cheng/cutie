/// 时间块业务逻辑层

use chrono::Utc;
use uuid::Uuid;

use crate::shared::{
    core::{AppError, AppResult, TimeBlock},
    database::{FreeTimeSlot, TimeBlockRepository, TimeBlockTaskRepository},
};

use super::payloads::{
    CreateTimeBlockPayload, TimeBlockStatsResponse, TimeConflictResponse, UpdateTimeBlockPayload,
};

/// 时间块服务
pub struct TimeBlockService<R: TimeBlockRepository, T: TimeBlockTaskRepository> {
    repository: R,
    task_repository: T,
}

impl<R: TimeBlockRepository, T: TimeBlockTaskRepository> TimeBlockService<R, T> {
    pub fn new(repository: R, task_repository: T) -> Self {
        Self {
            repository,
            task_repository,
        }
    }

    /// 创建新时间块
    pub async fn create_time_block(&self, payload: CreateTimeBlockPayload) -> AppResult<TimeBlock> {
        let now = Utc::now();

        // 检查时间冲突
        if self
            .repository
            .check_conflict(payload.start_time, payload.end_time, None)
            .await?
        {
            return Err(AppError::conflict("指定时间范围与现有时间块冲突"));
        }

        // 创建时间块
        let time_block_id = Uuid::new_v4();
        let mut time_block = TimeBlock::new(time_block_id, payload.start_time, payload.end_time, now)?;

        time_block.title = payload.title;
        time_block.glance_note = payload.glance_note;
        time_block.detail_note = payload.detail_note;
        time_block.area_id = payload.area_id;

        let created_block = self.repository.create(&time_block).await?;

        // 关联任务
        for task_id in payload.task_ids {
            self.task_repository
                .link_task(time_block_id, task_id)
                .await?;
        }

        Ok(created_block)
    }

    /// 根据ID获取时间块
    pub async fn get_time_block(&self, id: Uuid) -> AppResult<TimeBlock> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("TimeBlock", id.to_string()))
    }

    /// 更新时间块
    pub async fn update_time_block(
        &self,
        id: Uuid,
        payload: UpdateTimeBlockPayload,
    ) -> AppResult<TimeBlock> {
        let mut time_block = self.get_time_block(id).await?;
        let now = Utc::now();

        // 更新时间范围（如果提供了）
        let new_start_time = payload.start_time.unwrap_or(time_block.start_time);
        let new_end_time = payload.end_time.unwrap_or(time_block.end_time);

        // 验证新的时间范围
        if new_start_time != time_block.start_time || new_end_time != time_block.end_time {
            if self
                .repository
                .check_conflict(new_start_time, new_end_time, Some(id))
                .await?
            {
                return Err(AppError::conflict("新的时间范围与现有时间块冲突"));
            }

            time_block.update_time_range(new_start_time, new_end_time, now)?;
        }

        // 更新其他字段
        if let Some(title) = payload.title {
            time_block.title = title;
        }
        if let Some(glance_note) = payload.glance_note {
            time_block.glance_note = glance_note;
        }
        if let Some(detail_note) = payload.detail_note {
            time_block.detail_note = detail_note;
        }
        if let Some(area_id) = payload.area_id {
            time_block.area_id = area_id;
        }

        time_block.updated_at = now;
        self.repository.update(&time_block).await
    }

    /// 删除时间块
    pub async fn delete_time_block(&self, id: Uuid) -> AppResult<()> {
        // 检查时间块是否存在
        self.get_time_block(id).await?;

        // 清理任务关联
        self.task_repository.clear_block_tasks(id).await?;

        // 删除时间块
        self.repository.delete(id).await
    }

    /// 链接任务到时间块
    pub async fn link_task_to_block(&self, time_block_id: Uuid, task_id: Uuid) -> AppResult<()> {
        // 检查时间块是否存在
        self.get_time_block(time_block_id).await?;

        // TODO: 检查任务是否存在
        // 这里需要调用任务服务来验证任务存在

        self.task_repository.link_task(time_block_id, task_id).await
    }

    /// 取消任务与时间块的关联
    pub async fn unlink_task_from_block(
        &self,
        time_block_id: Uuid,
        task_id: Uuid,
    ) -> AppResult<()> {
        self.task_repository
            .unlink_task(time_block_id, task_id)
            .await
    }

    /// 获取时间块列表
    pub async fn get_time_blocks(
        &self,
        date: Option<chrono::DateTime<Utc>>,
        start_date: Option<chrono::DateTime<Utc>>,
        end_date: Option<chrono::DateTime<Utc>>,
        task_id: Option<Uuid>,
        area_id: Option<Uuid>,
    ) -> AppResult<Vec<TimeBlock>> {
        if let Some(task_id) = task_id {
            // 按任务ID查询
            self.repository.find_by_task_id(task_id).await
        } else if let Some(area_id) = area_id {
            // 按领域ID查询
            self.repository.find_by_area_id(area_id).await
        } else if let Some(date) = date {
            // 单日查询
            self.repository.find_by_date(date).await
        } else if let (Some(start), Some(end)) = (start_date, end_date) {
            // 日期范围查询
            self.repository.find_by_date_range(start, end).await
        } else {
            // 默认返回所有时间块
            self.repository.find_all().await
        }
    }

    /// 检查时间冲突
    pub async fn check_time_conflict(
        &self,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
        exclude_id: Option<Uuid>,
    ) -> AppResult<TimeConflictResponse> {
        let has_conflict = self
            .repository
            .check_conflict(start_time, end_time, exclude_id)
            .await?;

        let conflicting_blocks = if has_conflict {
            // 获取冲突的时间块
            let all_blocks = self
                .repository
                .find_by_date_range(start_time, end_time)
                .await?;

            let conflicts: Vec<TimeBlock> = all_blocks
                .into_iter()
                .filter(|block| {
                    // 排除指定的时间块
                    if let Some(exclude_id) = exclude_id {
                        if block.id == exclude_id {
                            return false;
                        }
                    }

                    // 检查时间重叠
                    !(block.end_time <= start_time || block.start_time >= end_time)
                })
                .collect();

            Some(conflicts)
        } else {
            None
        };

        Ok(TimeConflictResponse {
            has_conflict,
            start_time,
            end_time,
            conflicting_blocks,
        })
    }

    /// 查找空闲时间段
    pub async fn find_free_slots(
        &self,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
        min_duration_minutes: i32,
    ) -> AppResult<Vec<FreeTimeSlot>> {
        self.repository
            .find_free_slots(start_time, end_time, min_duration_minutes)
            .await
    }

    /// 获取时间块统计
    pub async fn get_time_block_stats(&self) -> AppResult<TimeBlockStatsResponse> {
        let all_blocks = self.repository.find_all().await?;
        let now = Utc::now();

        let total_count = all_blocks.len() as i64;

        // 今日时间块
        let today_start = crate::shared::core::normalize_to_day_start(now);
        let today_blocks = self.repository.find_by_date(today_start).await?;
        let today_count = today_blocks.len() as i64;

        // 本周时间块
        let week_start = crate::shared::core::get_week_start(now);
        let week_end = crate::shared::core::get_week_end(now);
        let week_blocks = self
            .repository
            .find_by_date_range(week_start, week_end)
            .await?;
        let this_week_count = week_blocks.len() as i64;

        // 本月时间块
        let month_start = crate::shared::core::get_month_start(now);
        let month_end = crate::shared::core::get_month_end(now);
        let month_blocks = self
            .repository
            .find_by_date_range(month_start, month_end)
            .await?;
        let this_month_count = month_blocks.len() as i64;

        // 计算总计划时间
        let total_planned_minutes: i64 = all_blocks
            .iter()
            .map(|block| block.duration_minutes())
            .sum();

        // 计算平均时长
        let avg_duration_minutes = if total_count > 0 {
            total_planned_minutes as f64 / total_count as f64
        } else {
            0.0
        };

        // 按领域分组统计
        let mut by_area = std::collections::HashMap::new();
        for block in &all_blocks {
            let area_key = block
                .area_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "无领域".to_string());
            *by_area.entry(area_key).or_insert(0) += 1;
        }

        Ok(TimeBlockStatsResponse {
            total_count,
            today_count,
            this_week_count,
            this_month_count,
            total_planned_minutes,
            avg_duration_minutes,
            by_area,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::time_blocks::repository::{SqlxTimeBlockRepository, SqlxTimeBlockTaskRepository},
        shared::database::connection::create_test_database,
    };

    #[tokio::test]
    async fn test_create_time_block() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTimeBlockRepository::new(pool.clone());
        let task_repository = SqlxTimeBlockTaskRepository::new(pool);
        let service = TimeBlockService::new(repository, task_repository);

        let now = Utc::now();
        let payload = CreateTimeBlockPayload {
            title: Some("Meeting".to_string()),
            glance_note: Some("Team meeting".to_string()),
            detail_note: None,
            start_time: now + chrono::Duration::hours(1),
            end_time: now + chrono::Duration::hours(2),
            area_id: None,
            task_ids: vec![],
        };

        let time_block = service.create_time_block(payload).await.unwrap();
        assert_eq!(time_block.title, Some("Meeting".to_string()));
        assert_eq!(time_block.duration_minutes(), 60);
    }

    #[tokio::test]
    async fn test_time_conflict_detection() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTimeBlockRepository::new(pool.clone());
        let task_repository = SqlxTimeBlockTaskRepository::new(pool);
        let service = TimeBlockService::new(repository, task_repository);

        let now = Utc::now();

        // 创建第一个时间块
        let payload1 = CreateTimeBlockPayload {
            title: Some("First Block".to_string()),
            glance_note: None,
            detail_note: None,
            start_time: now + chrono::Duration::hours(1),
            end_time: now + chrono::Duration::hours(2),
            area_id: None,
            task_ids: vec![],
        };
        service.create_time_block(payload1).await.unwrap();

        // 尝试创建冲突的时间块
        let payload2 = CreateTimeBlockPayload {
            title: Some("Conflicting Block".to_string()),
            glance_note: None,
            detail_note: None,
            start_time: now + chrono::Duration::minutes(90),
            end_time: now + chrono::Duration::minutes(150),
            area_id: None,
            task_ids: vec![],
        };

        let result = service.create_time_block(payload2).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("冲突"));
    }

    #[tokio::test]
    async fn test_find_free_slots() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTimeBlockRepository::new(pool.clone());
        let task_repository = SqlxTimeBlockTaskRepository::new(pool);
        let service = TimeBlockService::new(repository, task_repository);

        let now = Utc::now();

        // 创建时间块（上午9-10点）
        let payload = CreateTimeBlockPayload {
            title: Some("Morning Block".to_string()),
            glance_note: None,
            detail_note: None,
            start_time: now + chrono::Duration::hours(9),
            end_time: now + chrono::Duration::hours(10),
            area_id: None,
            task_ids: vec![],
        };
        service.create_time_block(payload).await.unwrap();

        // 查找空闲时间段（上午8点到下午6点，最少30分钟）
        let free_slots = service
            .find_free_slots(
                now + chrono::Duration::hours(8),
                now + chrono::Duration::hours(18),
                30,
            )
            .await
            .unwrap();

        // 应该找到至少一个空闲时间段
        assert!(!free_slots.is_empty());
    }
}

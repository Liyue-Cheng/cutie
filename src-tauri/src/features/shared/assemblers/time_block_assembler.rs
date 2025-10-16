/// TimeBlock 相关 DTO 装配器
/// 用于事件载荷中的完整 TimeBlock 数据组装
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TimeBlock, TimeBlockRow, TimeBlockViewDto},
    infra::core::{AppError, AppResult, DbError},
};

use super::LinkedTaskAssembler;

pub struct TimeBlockAssembler;

impl TimeBlockAssembler {
    /// 查询并组装完整的 TimeBlockViewDto（用于事件载荷）
    /// ✅ 禁止片面数据：返回完整对象
    ///
    /// 这个函数替代了 complete_task.rs、delete_task.rs、update_task.rs 中重复的 ~100 行代码
    pub async fn assemble_for_event_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        time_block_ids: &[Uuid],
    ) -> AppResult<Vec<TimeBlockViewDto>> {
        if time_block_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();

        for block_id in time_block_ids {
            // 1. 查询时间块（✅ 完整字段列表）
            // 🔥 事件装配器需要获取所有时间块数据，包括刚被软删除的（用于副作用）
            let query = r#"
                SELECT id, title, glance_note, detail_note, start_time, end_time,
                       start_time_local, end_time_local, time_type, creation_timezone,
                       is_all_day, area_id, created_at, updated_at, is_deleted, source_info,
                       external_source_id, external_source_provider, external_source_metadata,
                       recurrence_rule, recurrence_parent_id, recurrence_original_date
                FROM time_blocks
                WHERE id = ?
            "#;

            let block_row = sqlx::query_as::<_, TimeBlockRow>(query)
                .bind(block_id.to_string())
                .fetch_optional(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

            if let Some(row) = block_row {
                let block = TimeBlock::try_from(row)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;

                // 2. 查询关联的任务
                let linked_tasks =
                    LinkedTaskAssembler::get_for_time_block(&mut **tx, *block_id).await?;

                // 3. 组装 TimeBlockViewDto（✅ area_id 已直接从 block 获取）
                let view = TimeBlockViewDto {
                    id: block.id,
                    start_time: block.start_time,
                    end_time: block.end_time,
                    start_time_local: block.start_time_local,
                    end_time_local: block.end_time_local,
                    time_type: block.time_type,
                    creation_timezone: block.creation_timezone,
                    is_all_day: block.is_all_day,
                    title: block.title,
                    glance_note: block.glance_note,
                    detail_note: block.detail_note,
                    area_id: block.area_id,
                    linked_tasks,
                    is_recurring: block.recurrence_rule.is_some(),
                };

                result.push(view);
            }
        }

        Ok(result)
    }

    /// 从 TimeBlock 实体组装视图（非事务版本）
    pub async fn assemble_view(
        block: &TimeBlock,
        pool: &SqlitePool,
    ) -> AppResult<TimeBlockViewDto> {
        // 1. 创建基础视图（✅ area_id 已直接从 block 获取）
        let mut view = TimeBlockViewDto {
            id: block.id,
            start_time: block.start_time,
            end_time: block.end_time,
            start_time_local: block.start_time_local.clone(),
            end_time_local: block.end_time_local.clone(),
            time_type: block.time_type,
            creation_timezone: block.creation_timezone.clone(),
            is_all_day: block.is_all_day,
            title: block.title.clone(),
            glance_note: block.glance_note.clone(),
            detail_note: block.detail_note.clone(),
            area_id: block.area_id,
            linked_tasks: Vec::new(),
            is_recurring: block.recurrence_rule.is_some(),
        };

        // 2. 获取关联的任务
        view.linked_tasks = LinkedTaskAssembler::get_for_time_block(pool, block.id).await?;

        Ok(view)
    }
}

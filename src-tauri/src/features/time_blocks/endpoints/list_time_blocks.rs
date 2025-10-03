/// 获取时间块列表 API - 单文件组件
///
/// 支持按日期范围查询时间块
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    entities::{TimeBlock, TimeBlockViewDto},
    features::{
        tasks::shared::assemblers::LinkedTaskAssembler,
        time_blocks::shared::repositories::TimeBlockRepository,
    },
    shared::{core::AppResult, http::error_handler::success_response},
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `list_time_blocks`

## API端点
GET /api/time-blocks?start_date=...&end_date=...

## 预期行为简介
查询指定时间范围内的所有时间块，返回包含关联任务信息的视图模型列表。

## 输入输出规范
- **查询参数**:
  - start_date: 开始时间（ISO 8601 UTC）
  - end_date: 结束时间（ISO 8601 UTC）
- **后置条件**:
  - 返回该时间范围内所有未删除的时间块
  - 每个时间块包含关联的任务摘要
  - 时间块按 start_time 排序

## 边界情况
- 如果时间范围无效（start > end），返回 400
- 如果没有时间块，返回空数组

## 预期副作用
- 无（只读操作）
*/

// ==================== 请求参数 ====================
#[derive(Debug, Deserialize)]
pub struct ListTimeBlocksQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<ListTimeBlocksQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(time_blocks) => success_response(time_blocks).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        query: ListTimeBlocksQuery,
    ) -> AppResult<Vec<TimeBlockViewDto>> {
        let pool = app_state.db_pool();

        // 1. 解析时间范围
        let start_time = query
            .start_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let end_time = query
            .end_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // 2. 查询时间块（✅ 使用共享 Repository）
        let time_blocks = TimeBlockRepository::find_in_range(pool, start_time, end_time).await?;

        // 3. 为每个时间块组装视图模型
        let mut result = Vec::new();
        for block in time_blocks {
            let view = assemble_time_block_view(&block, pool).await?;
            result.push(view);
        }

        // 4. 按 start_time 排序
        result.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        Ok(result)
    }

    /// 组装单个时间块的视图模型
    async fn assemble_time_block_view(
        block: &TimeBlock,
        pool: &sqlx::SqlitePool,
    ) -> AppResult<TimeBlockViewDto> {
        // 1. 创建基础视图（✅ area_id 已直接从 block 获取）
        let mut view = TimeBlockViewDto {
            id: block.id,
            start_time: block.start_time,
            end_time: block.end_time,
            title: block.title.clone(),
            glance_note: block.glance_note.clone(),
            detail_note: block.detail_note.clone(),
            area_id: block.area_id,
            linked_tasks: Vec::new(),
            is_recurring: block.recurrence_rule.is_some(),
        };

        // 2. 获取关联的任务（✅ 使用共享 Assembler）
        view.linked_tasks = LinkedTaskAssembler::get_for_time_block(pool, block.id).await?;

        Ok(view)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TimeBlockRepository::find_in_range
// - LinkedTaskAssembler::get_for_time_block

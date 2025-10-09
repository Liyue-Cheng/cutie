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

## 1. 端点签名 (Endpoint Signature)

GET /api/time-blocks?start_date={start_date}&end_date={end_date}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我查看日历视图时，我需要看到特定时间范围内的所有时间块，
> 包括每个时间块关联的任务信息，以便我能够了解我的日程安排和待办事项。

### 2.2. 核心业务逻辑 (Core Business Logic)

查询指定时间范围内的所有未删除的时间块，并为每个时间块组装完整的视图模型。
返回的数据包括：
1. 时间块的基本信息（时间、标题、笔记、区域）
2. 关联的任务摘要列表（任务ID、标题、完成状态）
3. 是否为循环时间块的标记

查询结果按 `start_time` 升序排序，方便前端按时间顺序展示。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**Query Parameters:**
- `start_date` (string, optional): 开始时间（ISO 8601 UTC 格式）
- `end_date` (string, optional): 结束时间（ISO 8601 UTC 格式）

**注意**：两个参数都是可选的：
- 如果都不提供，返回所有时间块
- 如果只提供 `start_date`，返回该时间之后的所有时间块
- 如果只提供 `end_date`，返回该时间之前的所有时间块
- 如果都提供，返回该时间范围内的时间块

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `Array<TimeBlockViewDto>`

```json
[
  {
    "id": "uuid",
    "start_time": "2025-10-05T09:00:00Z",
    "end_time": "2025-10-05T10:00:00Z",
    "start_time_local": "09:00:00",
    "end_time_local": "10:00:00",
    "time_type": "FLOATING",
    "creation_timezone": "Asia/Shanghai",
    "is_all_day": false,
    "title": "string | null",
    "glance_note": "string | null",
    "detail_note": "string | null",
    "area_id": "uuid | null",
    "linked_tasks": [
      {
        "id": "uuid",
        "title": "string",
        "is_completed": false
      }
    ],
    "is_recurring": false
  },
  {
    "id": "uuid",
    "start_time": "2025-10-05T14:00:00Z",
    "end_time": "2025-10-05T15:00:00Z",
    "start_time_local": "14:00:00",
    "end_time_local": "15:00:00",
    "time_type": "FLOATING",
    "creation_timezone": "Asia/Shanghai",
    "is_all_day": false,
    "title": "string | null",
    "glance_note": "string | null",
    "detail_note": "string | null",
    "area_id": "uuid | null",
    "linked_tasks": [],
    "is_recurring": false
  }
]
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "时间范围参数格式无效"
}
```

**空结果情况:**

如果指定时间范围内没有时间块，返回空数组 `[]`。

## 4. 验证规则 (Validation Rules)

- `start_date`:
    - 如果提供，**必须**是有效的 ISO 8601 格式（支持 RFC3339）。
    - 如果格式无效，将被忽略（视为未提供）。
- `end_date`:
    - 如果提供，**必须**是有效的 ISO 8601 格式（支持 RFC3339）。
    - 如果格式无效，将被忽略（视为未提供）。
- **时间范围逻辑**:
    - 不要求 `start_date < end_date`（由数据库查询自然处理）。
    - 如果 `start_date >= end_date`，可能返回空数组（取决于数据）。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  解析查询参数：
    - 尝试将 `start_date` 字符串解析为 `DateTime<Utc>`
    - 尝试将 `end_date` 字符串解析为 `DateTime<Utc>`
    - 如果解析失败，将对应参数设为 `None`
2.  调用 `TimeBlockRepository::find_in_range` 查询时间块：
    - 传入 `start_time` 和 `end_time`（可能为 `None`）
    - 查询所有未删除的时间块（`deleted_at IS NULL`）
    - 根据时间范围过滤结果
3.  对每个时间块，调用 `assemble_time_block_view` 组装视图模型：
    - 创建 `TimeBlockViewDto` 基础对象
    - 填充所有基础字段（`id`, `start_time`, `end_time`, `title`, 等）
    - 调用 `LinkedTaskAssembler::get_for_time_block` 查询关联的任务
    - 填充 `linked_tasks` 字段
    - 设置 `is_recurring` 标记（基于 `recurrence_rule` 是否为空）
4.  对结果列表按 `start_time` 升序排序。
5.  返回 `200 OK` 和时间块视图列表。

## 6. 边界情况 (Edge Cases)

- **没有提供时间范围参数:** 返回所有未删除的时间块。
- **时间范围内没有时间块:** 返回空数组 `[]`。
- **`start_date` 格式无效:** 忽略该参数，相当于没有下限。
- **`end_date` 格式无效:** 忽略该参数，相当于没有上限。
- **`start_date >= end_date`:** 可能返回空数组或部分结果（取决于数据）。
- **时间块没有关联任务:** `linked_tasks` 字段为空数组 `[]`。
- **时间块关联多个任务:** `linked_tasks` 包含所有关联任务的摘要。
- **大量时间块:** 当前实现一次性加载所有结果（未来可能需要分页）。
- **跨时区查询:** 所有时间都使用 UTC，前端负责时区转换和展示。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询指定范围内的时间块（`time_blocks` 表）。
    - **`SELECT`:** N次，为每个时间块查询关联的任务（`task_time_block_links` 和 `tasks` 表）。
    - **注意**：当前实现使用 N+1 查询模式，可能需要优化为 JOIN 查询（性能考虑）。
    - **无事务**：只读操作，不使用事务。
- **性能考虑:**
    - 时间块数量较多时，可能需要较长查询时间。
    - 未来可能需要实现分页或虚拟滚动。
- **日志记录:**
    - 失败时，记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*
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

        // 2. 获取关联的任务（✅ 使用共享 Assembler）
        view.linked_tasks = LinkedTaskAssembler::get_for_time_block(pool, block.id).await?;

        Ok(view)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TimeBlockRepository::find_in_range
// - LinkedTaskAssembler::get_for_time_block

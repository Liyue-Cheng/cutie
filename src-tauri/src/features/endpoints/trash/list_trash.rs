/// 回收站列表 API - 单文件组件
///
/// 查询回收站中的任务列表
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
    entities::task::response_dtos::TaskCardDto,
    features::shared::repositories::TaskRepository,
    features::shared::ViewTaskCardAssembler,
    infra::{core::AppResult, http::error_handler::success_response},
    startup::AppState,
};

/// 回收站查询参数
#[derive(Debug, Deserialize)]
pub struct ListTrashQuery {
    /// 每页数量，默认 50
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// 偏移量，默认 0
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// 回收站响应
#[derive(Debug, Serialize)]
pub struct ListTrashResponse {
    pub tasks: Vec<TaskCardDto>,
    pub total: usize,
}

// ==================== 文档层 ====================
/*
CABC for `list_trash`

## 1. 端点签名 (Endpoint Signature)

GET /api/trash

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想查看回收站中的所有已删除任务，
> 以便我可以选择恢复或彻底删除它们。

### 2.2. 核心业务逻辑 (Core Business Logic)

查询回收站中的任务列表（deleted_at IS NOT NULL），
按删除时间倒序排列（最近删除的在前）。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**Query Parameters:**
- `limit` (integer, optional): 每页数量，默认 50
- `offset` (integer, optional): 偏移量，默认 0

### 3.2. 响应 (Responses)

**200 OK:**

```json
{
  "tasks": [
    {
      "id": "uuid",
      "title": "string",
      "deleted_at": "2025-10-08T12:00:00Z",
      ...
    }
  ],
  "total": 10
}
```

## 4. 验证规则 (Validation Rules)

- `limit`: 必须 > 0，最大 100
- `offset`: 必须 >= 0

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 解析查询参数（limit、offset）
2. 查询回收站中的任务列表（TaskRepository::find_deleted_tasks）
3. 为每个任务装配完整的 TaskCardDto
4. 返回任务列表和总数

## 6. 边界情况 (Edge Cases)

- **回收站为空:** 返回空数组
- **offset 超出范围:** 返回空数组
- **limit 过大:** 限制为最大 100

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 查询回收站中的任务

## 8. 契约 (Contract)

### Pre-conditions:
- 无

### Post-conditions:
- 返回回收站中的任务列表

### Invariants:
- 只返回 deleted_at IS NOT NULL 的任务
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<ListTrashQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        query: ListTrashQuery,
    ) -> AppResult<ListTrashResponse> {
        // 验证参数
        let limit = query.limit.min(100).max(1);
        let offset = query.offset.max(0);

        // 查询回收站中的任务
        let tasks = TaskRepository::find_deleted_tasks(app_state.db_pool(), limit, offset).await?;

        // 装配完整的 TaskCardDto
        let task_cards = ViewTaskCardAssembler::assemble_batch(tasks, app_state.db_pool()).await?;

        Ok(ListTrashResponse {
            total: task_cards.len(),
            tasks: task_cards,
        })
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_deleted_tasks
// - ViewTaskCardAssembler::assemble_batch

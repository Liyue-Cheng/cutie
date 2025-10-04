/// 获取视图排序偏好 API - 单文件组件
/// GET /view-preferences/:context_key
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::{
    entities::view_preference::{ViewPreference, ViewPreferenceDto, ViewPreferenceRow},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_view_preference`

## 1. 端点签名 (Endpoint Signature)

GET /api/view-preferences/:context_key

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户,我想要获取某个特定视图(如Staging区、今日看板、项目看板等)的任务排序配置,
> 以便应用程序能够按照我上次保存的顺序显示任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据视图上下文标识(context_key)查询该视图的任务排序偏好。
返回包含排序后的任务ID数组和最后更新时间的 `ViewPreferenceDto`。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `context_key` (String, required): 视图上下文唯一标识
  - 格式规范:
    - Staging区: `misc::staging`
    - 每日视图: `daily::YYYY-MM-DD` (如 `daily::2025-10-03`)
    - Area视图: `area::{uuid}` (如 `area::a1b2c3d4-...`)
    - Project视图: `project::{uuid}` (如 `project::a1b2c3d4-...`)

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `ViewPreferenceDto`

```json
{
  "context_key": "daily::2025-10-03",
  "sorted_task_ids": [
    "task-uuid-1",
    "task-uuid-2",
    "task-uuid-3"
  ],
  "updated_at": "2025-10-05T12:00:00Z"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "ViewPreference not found: daily::2025-10-03"
}
```

## 4. 验证规则 (Validation Rules)

- `context_key`:
    - **必须**存在于 URL 路径中。
    - 格式不做强制校验(由调用者保证格式正确性)。
    - 如果数据库中不存在对应记录,返回 `404 NOT_FOUND`。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从 URL 路径参数中提取 `context_key`。
2.  调用数据访问层查询视图偏好记录(`database::find_by_context_key`)。
3.  如果记录不存在,返回 `404 NOT_FOUND` 错误。
4.  将数据库行(`ViewPreferenceRow`)转换为领域实体(`ViewPreference`):
    - 解析 JSON 字符串 `sorted_task_ids` 为字符串数组。
    - 解析 RFC 3339 字符串 `updated_at` 为 DateTime<Utc>。
5.  将领域实体转换为 DTO(`ViewPreferenceDto`)。
6.  返回 `200 OK` 和 DTO。

## 6. 边界情况 (Edge Cases)

- **context_key 不存在:** 返回 `404 NOT_FOUND` 错误。
- **sorted_task_ids 为空数组:** 正常返回,允许空数组(表示该视图暂无任务或所有任务已删除)。
- **sorted_task_ids 包含已删除的任务ID:** 不做验证,返回原始数据(前端负责过滤无效ID)。
- **context_key 格式错误:** 不做格式校验,直接查询(数据库查询不到则返回404)。
- **幂等性:** 多次查询相同 `context_key`,结果一致(无副作用)。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次查询 `view_preferences` 表。
- **无写操作:** 此端点为只读查询,不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时(如记录不存在),以 `WARN` 级别记录错误信息。

*(无其他已知副作用)*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(context_key): Path<String>,
) -> Response {
    match logic::execute(&app_state, &context_key).await {
        Ok(preference_dto) => success_response(preference_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, context_key: &str) -> AppResult<ViewPreferenceDto> {
        let pool = app_state.db_pool();

        // 查询视图偏好
        let preference = database::find_by_context_key(pool, context_key)
            .await?
            .ok_or_else(|| AppError::not_found("ViewPreference", context_key))?;

        // 转换为 DTO
        Ok(ViewPreferenceDto {
            context_key: preference.context_key,
            sorted_task_ids: preference.sorted_task_ids,
            updated_at: preference.updated_at,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    pub async fn find_by_context_key(
        pool: &sqlx::SqlitePool,
        context_key: &str,
    ) -> AppResult<Option<ViewPreference>> {
        let query = r#"
            SELECT context_key, sorted_task_ids, updated_at
            FROM view_preferences
            WHERE context_key = ?
        "#;

        let row = sqlx::query_as::<_, ViewPreferenceRow>(query)
            .bind(context_key)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(row) => {
                let pref = ViewPreference::try_from(row)?;
                Ok(Some(pref))
            }
            None => Ok(None),
        }
    }
}

/// 保存视图排序偏好 API - 单文件组件
/// PUT /view-preferences/:context_key
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::view_preference::{
        SaveViewPreferenceRequest, ViewPreference, ViewPreferenceDto, ViewPreferenceRow,
    },
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `save_view_preference`

## 1. 端点签名 (Endpoint Signature)

PUT /api/view-preferences/:context_key

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户,当我在某个视图(如Staging区、今日看板、项目看板等)中拖拽调整任务顺序后,
> 我想要系统能够持久化保存这个排序配置,以便下次打开该视图时能恢复我上次的排序。

### 2.2. 核心业务逻辑 (Core Business Logic)

保存或更新某个视图的任务排序偏好。使用 UPSERT 逻辑(INSERT OR REPLACE),
如果该 `context_key` 已存在则更新,否则创建新记录。
排序配置以任务ID数组的形式存储,数组顺序即为任务的显示顺序。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `context_key` (String, required): 视图上下文标识
  - 格式规范:
    - Staging区: `misc::staging`
    - 每日视图: `daily::YYYY-MM-DD` (如 `daily::2025-10-03`)
    - Area视图: `area::{uuid}` (如 `area::a1b2c3d4-...`)
    - Project视图: `project::{uuid}` (如 `project::a1b2c3d4-...`)

**请求体 (Request Body):** `application/json`

```json
{
  "sorted_task_ids": ["string"] (required, 任务ID数组,非空)
}
```

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

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "context_key", "code": "CONTEXT_KEY_EMPTY", "message": "Context key 不能为空" }
  ]
}
```

或

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "context_key", "code": "CONTEXT_KEY_EMPTY", "message": "Context key 不能为空" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `context_key`:
    - **必须**存在且为非空字符串(trim后)。
    - 违反时返回错误码: `CONTEXT_KEY_EMPTY`
- `sorted_task_ids`:
    - **允许**为空数组（看板可能为空，例如删除了所有任务）。
    - 注意: 允许包含重复的任务ID(不做唯一性校验)
    - 注意: 不验证任务ID是否真实存在于数据库中(前端负责保证数据有效性)

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  验证请求体:
    - 检查 `context_key` 是否为空(trim后)。
2.  通过 `Clock` 服务获取当前时间 `now`。
3.  构建 `ViewPreference` 领域实体:
    - 设置 `context_key` 为请求中的值。
    - 设置 `sorted_task_ids` 为请求中的数组。
    - 设置 `updated_at` 为当前时间。
4.  调用数据访问层执行 UPSERT 操作(`database::upsert`):
    - 将 `sorted_task_ids` 数组序列化为 JSON 字符串。
    - 将 `updated_at` 转换为 RFC 3339 字符串。
    - 执行 `INSERT ... ON CONFLICT(context_key) DO UPDATE` SQL。
5.  重新查询保存后的记录(`database::find_by_context_key`):
    - 确保返回的数据与数据库中实际存储的数据一致。
6.  将查询结果转换为 DTO(`ViewPreferenceDto`)。
7.  返回 `200 OK` 和 DTO。

## 6. 边界情况 (Edge Cases)

- **context_key 为空字符串或仅包含空格:** 返回 `422` 错误,错误码 `CONTEXT_KEY_EMPTY`。
- **sorted_task_ids 为空数组:** 允许（看板可能为空，例如删除了所有任务）。
- **sorted_task_ids 包含重复的任务ID:** 允许,不做去重处理(保留原始顺序)。
- **sorted_task_ids 包含不存在的任务ID:** 允许,不做验证(前端负责过滤)。
- **重复调用相同的 context_key:** UPSERT 逻辑,每次更新 `sorted_task_ids` 和 `updated_at`。
- **幂等性:** 相同参数重复调用,结果一致(最后更新时间会改变,但排序数据相同)。
- **并发写入相同 context_key:** SQLite 的 UPSERT 语法保证原子性,后执行的请求会覆盖先执行的结果。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`INSERT` 或 `UPDATE`:** 1条记录到 `view_preferences` 表(取决于记录是否已存在)。
        - 新记录: `INSERT` 1条。
        - 已存在: `UPDATE` 1条(更新 `sorted_task_ids` 和 `updated_at`)。
    - **`SELECT`:** 1次查询 `view_preferences` 表(保存后重新查询以返回最新数据)。
    - **无事务包装:** 单条 UPSERT 语句,无需显式事务(SQLite 隐式事务)。
- **无 SSE 事件:** 此端点不发送 SSE 事件(视图偏好是客户端本地状态,无需广播)。
- **日志记录:**
    - 成功时,以 `INFO` 或 `DEBUG` 级别记录保存操作。
    - 失败时(如验证失败或数据库错误),以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*(无其他已知副作用)*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(context_key): Path<String>,
    Json(payload): Json<SaveViewPreferenceRequest>,
) -> Response {
    match logic::execute(&app_state, context_key, payload).await {
        Ok(preference_dto) => success_response(preference_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        context_key: String,
        payload: SaveViewPreferenceRequest,
    ) -> AppResult<ViewPreferenceDto> {
        // 1. 验证 context_key（从路径参数获取）
        if context_key.trim().is_empty() {
            return Err(AppError::validation_error(
                "context_key",
                "Context key 不能为空",
                "CONTEXT_KEY_EMPTY",
            ));
        }

        // ✅ 允许空的任务列表（看板可能为空，例如删除了所有任务）

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        let pool = app_state.db_pool();
        let now = app_state.clock().now_utc();

        // 2. 构建实体
        let preference = ViewPreference {
            context_key,
            sorted_task_ids: payload.sorted_task_ids,
            updated_at: now,
        };

        // 3. 🔥 优化：直接返回 UPSERT 结果，避免二次查询
        let saved = database::upsert_optimized(pool, &preference).await?;

        // 4. 返回 DTO
        Ok(ViewPreferenceDto {
            context_key: saved.context_key,
            sorted_task_ids: saved.sorted_task_ids,
            updated_at: saved.updated_at,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    /// 🔥 优化版本：单次 UPSERT，避免二次查询
    pub async fn upsert_optimized(
        pool: &sqlx::SqlitePool,
        preference: &ViewPreference,
    ) -> AppResult<ViewPreference> {
        // 🔥 预先序列化，减少运行时开销
        let sorted_task_ids_json = serde_json::to_string(&preference.sorted_task_ids)?;
        let updated_at = preference.updated_at.to_rfc3339();

        let query = r#"
            INSERT INTO view_preferences (context_key, sorted_task_ids, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(context_key) DO UPDATE SET
                sorted_task_ids = excluded.sorted_task_ids,
                updated_at = excluded.updated_at
            RETURNING context_key, sorted_task_ids, updated_at
        "#;

        // 🔥 使用 RETURNING 子句避免二次查询
        let row = sqlx::query_as::<_, ViewPreferenceRow>(query)
            .bind(&preference.context_key)
            .bind(&sorted_task_ids_json)
            .bind(&updated_at)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        // 直接返回结果，无需额外查询
        ViewPreference::try_from(row).map_err(|e| AppError::validation_error(
            "view_preference",
            &e,
            "PARSE_ERROR",
        ))
    }

    pub async fn upsert(
        pool: &sqlx::SqlitePool,
        preference: &ViewPreference,
    ) -> AppResult<ViewPreference> {
        // 序列化任务ID数组为 JSON
        let sorted_task_ids_json = serde_json::to_string(&preference.sorted_task_ids)?;

        let updated_at = preference.updated_at.to_rfc3339();

        let query = r#"
            INSERT INTO view_preferences (context_key, sorted_task_ids, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(context_key) DO UPDATE SET
                sorted_task_ids = excluded.sorted_task_ids,
                updated_at = excluded.updated_at
        "#;

        sqlx::query(query)
            .bind(&preference.context_key)
            .bind(&sorted_task_ids_json)
            .bind(&updated_at)
            .execute(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        // 返回更新后的数据
        find_by_context_key(pool, &preference.context_key)
            .await?
            .ok_or_else(|| AppError::not_found("ViewPreference", &preference.context_key))
    }

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
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
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

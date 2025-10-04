/// 创建 Area API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};

use crate::{
    entities::{Area, AreaDto, CreateAreaRequest},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_area`

## 1. 端点签名 (Endpoint Signature)

POST /api/areas

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要创建一个新的领域（Area），以便我能将任务分类到不同的项目或上下文中，
> 并通过颜色标记快速识别不同领域的任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

在数据库中创建一个新的 `Area` 实体。系统将验证名称唯一性（在未删除的领域中），
并确保颜色格式符合十六进制颜色码规范（#RRGGBB）。新领域可以是根领域，也可以指定父领域。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "name": "string (required, 非空)",
  "color": "string (required, 格式: #RRGGBB)",
  "parent_area_id": "string (UUID) | null (optional)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `AreaDto`

```json
{
  "id": "uuid",
  "name": "string",
  "color": "string (#RRGGBB)",
  "parent_area_id": "uuid | null",
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "name", "code": "NAME_EMPTY", "message": "名称不能为空" }
  ]
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "Area 名称已存在"
}
```

## 4. 验证规则 (Validation Rules)

- `name`:
    - **必须**存在。
    - **必须**为非空字符串 (trim后)。
    - 在未删除的领域中**必须**唯一。
    - 违反时返回错误码：`NAME_EMPTY` 或 `CONFLICT`
- `color`:
    - **必须**存在。
    - **必须**符合格式 `#RRGGBB`（7个字符，以#开头，后跟6个十六进制字符）。
    - 违反时返回错误码：`INVALID_COLOR`
- `parent_area_id`:
    - 如果提供，**必须**是有效的 UUID 格式。
    - 如果提供，**应该**指向一个存在的 Area（当前未强制验证）。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation` 验证 `name` 非空。
2.  调用 `Area::validate_color` 验证 `color` 格式。
3.  启动数据库事务（`db_pool().begin()`）。
4.  调用 `database::check_name_exists_in_tx` 检查名称是否已存在。
5.  如果名称已存在，返回 `409 Conflict` 错误并回滚事务。
6.  通过 `IdGenerator` 生成新的 `area_id`（UUID）。
7.  通过 `Clock` 服务获取当前时间 `now`。
8.  构造 `Area` 领域实体对象：
    - 设置 `id = area_id`, `name`, `color`, `parent_area_id`
    - 设置 `created_at = now`, `updated_at = now`
    - 设置 `is_deleted = false`
9.  调用 `database::insert_area_in_tx` 持久化领域到 `areas` 表。
10. 提交数据库事务（`tx.commit()`）。
11. 组装 `AreaDto` 并返回 `201 Created`。

## 6. 边界情况 (Edge Cases)

- **`name` 为空或全空格:** 返回 `422` 错误，错误码 `NAME_EMPTY`。
- **`name` 已存在（未删除的领域中）:** 返回 `409` 错误，消息 "Area 名称已存在"。
- **`color` 格式无效（非 #RRGGBB）:** 返回 `422` 错误，错误码 `INVALID_COLOR`。
- **`parent_area_id` 无效（不存在或格式错误）:** 当前实现中正常返回，未验证父领域存在性。
- **并发创建相同名称:** 可能导致两个检查都通过，但由于唯一约束（如果有）会导致数据库错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询 `areas` 表以检查名称唯一性。
    - **`INSERT`:** 1条记录到 `areas` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 成功时，可能以 `INFO` 级别记录 "Area created successfully"（如有）。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无 SSE 事件，无写入许可，无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateAreaRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(area_dto) => created_response(area_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, request: CreateAreaRequest) -> AppResult<AreaDto> {
        // 1. 验证
        if request.name.trim().is_empty() {
            return Err(AppError::validation_error(
                "name",
                "名称不能为空",
                "NAME_EMPTY",
            ));
        }
        if !Area::validate_color(&request.color) {
            return Err(AppError::validation_error(
                "color",
                "颜色格式无效",
                "INVALID_COLOR",
            ));
        }

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 检查名称唯一性
        let exists = database::check_name_exists_in_tx(&mut tx, &request.name).await?;
        if exists {
            return Err(AppError::conflict("Area 名称已存在"));
        }

        // 3. 生成 ID 和时间戳
        let area_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 4. 创建 Area
        let area = Area {
            id: area_id,
            name: request.name,
            color: request.color,
            parent_area_id: request.parent_area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        database::insert_area_in_tx(&mut tx, &area).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(AreaDto {
            id: area.id,
            name: area.name,
            color: area.color,
            parent_area_id: area.parent_area_id,
            created_at: area.created_at,
            updated_at: area.updated_at,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    pub async fn check_name_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM areas WHERE name = ? AND is_deleted = false";
        let count: i64 = sqlx::query_scalar(query)
            .bind(name)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn insert_area_in_tx(tx: &mut Transaction<'_, Sqlite>, area: &Area) -> AppResult<()> {
        let query = r#"
            INSERT INTO areas (id, name, color, parent_area_id, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(area.id.to_string())
            .bind(&area.name)
            .bind(&area.color)
            .bind(area.parent_area_id.map(|id| id.to_string()))
            .bind(area.created_at.to_rfc3339())
            .bind(area.updated_at.to_rfc3339())
            .bind(area.is_deleted)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }
}

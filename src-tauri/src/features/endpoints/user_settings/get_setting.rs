/// 获取单个用户设置 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::{
    entities::user_setting::UserSettingDto,
    features::user_settings::shared::{create_default_setting_entity, UserSettingRepository},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_setting`

## 1. 端点签名 (Endpoint Signature)

GET /api/user-settings/{key}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查询单个设置项的值，
> 以便我能获取特定配置（如当前语言、显示缩放等）。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据设置键（setting_key）查询对应的设置项。如果数据库中不存在，
则返回该键的默认值（如果该键在默认设置列表中）。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `key` (String, required): 设置键，例如 `appearance.language`

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `UserSettingDto`

```json
{
  "setting_key": "appearance.language",
  "setting_value": "\"en\"",
  "value_type": "string",
  "updated_at": "2025-01-11T12:00:00Z",
  "created_at": "2025-01-11T12:00:00Z"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Setting key 'unknown.key' not found and has no default"
}
```

## 4. 验证规则 (Validation Rules)

- `key`:
    - **必须**存在于数据库中或在默认设置列表中。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从路径参数中提取 `key`。
2.  查询数据库（`UserSettingRepository::find_by_key`）。
3.  如果找到记录，直接返回。
4.  如果未找到，尝试从默认设置列表中获取该键的默认值。
5.  如果默认值也不存在，返回 `404` 错误。
6.  组装 `UserSettingDto` 并返回 `200 OK`。

## 6. 边界情况 (Edge Cases)

- **设置不存在且无默认值:** 返回 `404` 错误。
- **设置存在于默认列表但未保存:** 返回默认值。
- **key 为空字符串:** 返回 `404` 错误（无匹配记录）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询 `user_settings` 表的单条记录。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（如设置不存在），以 `WARN` 级别记录错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(key): Path<String>) -> Response {
    match logic::execute(&app_state, key).await {
        Ok(setting) => success_response(setting).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, key: String) -> AppResult<UserSettingDto> {
        let pool = app_state.db_pool();

        // 查询设置
        let setting = UserSettingRepository::find_by_key(pool, &key).await?;

        // 如果不存在，尝试返回默认值
        let setting = match setting {
            Some(s) => s,
            None => create_default_setting_entity(&key)
                .ok_or_else(|| AppError::not_found("user_setting", key))?,
        };

        let dto: UserSettingDto = setting.into();

        Ok(dto)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已使用共享 Repository：
// - UserSettingRepository::find_by_key

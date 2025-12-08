/// 获取所有用户设置 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    entities::user_setting::UserSettingDto,
    features::user_settings::shared::{create_all_default_entities, UserSettingRepository},
    infra::{core::AppResult, http::error_handler::success_response},
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_all_settings`

## 1. 端点签名 (Endpoint Signature)

GET /api/user-settings

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要获取所有的用户设置项，
> 以便我能查看和管理我的应用配置（如语言、缩放、主题等）。

### 2.2. 核心业务逻辑 (Core Business Logic)

查询数据库中所有的用户设置项。如果数据库为空（首次访问），
则返回系统默认设置列表。所有设置按 category 和 setting_key 排序返回。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**无请求参数**

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `Vec<UserSettingDto>`

```json
[
  {
    "setting_key": "appearance.language",
    "setting_value": "\"en\"",
    "value_type": "string",
    "updated_at": "2025-01-11T12:00:00Z",
    "created_at": "2025-01-11T12:00:00Z"
  },
  {
    "setting_key": "appearance.display_scale",
    "setting_value": "100",
    "value_type": "number",
    "updated_at": "2025-01-11T12:00:00Z",
    "created_at": "2025-01-11T12:00:00Z"
  }
]
```

**500 Internal Server Error:**

```json
{
  "error_code": "INTERNAL_ERROR",
  "message": "Database query failed"
}
```

## 4. 验证规则 (Validation Rules)

- 无输入验证需求（无请求参数）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  查询数据库中所有用户设置（`UserSettingRepository::find_all`）。
2.  如果数据库返回空列表（首次使用），返回系统默认设置列表。
3.  将所有设置实体转换为 `UserSettingDto`。
4.  返回设置列表（已按 category, setting_key 排序）。

## 6. 边界情况 (Edge Cases)

- **数据库为空（首次访问）:** 返回包含所有默认设置的列表（9个设置项）。
- **部分设置缺失:** 只返回已保存的设置，不会自动填充缺失的默认值。
- **数据库连接失败:** 返回 `500` 错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询 `user_settings` 表所有记录。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（如数据库错误），以 `ERROR` 级别记录错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(settings) => success_response(settings).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<UserSettingDto>> {
        let pool = app_state.db_pool();

        // 查询所有设置
        let mut settings = UserSettingRepository::find_all(pool).await?;

        // 如果数据库为空，返回默认设置
        if settings.is_empty() {
            settings = create_all_default_entities();
        }

        // 转换为 DTO
        let dtos: Vec<UserSettingDto> = settings.into_iter().map(|s| s.into()).collect();

        Ok(dtos)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已使用共享 Repository：
// - UserSettingRepository::find_all

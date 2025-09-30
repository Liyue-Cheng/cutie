# Rust 单文件组件 (SFC) 规范

本规范定义了在本项目后端采用的“单文件组件”（Single-File Component, SFC）架构模式。此模式旨在将一个独立的业务功能（通常对应一个API端点）的所有相关代码（文档、路由、验证、业务逻辑、数据访问）聚合在单个 `.rs` 文件中，以提升内聚性、可维护性和开发效率。

## 1. 核心理念

- **高内聚**: 一个文件的改动对应一个业务功能的修改。
- **低耦合**: 各个SFC之间应尽可能独立，减少跨文件依赖。
- **关注点分离 (Separation of Concerns)**: 在文件内部通过 `mod` 模块化组织不同层次的代码，实现逻辑上的清晰分层。
- **约定优于配置**: 遵循统一的结构和命名约定，降低认知负荷。

## 2. 文件结构

每个SFC文件都应遵循以下内部模块结构。所有模块都是可选的，但建议至少包含 `logic` 和 `database` 模块。

```rust
/// (可选) 文件顶部的文档注释，简要描述SFC的功能。
// --- CABC (Context, Action, Boundary, Consequence) 文档 ---
/*
CABC for `your_feature_name`

## API端点
[METHOD] /api/path/to/endpoint

## 预期行为简介
...

## 输入输出规范
- **前置条件**: ...
- **后置条件**: ...
- **不变量**: ...

## 边界情况
...

## 预期副作用
...

## 请求/响应示例
...
*/

// --- 依赖引入 ---
use axum::{...};
use serde::{Deserialize, Serialize};
use sqlx::{...};
use uuid::Uuid;
// ... 其他 crate 依赖 ...
use crate::{...}; // 内部依赖

// --- (可选) 请求/响应结构体定义 ---
#[derive(Deserialize)]
pub struct FeatureRequest { ... }

#[derive(Serialize)]
pub struct FeatureResponse { ... }

// --- HTTP 处理器 (Handler) ---
/// Axum HTTP处理器，作为SFC的入口。
/// 职责:
/// 1. 从HTTP请求中提取数据（State, Path, Json, Query等）。
/// 2. 调用 `logic::execute` 函数。
/// 3. 将 `logic::execute` 的 `Result` 转换为 `axum::response::Response`。
pub async fn handle(
    State(app_state): State<AppState>,
    // ... 其他 extractors ...
) -> Response {
    match logic::execute(&app_state, /* ... */).await {
        Ok(result) => success_response(result).into_response(), // 或 created_response, etc.
        Err(err) => err.into_response(),
    }
}

// --- 验证层 (Validation Layer) ---
/// **可选** 模块，用于处理复杂的输入验证。
/// 职责:
/// 1. 验证请求数据的格式、范围、业务规则。
/// 2. 将原始请求结构体 (`FeatureRequest`) 转换为已验证的数据结构 (`ValidatedData`)。
/// 3. 返回 `Result<ValidatedData, Vec<ValidationError>>`。
mod validation {
    use super::*;

    pub struct ValidatedData { ... }

    pub fn validate_request(request: &FeatureRequest) -> Result<ValidatedData, Vec<ValidationError>> {
        // ... 验证逻辑 ...
    }
}

// --- 业务逻辑层 (Business Logic Layer) ---
/// **核心** 模块，包含该功能的主要业务逻辑。
/// 职责:
/// 1. (可选) 调用 `validation` 模块进行输入验证。
/// 2. 编排一个或多个 `database` 模块中的函数来完成业务目标。
/// 3. 处理业务错误和边界情况。
/// 4. 不直接进行SQL查询，而是调用 `database` 模块的函数。
/// 5. 开启和提交事务。
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, /* ... */) -> AppResult<FeatureResponse> {
        // 1. (可选) 验证
        let validated_data = validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        // 2. 开启事务
        let mut tx = app_state.db_pool().begin().await?;

        // 3. 编排数据操作
        let data = database::find_something_in_tx(&mut tx, ...).await?;
        // ...更多逻辑...
        database::update_something_in_tx(&mut tx, ...).await?;

        // 4. 提交事务
        tx.commit().await?;

        // 5. 返回结果
        Ok(FeatureResponse { ... })
    }
}

// --- 数据访问层 (Data Access Layer) ---
/// **核心** 模块，负责所有数据库交互。
/// 职责:
/// 1. 定义与此功能相关的SQL查询。
/// 2. 所有函数都应接受 `Transaction<'_, Sqlite>` 作为参数。
/// 3. 函数应返回 `AppResult<T>`。
/// 4. 将 `sqlx::Error` 包装为 `AppError::DatabaseError`。
/// 5. 将 `sqlx` 返回的 `Row` 结构体转换为领域实体。
mod database {
    use super::*;

    pub async fn find_something_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        id: Uuid,
    ) -> AppResult<Option<Entity>> {
        let row = sqlx::query_as::<_, EntityRow>("SELECT ...")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        row.map(Entity::try_from).transpose().map_err(|e| AppError::DatabaseError(e.into()))
    }

    pub async fn update_something_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        entity: &Entity,
    ) -> AppResult<()> {
        sqlx::query("UPDATE ...")
            .bind(...)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}
```

## 3. 组织和路由

1.  **功能目录**: 每个主要功能（如 `tasks`, `areas`, `schedules`）在 `src-tauri/src/features/` 下拥有自己的目录。
2.  **端点目录**: 在每个功能目录内，创建一个 `endpoints/` 子目录来存放所有的SFC文件。
    ```
    src-tauri/src/features/
    └── tasks/
        ├── endpoints/
        │   ├── create_task.rs
        │   ├── get_task.rs
        │   ├── update_task.rs
        │   └── mod.rs      // 导出所有端点的 handle
        └── mod.rs          // 组装路由
    ```
3.  **端点模块 (`endpoints/mod.rs`)**: 此文件负责公开所有SFC的 `handle` 函数，并可选择性地重命名以避免冲突。

    ```rust
    // src-tauri/src/features/tasks/endpoints/mod.rs
    pub mod create_task;
    pub mod get_task;
    pub mod update_task;

    pub use create_task::handle as create_task_handler;
    pub use get_task::handle as get_task_handler;
    pub use update_task::handle as update_task_handler;
    ```

4.  **功能根模块 (`tasks/mod.rs`)**: 此文件负责将所有端点的 `handle` 函数组装成一个 `axum::Router`。

    ```rust
    // src-tauri/src/features/tasks/mod.rs
    use axum::{routing::{get, post, patch}, Router};
    use crate::startup::AppState;

    pub mod endpoints;
    pub use endpoints::*;

    pub fn create_routes() -> Router<AppState> {
        Router::new()
            .route("/", post(create_task_handler))
            .route("/:id", get(get_task_handler).patch(update_task_handler))
    }
    ```

5.  **顶层路由 (`features/mod.rs`)**: 最顶层的 `mod.rs` 文件将所有功能的路由聚合起来。
    ```rust
    // src-tauri/src/features/mod.rs
    pub fn create_feature_routes() -> Router<AppState> {
        Router::new()
            .nest("/tasks", tasks::create_routes())
            .nest("/areas", areas::create_routes())
            // ...
    }
    ```

## 4. 最佳实践

### 4.1 事务管理

- **业务逻辑层（`logic`）** 负责开启和提交事务
- 所有数据库操作（`database`层函数）都必须在事务中执行
- 只读操作可以省略事务，直接从 `app_state.db_pool()` 获取连接

### 4.2 依赖注入

严格通过 `AppState` 注入依赖，**必须使用正确的方法名**：

| 依赖     | 正确方法                              | ❌ 错误示例       |
| -------- | ------------------------------------- | ----------------- |
| ID生成器 | `app_state.id_generator().new_uuid()` | ~~`.generate()`~~ |
| 时钟     | `app_state.clock().now_utc()`         | ~~`.now()`~~      |
| 数据库   | `app_state.db_pool()`                 | ✅                |

**示例：**

```rust
// ✅ 正确
let task_id = app_state.id_generator().new_uuid();
let now = app_state.clock().now_utc();

// ❌ 错误
let task_id = app_state.id_generator().generate(); // 编译失败
let now = app_state.clock().now();                // 编译失败
```

### 4.3 使用现有工具 - ⚠️ 重要

**禁止重新实现已有功能！** 在编写任何工具函数之前，先检查 `shared/` 模块：

#### **排序算法（LexoRank）**

```rust
// ✅ 正确：使用 shared 中的工具
use crate::shared::core::utils::{
    generate_initial_sort_order,  // 生成初始排序字符串
    get_rank_after,                // 在指定位置之后
    get_rank_before,               // 在指定位置之前
    get_mid_lexo_rank,             // 在两个位置之间
};

let sort_order = get_rank_after(&max)?;

// ❌ 错误：自行实现排序算法
let mut chars: Vec<char> = max.chars().collect();
*last_char = ((*last_char as u8) + 1) as char;  // 不符合 LexoRank 规范
```

#### **时间工具**

```rust
// ✅ 使用 shared 中的时间工具
use crate::shared::core::utils::time_utils;
```

#### **常用 shared 工具**

- `shared/core/utils/sort_order_utils.rs` - LexoRank 排序算法
- `shared/core/utils/time_utils.rs` - 时间处理工具
- `shared/ports/clock.rs` - 时钟接口
- `shared/ports/id_generator.rs` - ID 生成接口

### 4.4 错误处理

- 使用 `AppResult<T>` 和 `AppError` 进行统一的错误处理
- `database` 层将 `sqlx::Error` 转换为 `AppError::DatabaseError`
- `SortOrderError` 会自动转换为 `AppError` (通过 `From` trait)
- 直接使用 `?` 操作符进行错误传播

**示例：**

```rust
// ✅ 正确：利用自动转换
let sort_order = get_rank_after(&max)?;  // SortOrderError -> AppError

// ❌ 错误：手动构造不存在的错误变体
AppError::LexoRankError(...)  // 编译失败
```

### 4.5 幂等性

- 对于 `POST`（创建）和 `DELETE` 操作，应考虑幂等性
- 如果资源已存在或已删除，通常应返回成功状态码（`200 OK` 或 `204 No Content`），而不是错误

### 4.6 代码审查清单

在提交代码前检查：

- [ ] 是否使用了正确的 trait 方法（`new_uuid()`, `now_utc()`）？
- [ ] 是否复用了 `shared/` 中的现有工具？
- [ ] 排序功能是否使用了 LexoRank 工具函数？
- [ ] 错误处理是否使用了 `?` 操作符？
- [ ] 是否在事务中执行了所有写操作？

通过遵循此规范，我们可以构建一个既灵活又有序、易于理解和扩展的后端系统。

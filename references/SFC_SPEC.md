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

## 4. 共享资源清单

**在编写单文件组件之前，请先查看以下共享资源清单，避免重复编写！**

### 4.1 跨功能模块共享资源 (`features/shared`)

这些资源可以在所有功能模块中使用：

#### 📦 Repositories（数据仓库）

- **`AreaRepository`** (`features/shared/repositories/area_repository.rs`)
  - `get_summary(executor, area_id)` - 获取 Area 摘要
  - `get_summaries_batch(executor, area_ids)` - 批量获取 Area 摘要

#### 🔧 Utilities（工具类）

- **`TransactionHelper`** (`features/shared/transaction.rs`)
  - `begin(pool)` - 开始事务（统一错误处理）
  - `commit(tx)` - 提交事务（统一错误处理）

**使用示例：**

```rust
use crate::features::shared::{repositories::AreaRepository, TransactionHelper};

let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
// ... 业务逻辑 ...
TransactionHelper::commit(tx).await?;
```

---

### 4.2 Tasks 模块共享资源 (`features/tasks/shared`)

这些资源专门用于任务相关操作：

#### 📦 Repositories（数据仓库）

- **`TaskRepository`** (`features/tasks/shared/repositories/task_repository.rs`)
  - `find_by_id_in_tx(tx, task_id)` - 在事务中查询任务
  - `find_by_id(pool, task_id)` - 非事务查询任务
  - `insert_in_tx(tx, task)` - 插入任务
  - `update_in_tx(tx, task_id, request)` - 更新任务
  - `soft_delete_in_tx(tx, task_id)` - 软删除任务
  - `set_completed_in_tx(tx, task_id, completed_at)` - 设置任务为已完成
  - `set_reopened_in_tx(tx, task_id, updated_at)` - 重新打开任务

- **`TaskScheduleRepository`** (`features/tasks/shared/repositories/task_schedule_repository.rs`)
  - `has_any_schedule(executor, task_id)` - 检查任务是否有日程
  - `has_schedule_for_day_in_tx(tx, task_id, scheduled_day)` - 检查某天是否有日程
  - `create_in_tx(tx, task_id, scheduled_day)` - 创建日程记录
  - `update_today_to_completed_in_tx(tx, task_id, now)` - 更新当天日程为已完成
  - `delete_future_schedules_in_tx(tx, task_id, now)` - 删除未来日程
  - `delete_all_in_tx(tx, task_id)` - 删除任务的所有日程
  - `get_all_for_task(pool, task_id)` - 获取任务的所有日程记录

- **`TaskTimeBlockLinkRepository`** (`features/tasks/shared/repositories/task_time_block_link_repository.rs`)
  - `link_in_tx(tx, task_id, block_id)` - 创建任务到时间块的链接
  - `delete_all_for_task_in_tx(tx, task_id)` - 删除任务的所有链接
  - `delete_all_for_block_in_tx(tx, block_id)` - 删除时间块的所有链接
  - `find_linked_time_blocks_in_tx(tx, task_id)` - 查询任务链接的所有时间块
  - `is_exclusive_link_in_tx(tx, block_id, task_id)` - 检查时间块是否独占链接某任务
  - `count_remaining_tasks_in_block_in_tx(tx, block_id)` - 统计时间块剩余链接任务数

#### 🏗️ Assemblers（装配器）

- **`TaskAssembler`** (`features/tasks/shared/assembler.rs`)
  - `task_to_card_basic(task)` - 从 Task 实体创建基础 TaskCardDto
  - `task_to_card_full(task, schedule_status, area, schedule_info)` - 创建完整 TaskCardDto
  - `task_to_detail_basic(task)` - 创建基础 TaskDetailDto

- **`LinkedTaskAssembler`** (`features/tasks/shared/assemblers/linked_task_assembler.rs`)
  - `get_summaries_batch(executor, task_ids)` - 批量获取任务摘要
  - `get_for_time_block(executor, block_id)` - 获取时间块关联的任务摘要

- **`TimeBlockAssembler`** (`features/tasks/shared/assemblers/time_block_assembler.rs`)
  - `assemble_for_event_in_tx(tx, time_block_ids)` - 查询并组装完整的 TimeBlockViewDto（用于事件载荷）
  - `assemble_view(block, pool)` - 从 TimeBlock 实体组装视图（非事务版本）

---

### 4.3 TimeBlocks 模块共享资源 (`features/time_blocks/shared`)

这些资源专门用于时间块相关操作：

#### 📦 Repositories（数据仓库）

- **`TimeBlockRepository`** (`features/time_blocks/shared/repositories/time_block_repository.rs`)
  - `find_by_id_in_tx(tx, block_id)` - 在事务中查询时间块
  - `find_by_id(pool, block_id)` - 非事务查询时间块
  - `insert_in_tx(tx, block)` - 插入时间块
  - `update_in_tx(tx, block_id, request, updated_at)` - 更新时间块
  - `soft_delete_in_tx(tx, block_id)` - 软删除时间块
  - `truncate_to_in_tx(tx, block_id, end_time)` - 截断时间块到指定时间
  - `find_in_range(pool, start_time, end_time)` - 查询时间范围内的时间块
  - `exists_in_tx(tx, block_id)` - 检查时间块是否存在

#### 🔍 Utilities（工具类）

- **`TimeBlockConflictChecker`** (`features/time_blocks/shared/conflict_checker.rs`)
  - `check_in_tx(tx, start_time, end_time, exclude_id)` - 检查时间冲突

---

### 4.4 Views 模块共享资源 (`features/views/shared`)

这些资源专门用于视图聚合：

#### 🏗️ Assemblers（装配器）

- **`ViewTaskCardAssembler`** (`features/views/shared/task_card_assembler.rs`)
  - `assemble_full(task, pool)` - 为 Task 组装完整 TaskCard（包括 area、schedule_status）
  - `assemble_batch(tasks, pool)` - 批量组装 TaskCards
  - `assemble_with_status(task, pool, status)` - 组装 TaskCard 并明确设置 schedule_status

---

## 5. 开发原则与规范 ⚠️

### 5.1 高内聚原则

**单文件组件 = 一个完整的业务功能**

- 一个 SFC 文件应该包含处理一个 API 端点所需的所有逻辑
- HTTP 处理、验证、业务逻辑、数据访问都在同一个文件中
- 除非逻辑可以被多个端点复用，否则不要过早抽象

### 5.2 按需抽象原则

**什么时候应该使用共享资源？**
✅ **应该使用共享资源的情况：**

- 共享资源列表（第 4 章）中已有的功能
- 3个或以上的端点使用相同的数据库查询
- 复杂的 DTO 组装逻辑在多处重复

❌ **不应该抽象的情况：**

- 只有 1-2 个端点使用的查询
- 端点特定的验证逻辑
- 简单的数据库操作（INSERT/UPDATE）

### 5.3 共享资源使用规范 🚨

#### ✅ 正确做法：优先使用共享资源

```rust
// ✅ 正确：使用共享 Repository
use crate::features::tasks::shared::repositories::TaskRepository;

let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id).await?;
```

```rust
// ✅ 正确：使用共享 TransactionHelper
use crate::features::shared::TransactionHelper;

let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
// ... 业务逻辑 ...
TransactionHelper::commit(tx).await?;
```

#### ❌ 错误做法：重复编写已有功能

```rust
// ❌ 错误：重复编写查询任务的代码
mod database {
    pub async fn find_task(tx: &mut Transaction, task_id: Uuid) -> AppResult<Task> {
        let query = "SELECT * FROM tasks WHERE id = ?";
        // ... 这个功能 TaskRepository 已经提供了！
    }
}
```

#### 📝 正确做法：端点特定的 SQL 直接写在数据访问层

```rust
// ✅ 正确：端点特定的复杂查询，直接写在 database 模块
mod database {
    pub async fn find_tasks_with_special_filter(
        pool: &SqlitePool,
        custom_criteria: &str,
    ) -> AppResult<Vec<Task>> {
        // 这是端点特定的查询，共享资源中没有
        let query = r#"
            SELECT * FROM tasks
            WHERE custom_field = ?
              AND some_complex_condition
        "#;
        // ... 实现查询
    }
}
```

### 5.4 禁止修改共享资源 🚫

**重要规则：**

- ❌ **禁止**在开发新功能时直接修改 `features/shared`、`features/xxx/shared` 中的文件
- ❌ **禁止**为了一个端点的需求修改共享 Repository
- ✅ **允许**在你的 SFC 的 `database` 模块中编写端点特定的 SQL

**原因：**

- 共享资源被多个端点使用，随意修改可能破坏其他功能
- 共享资源的更新、重构是**重构团队**的职责
- 保持 SFC 的独立性，降低耦合

**如果需要新的共享功能怎么办？**

1. 在你的 SFC 的 `database` 模块中先实现功能
2. 功能验证通过后，由**重构团队**评估是否需要提取到共享资源
3. 重构团队会统一更新共享资源和相关文档

### 5.5 开发流程 📋

```
1️⃣ 查看共享资源清单（第 4 章）
   └─ 需要的功能是否已存在？

2️⃣ 如果存在 → 直接使用共享资源
   └─ 导入对应的 Repository/Assembler

3️⃣ 如果不存在 → 在 SFC 的 database 模块中编写 SQL
   └─ 不要修改共享资源！

4️⃣ 功能完成后 → 提交代码审查
   └─ 审查者会评估是否需要重构

5️⃣ 重构团队定期审查 → 提取通用逻辑到共享资源
   └─ 更新文档和代码
```

---

## 6. 最佳实践

### 6.1 事务管理

- **业务逻辑层（`logic`）** 负责开启和提交事务
- 所有数据库操作（`database`层函数）都必须在事务中执行
- 只读操作可以省略事务，直接从 `app_state.db_pool()` 获取连接

**✅ 推荐使用 TransactionHelper：**

```rust
use crate::features::shared::TransactionHelper;

let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
// ... 业务逻辑 ...
TransactionHelper::commit(tx).await?;
```

### 6.2 依赖注入

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

### 6.3 使用现有工具 - ⚠️ 重要

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

### 6.4 错误处理

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

### 6.5 幂等性

- 对于 `POST`（创建）和 `DELETE` 操作，应考虑幂等性
- 如果资源已存在或已删除，通常应返回成功状态码（`200 OK` 或 `204 No Content`），而不是错误

### 6.6 数据库 Schema - ⚠️ 关键

**在编写任何数据库查询之前，必须先查看数据库 Schema！禁止猜测表名或字段名！**

#### **查看 Schema 的位置**

```
src-tauri/migrations/20241001000000_initial_schema.sql
```

#### **常见错误示例**

```rust
// ❌ 错误：猜测表名
SELECT * FROM ordering WHERE ...  // 如果 schema 是复数，这会报错！

// ✅ 正确：查看 schema 后确认表名
SELECT * FROM orderings WHERE ...  // 数据库表名是 'orderings'
```

#### **开发流程**

```
1️⃣ 查看 migrations/xxx_initial_schema.sql
   └─ 确认表名、字段名、类型、约束

2️⃣ 编写 SQL 查询
   └─ 使用准确的表名和字段名

3️⃣ 编写 Rust 代码
   └─ 确保绑定参数与字段类型匹配
```

#### **关键表名清单**（供快速参考）

| 实体   | 表名（单数/复数） | 常见错误 |
| ------ | ----------------- | -------- |
| 任务   | `tasks`           | ✅ 复数  |
| 区域   | `areas`           | ✅ 复数  |
| 日程   | `task_schedules`  | ✅ 复数  |
| 时间块 | `time_blocks`     | ✅ 复数  |
| 模板   | `templates`       | ✅ 复数  |
| 排序   | `orderings`       | ✅ 复数  |
| 项目   | `projects`        | ✅ 复数  |

**注意：** 所有表名统一使用复数形式。

#### **实际案例**

```rust
// ❌ 错误：没有查看 schema，猜测表名
let query = "SELECT * FROM task_schedule WHERE ...";
// 运行时错误：no such table: task_schedule

// ✅ 正确：查看 migrations/xxx.sql，确认表名
let query = "SELECT * FROM task_schedules WHERE ...";
// 成功执行
```

### 6.7 数据真实性原则 - ⚠️ 关键

**后端返回的数据必须反映数据库的真实状态，不能依赖默认值或猜测！**

#### **错误模式：依赖 Assembler 的默认值**

```rust
// ❌ 错误：返回带默认值的数据
let task_card = TaskAssembler::task_to_card_basic(&task);
// task_card.schedule_status = ScheduleStatus::Staging (默认值)
return task_card;  // 返回了错误的状态！
```

**问题：**

- Assembler 的 `_basic` 方法使用默认值作为占位符
- 如果不查询实际状态就返回，前端会接收到错误数据
- 导致 UI 状态不一致

#### **正确模式：查询实际状态**

```rust
// ✅ 正确：查询实际状态并填充
let mut task_card = TaskAssembler::task_to_card_basic(&task);

// 查询实际的 schedule_status
let schedules = database::get_task_schedules(pool, task_id).await?;
task_card.schedule_status = if !schedules.is_empty() {
    ScheduleStatus::Scheduled
} else {
    ScheduleStatus::Staging
};

// 查询其他关联信息
task_card.sort_order = database::get_task_sort_order(pool, task_id).await?;
task_card.area = database::get_area_summary(pool, area_id).await?;

return task_card;  // 返回完整准确的数据 ✅
```

#### **避免冗余查询**

```rust
// ❌ 冗余：查询两次相同的表
let schedules = get_task_schedules(task_id).await?;
let has_schedule = has_any_schedule(task_id).await?;  // 冗余！

// ✅ 高效：复用已查询的数据
let schedules = get_task_schedules(task_id).await?;
let has_schedule = !schedules.is_empty();  // 直接判断
```

#### **实际案例：get_task 端点**

**错误版本（导致 UI bug）：**

```rust
let task_card = TaskAssembler::task_to_card_basic(&task);
// schedule_status = 'staging' (默认)
return TaskDetailDto { card: task_card };
// 前端：点击任务 → 任务跳到 Staging 列 ❌
```

**正确版本：**

```rust
let mut task_card = TaskAssembler::task_to_card_basic(&task);
let schedules = database::get_task_schedules(pool, task_id).await?;
task_card.schedule_status = if !schedules.is_empty() {
    Scheduled
} else {
    Staging
};
return TaskDetailDto { card: task_card };
// 前端：点击任务 → 任务保持在正确列 ✅
```

### 6.8 SSE 事件与 HTTP 响应数据一致性 - 🚨 关键警示

**问题：SSE 推送的数据与 HTTP 响应的数据不一致，导致前端状态混乱！**

#### **错误模式：在填充完整数据前写入 SSE**

```rust
// ❌ 错误：SSE 和 HTTP 返回的数据不一致
pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<Response> {
    let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

    // 1. 修改数据库
    database::update_something(&mut tx, task_id).await?;

    // 2. 组装基础数据（使用默认值）
    let mut task_card = TaskAssembler::task_to_card_basic(&task);
    // task_card.schedules = None (默认值，未填充)

    // 3. ❌ 在事务内写入 SSE（数据不完整！）
    let event = DomainEvent::new("task.updated", "task", task_id, json!({
        "task": task_card,  // schedules = None ❌
    }));
    outbox_repo.append_in_tx(&mut tx, &event).await?;

    // 4. 提交事务
    TransactionHelper::commit(tx).await?;

    // 5. ❌ 之后才填充完整数据
    task_card.schedules = TaskAssembler::assemble_schedules(pool, task_id).await?;

    // 6. 返回 HTTP（数据完整）
    Ok(Response { task: task_card })  // schedules = Some([...]) ✅
}
```

**问题：**

- SSE 推送：`task.schedules = None`（不完整）
- HTTP 返回：`task.schedules = Some([...])`（完整）
- 前端收到两份不同的数据，导致状态不一致！

#### **正确模式：先填充完整数据，再写入 SSE**

```rust
// ✅ 正确：确保 SSE 和 HTTP 数据完全一致
pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<Response> {
    // 1. 业务事务：只处理核心数据修改
    let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
    database::update_something(&mut tx, task_id).await?;
    let mut task_card = TaskAssembler::task_to_card_basic(&task);
    TransactionHelper::commit(tx).await?;

    // 2. ✅ 填充所有完整数据（事务已提交，可以查询）
    // ⚠️ 必须在写入 SSE 之前完成！
    task_card.schedules = TaskAssembler::assemble_schedules(pool, task_id).await?;
    task_card.area = get_area_summary(pool, area_id).await?;
    // ... 填充所有需要的关联数据

    // 3. ✅ 写入 SSE（在新事务中，数据完整）
    let mut outbox_tx = TransactionHelper::begin(app_state.db_pool()).await?;
    let event = DomainEvent::new("task.updated", "task", task_id, json!({
        "task": task_card,  // schedules = Some([...]) ✅
    }));
    outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
    TransactionHelper::commit(outbox_tx).await?;

    // 4. ✅ 返回 HTTP（与 SSE 数据一致）
    Ok(Response { task: task_card })
}
```

#### **数据流对比**

**❌ 错误流程：**

```
业务事务 → 组装基础数据 → SSE(不完整) → commit() → 填充完整数据 → HTTP(完整)
                                ↑ 不一致！
```

**✅ 正确流程：**

```
业务事务 → commit() → 填充完整数据 → SSE(完整) → HTTP(完整)
                                      ↑ 一致！✅
```

#### **实际案例：update_task 端点**

**错误版本（已修复）：**

```rust
// ❌ 旧代码：SSE 在填充 schedules 之前
task_card.schedule_status = determine_status(&mut tx, task_id).await?;

// SSE 写入（schedules = None）
outbox_repo.append_in_tx(&mut tx, &event).await?;
TransactionHelper::commit(tx).await?;

// 之后才填充 schedules
task_card.schedules = assemble_schedules(pool, task_id).await?;

// 结果：前端看板不显示新创建的任务！❌
```

**正确版本：**

```rust
// ✅ 新代码：先填充完整数据
task_card.schedule_status = determine_status(&mut tx, task_id).await?;
TransactionHelper::commit(tx).await?;

// ⚠️ 必须在 SSE 之前填充！
task_card.schedules = assemble_schedules(pool, task_id).await?;

// SSE 写入（schedules = Some([...])）
let mut outbox_tx = TransactionHelper::begin(pool).await?;
outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
TransactionHelper::commit(outbox_tx).await?;

// 结果：任务立即显示在日期看板！✅
```

#### **开发清单**

在编写包含 SSE 的端点时，务必检查：

- [ ] ✅ 业务事务提交后，是否填充了所有关联数据？
- [ ] ✅ SSE 事件载荷中的数据是否完整？
- [ ] ✅ SSE 推送的数据与 HTTP 响应是否一致？
- [ ] ✅ 是否使用了独立的 outbox 事务？
- [ ] ✅ `schedules` 字段是否已填充？
- [ ] ✅ `area` 字段是否已填充？
- [ ] ✅ 所有派生字段是否已正确计算？

#### **关键原则**

> **SSE 和 HTTP 必须返回完全相同的数据！**  
> **在写入 SSE 之前，确保所有数据已填充完整！**

---

### 6.9 代码审查清单

在提交代码前检查：

- [ ] **是否查看了共享资源清单（第 4 章）？**（新增！🔥）
- [ ] **是否使用了已有的共享 Repository/Assembler？**（新增！🔥）
- [ ] **是否遵守"禁止修改共享资源"原则？**（新增！🔥）
- [ ] **SSE 和 HTTP 返回的数据是否一致？**（新增！🚨）
- [ ] **是否在填充完整数据后才写入 SSE？**（新增！🚨）
- [ ] **是否查看了数据库 schema？**（最重要！）
- [ ] **返回的所有字段是否反映真实数据库状态？**
- [ ] 是否使用了正确的 trait 方法（`new_uuid()`, `now_utc()`）？
- [ ] 是否复用了 `shared/` 中的现有工具？
- [ ] 排序功能是否使用了 LexoRank 工具函数？
- [ ] 错误处理是否使用了 `?` 操作符？
- [ ] 是否在事务中执行了所有写操作？
- [ ] 是否使用了 `TransactionHelper`？
- [ ] SQL 查询的表名和字段名是否与 schema 完全一致？
- [ ] **是否有冗余查询可以优化？**

通过遵循此规范，我们可以构建一个既灵活又有序、易于理解和扩展的后端系统。

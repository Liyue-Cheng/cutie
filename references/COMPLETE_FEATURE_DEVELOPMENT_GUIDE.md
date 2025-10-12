# Cutie 完整功能开发手册

> 从零到完成一个新功能的完整指南，整合所有开发规范、公共资源列表和经验教训

**版本**: 2.0
**最后更新**: 2025-10-12

---

## 📋 目录

1. [开发前必读](#开发前必读)
2. [后端架构概览](#后端架构概览)
3. [后端开发完整流程](#后端开发完整流程)
4. [前端开发完整流程](#前端开发完整流程)
5. [公共资源完整清单](#公共资源完整清单)
6. [数据结构修改影响分析](#数据结构修改影响分析)
7. [关键经验教训](#关键经验教训)
8. [开发检查清单](#开发检查清单)
9. [常见问题与调试](#常见问题与调试)

---

## 开发前必读

### 核心原则

1. **文档驱动开发**: 代码实现必须与 CABC 文档完全一致
2. **数据真实性**: 后端返回的数据必须反映数据库真实状态,不能依赖默认值
3. **SSE 一致性**: SSE 事件和 HTTP 响应必须返回完全相同的数据
4. **Schema 优先**: 编写任何 SQL 前必须先查看数据库 Schema
5. **复用优先**: 使用共享资源,禁止重复实现已有功能
6. **分层清晰**: 理解 `infra/` (基础设施) 和 `features/shared/` (业务共享) 的区别

### 必须查看的文档

开发新功能前,按顺序阅读:

1. **Schema 定义**: `src-tauri/migrations/20241001000000_initial_schema.sql`
2. **后端架构**: 本文档 [后端架构概览](#后端架构概览)
3. **共享资源清单**: 本文档 [公共资源完整清单](#公共资源完整清单)
4. **业务逻辑规范**: `notes/业务逻辑.md`
5. **SFC 开发规范**: `references/SFC_SPEC.md`
6. **数据结构耦合**: `references/DATA_SCHEMA_COUPLING.md`
7. **开发经验教训**: `ai-doc/LESSONS_LEARNED.md`

---

## 后端架构概览

### 架构分层

Cutie 后端采用**清晰的分层架构**，将技术基础设施与业务逻辑分离：

```
src-tauri/src/
├── infra/                    ← 基础设施层 (Infrastructure Layer)
│   ├── core/                 - 错误处理、工具函数、构建信息
│   ├── database/             - 数据库连接和事务管理
│   ├── http/                 - HTTP 基础设施 (中间件、响应、错误处理)
│   ├── events/               - 事件系统 (SSE、事件分发、Outbox)
│   ├── logging/              - 统一日志系统
│   └── ports/                - 外部依赖抽象 (时钟、ID生成器)
│
├── features/                 ← 业务逻辑层 (Business Logic Layer)
│   ├── shared/               ← 业务共享层 (跨功能共享业务逻辑)
│   │   ├── repositories/     - 数据访问层 (Repository traits + 实现)
│   │   ├── assemblers/       - 数据组装层 (DTO assemblers)
│   │   ├── services/         - 业务服务层 (跨功能业务逻辑)
│   │   └── validators/       - 验证器层 (业务规则验证)
│   │
│   ├── endpoints/            ← HTTP 端点层 (所有 API handlers)
│   │   ├── area/             - Area 相关端点
│   │   ├── tasks/            - Task 相关端点
│   │   ├── time_blocks/      - TimeBlock 相关端点
│   │   └── ...
│   │
│   ├── areas.rs              ← 功能模块入口 (路由定义)
│   ├── tasks.rs
│   ├── time_blocks.rs
│   └── ...
│
├── entities/                 ← 领域模型层 (Domain Entities & DTOs)
│   ├── task/
│   ├── time_block/
│   └── ...
│
├── config/                   ← 配置层
├── startup/                  ← 应用启动层
└── lib.rs                    ← 库根文件
```

### 模块职责

#### 1. `infra/` - 基础设施层

**定位**: 技术关注点（如何实现）

**职责**:

- 提供与业务无关的技术性基础组件
- 不包含任何业务规则，只负责技术实现细节
- 位于分层架构的最底层

**关键模块**:

- `core`: 错误类型(`AppError`)、工具函数、构建信息
- `database`: 数据库连接池、事务管理
- `http`: HTTP 响应构建、错误处理、中间件
- `events`: SSE 基础设施、事件分发、Outbox
- `logging`: 分层日志系统
- `ports`: 依赖注入抽象 (`Clock`, `IdGenerator`)

**导入示例**:

```rust
use crate::infra::{
    core::{AppError, AppResult},
    http::success_response,
    ports::{Clock, IdGenerator},
};
```

#### 2. `features/shared/` - 业务共享层

**定位**: 业务关注点（做什么）

**职责**:

- 提供跨功能模块的业务逻辑复用
- 包含业务语义的数据访问、组装、服务和验证
- 位于业务逻辑层，服务于各个功能模块

**分层架构**:

```
features/shared/
├── repositories/        ← 数据访问层
│   ├── traits.rs        - Repository 抽象接口
│   ├── transaction.rs   - 事务辅助工具
│   ├── task_repository.rs
│   ├── time_block_repository.rs
│   └── ...
│
├── assemblers/          ← 数据组装层
│   ├── task_assembler.rs
│   ├── time_block_assembler.rs
│   └── ...
│
├── services/            ← 业务服务层
│   ├── ai_classification_service.rs
│   ├── conflict_checker.rs
│   └── ...
│
└── validators/          ← 验证器层
    ├── task_validator.rs
    ├── time_block_validator.rs
    └── ...
```

**导入示例**:

```rust
// 方式一：顶层导出（推荐用于简单场景）
use crate::features::shared::{
    TaskRepository,
    TaskAssembler,
    TaskValidator,
    TransactionHelper,
};

// 方式二：带命名空间（推荐用于复杂场景）
use crate::features::shared::{
    repositories::{TaskRepository, AreaRepository},
    assemblers::TaskAssembler,
    validators::TaskValidator,
    services::AiClassificationService,
};
```

#### 3. `features/endpoints/` - HTTP 端点层

**定位**: API 处理层

**职责**:

- 处理 HTTP 请求和响应
- 调用业务逻辑层完成功能
- 采用 SFC (Single File Component) 模式组织代码

**结构**:

```
features/endpoints/
├── area/
│   ├── mod.rs            - 导出所有 handlers
│   ├── create_area.rs    - POST /api/areas
│   ├── update_area.rs    - PATCH /api/areas/:id
│   └── ...
├── tasks/
│   ├── mod.rs
│   ├── create_task.rs
│   └── ...
└── ...
```

每个功能模块（如 `areas.rs`, `tasks.rs`）负责定义路由：

```rust
// src/features/tasks.rs
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::tasks::list_tasks))
        .route("/", post(endpoints::tasks::create_task))
        // ...
}
```

### 关键区别: `infra/` vs `features/shared/`

| 维度         | `infra/`                                | `features/shared/`                                 |
| ------------ | --------------------------------------- | -------------------------------------------------- |
| **关注点**   | 技术实现（How）                         | 业务逻辑（What）                                   |
| **职责**     | HTTP、数据库、日志、事件                | Repositories、Assemblers、Services、Validators     |
| **业务语义** | 无业务语义                              | 包含业务语义                                       |
| **依赖方向** | 被所有层依赖                            | 被端点层依赖                                       |
| **示例**     | `AppError`, `success_response`, `Clock` | `TaskRepository`, `TaskValidator`, `TaskAssembler` |

### 导入路径规范

**正确示例**:

```rust
// ✅ 基础设施导入
use crate::infra::core::{AppError, AppResult};
use crate::infra::http::success_response;
use crate::infra::ports::Clock;

// ✅ 业务共享导入
use crate::features::shared::{
    TaskRepository,
    TaskValidator,
    TransactionHelper,
};

// ✅ 实体导入
use crate::entities::task::{Task, CreateTaskRequest, TaskCardDto};
```

**错误示例**:

```rust
// ❌ 错误: shared 已重命名为 infra
use crate::shared::core::AppError;

// ❌ 错误: 混淆业务层和基础设施层
use crate::features::shared::AppError;  // AppError 在 infra 中
use crate::infra::TaskRepository;       // TaskRepository 在 features/shared 中
```

---

## 后端开发完整流程

### Step 1: 设计阶段

#### 1.1 查看数据库 Schema

**⚠️ 最重要的第一步!**

```bash
# 查看 Schema
cat src-tauri/migrations/20241001000000_initial_schema.sql

# 确认:
# - 表名 (所有表名都是复数: tasks, areas, time_blocks, orderings 等)
# - 字段名和类型
# - 约束条件
# - 索引设计
```

**常见错误**:

```rust
// ❌ 错误: 猜测表名
SELECT * FROM ordering WHERE ...

// ✅ 正确: 查看 schema 确认
SELECT * FROM orderings WHERE ...  // 表名是 orderings
```

#### 1.2 检查共享资源清单

**必须先查看** [公共资源完整清单](#公共资源完整清单),避免重复实现!

检查项:

- [ ] 需要的 Repository 是否已存在?
- [ ] 需要的 Assembler 是否已存在?
- [ ] 需要的 Validator 是否已存在?
- [ ] 需要的 Service 是否已存在?
- [ ] 需要的工具函数是否已存在?

如果存在,直接使用;如果不存在,在 SFC 的 `database` 模块中实现。

**⚠️ 禁止修改共享资源!** 在开发新功能时,不要修改 `features/shared` 中的代码。如果需要新增共享功能，应该单独规划并与团队讨论。

#### 1.3 参考类似功能

根据复杂度选择参考:

- 简单 CRUD → 参考 `features/endpoints/area/create_area.rs`
- 复杂业务逻辑 → 参考 `features/endpoints/tasks/complete_task.rs`
- 跨实体操作 → 参考 `features/endpoints/time_blocks/create_from_task.rs`
- 使用验证器 → 参考 `features/endpoints/tasks/create_task.rs`

---

### Step 2: 创建实体层

#### 2.1 创建实体 Model

**文件**: `src-tauri/src/entities/xxx/model.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 领域实体 (业务层使用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

/// 数据库行映射 (sqlx 使用)
#[derive(Debug, FromRow)]
pub struct EntityRow {
    pub id: String,          // ⚠️ 数据库中是 TEXT
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

/// 从数据库行转换为领域实体
impl TryFrom<EntityRow> for Entity {
    type Error = uuid::Error;

    fn try_from(row: EntityRow) -> Result<Self, Self::Error> {
        Ok(Entity {
            id: Uuid::parse_str(&row.id)?,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        })
    }
}
```

#### 2.2 创建 Request DTOs

**文件**: `src-tauri/src/entities/xxx/request_dtos.rs`

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateEntityRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateEntityRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}
```

#### 2.3 创建 Response DTOs

**文件**: `src-tauri/src/entities/xxx/response_dtos.rs`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct EntityDto {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### 2.4 导出模块

**文件**: `src-tauri/src/entities/xxx/mod.rs`

```rust
pub mod model;
pub mod request_dtos;
pub mod response_dtos;

pub use model::*;
pub use request_dtos::*;
pub use response_dtos::*;
```

**文件**: `src-tauri/src/entities/mod.rs`

```rust
pub mod xxx;  // ← 添加
```

---

### Step 3: 创建端点 (SFC 模式)

#### 3.1 SFC 文件结构

**文件**: `src-tauri/src/features/endpoints/xxx/create_xxx.rs`

```rust
/// 创建 XXX - 单文件组件
///
/// ⚠️ 开发前必读:
/// 1. 查看 Schema: migrations/xxx.sql
/// 2. 查看共享资源清单: COMPLETE_FEATURE_DEVELOPMENT_GUIDE.md
/// 3. 使用已有的 Repository/Assembler/Validator,禁止重复实现

// ==================== CABC 文档 ====================
/*
CABC for `create_xxx`

## 1. 端点签名
POST /api/xxx

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要创建一个新的XXX,以便...

### 2.2 核心业务逻辑
[详细描述业务逻辑]

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "name": "string (required)"
}

### 3.2 响应 (Responses)
**201 Created:**
{
  "id": "uuid",
  "name": "string",
  ...
}

## 4. 验证规则
- name: 必须,非空,长度 <= 255

## 5. 业务逻辑详解
1. 验证输入
2. 开启事务
3. 生成 UUID 和时间戳
4. 插入数据库
5. 提交事务
6. 返回结果

## 6. 边界情况
- name 为空: 返回 422
- name 重复: 返回 409 (如果有唯一约束)

## 7. 预期副作用
### 数据库操作:
- INSERT: 1条记录到 xxx 表
- 事务边界: begin() → commit()

### SSE 事件:
- xxx.created

## 8. 契约
### 前置条件:
- request.name 不为空

### 后置条件:
- 数据库中存在新记录
- 返回完整的 EntityDto

### 不变量:
- id 和 created_at 一旦创建永不改变
*/

// ==================== 依赖引入 ====================
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::xxx::{Entity, EntityDto, CreateEntityRequest},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult},
        http::created_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateEntityRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(dto) => created_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
// ⚠️ 推荐：如果验证逻辑会被多个端点复用，应该创建共享 Validator
// 参考 features/shared/validators/task_validator.rs
//
// 如果只在当前端点使用，可以保留在 validation 模块中
mod validation {
    use super::*;
    use crate::infra::core::ValidationError;

    pub fn validate_request(request: &CreateEntityRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 验证 name
        if request.name.trim().is_empty() {
            errors.push(ValidationError::new("name", "名称不能为空", "REQUIRED"));
        }

        if request.name.len() > 255 {
            errors.push(ValidationError::new("name", "名称长度不能超过255个字符", "MAX_LENGTH"));
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors));
        }

        Ok(())
    }
}

// 🔍 如果验证逻辑需要复用，使用共享 Validator：
// use crate::features::shared::XxxValidator;
// XxxValidator::validate_create_request(&request)?;

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateEntityRequest,
    ) -> AppResult<EntityDto> {
        // 1. 验证
        validation::validate_request(&request)?;

        // 2. 获取依赖
        let id = app_state.id_generator().new_uuid();  // ✅ 正确方法名
        let now = app_state.clock().now_utc();         // ✅ 正确方法名

        // 3. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 4. 创建实体
        let entity = Entity {
            id,
            name: request.name,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        // 5. 插入数据库
        database::insert_in_tx(&mut tx, &entity).await?;

        // 6. 提交事务
        TransactionHelper::commit(tx).await?;

        // 7. 组装 DTO
        let dto = EntityDto {
            id: entity.id,
            name: entity.name,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        };

        // 8. (可选) 发送 SSE 事件
        // let mut outbox_tx = TransactionHelper::begin(app_state.db_pool()).await?;
        // ... emit event ...
        // TransactionHelper::commit(outbox_tx).await?;

        Ok(dto)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::Transaction;
    use sqlx::Sqlite;

    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        entity: &Entity,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO xxx_table (id, name, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(entity.id.to_string())
            .bind(&entity.name)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .bind(entity.is_deleted)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
```

#### 3.2 关键检查项

**依赖注入 ✅**:

```rust
// ✅ 正确
let id = app_state.id_generator().new_uuid();
let now = app_state.clock().now_utc();
let pool = app_state.db_pool();

// ❌ 错误 (方法名不存在)
let id = app_state.id_generator().generate();
let now = app_state.clock().now();
```

**事务管理 ✅**:

```rust
// ✅ 使用 TransactionHelper
use crate::features::shared::TransactionHelper;

let mut tx = TransactionHelper::begin(pool).await?;
// ... 业务逻辑 ...
TransactionHelper::commit(tx).await?;
```

**使用共享资源 ✅**:

```rust
// ✅ 正确: 使用已有的 Repository
use crate::features::shared::TaskRepository;

let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id).await?;

// ✅ 正确: 使用已有的 Validator
use crate::features::shared::TaskValidator;

TaskValidator::validate_create_request(&request)?;

// ✅ 正确: 使用已有的 Assembler
use crate::features::shared::TaskAssembler;

let task_card = TaskAssembler::task_to_card_full(&task, schedule_status, area, schedule_info);

// ❌ 错误: 重复实现查询
mod database {
    pub async fn find_task(...) { ... }  // TaskRepository 已经提供了!
}

// ❌ 错误: 重复实现验证
mod validation {
    pub fn validate_task(...) { ... }  // TaskValidator 已经提供了!
}
```

---

### Step 4: SSE 事件处理 (如果需要)

#### 4.1 SSE 数据一致性原则 🚨

**关键原则**: SSE 事件和 HTTP 响应必须返回完全相同的数据!

**❌ 错误模式**: 在填充完整数据前发送 SSE

```rust
// ❌ 错误
let mut tx = TransactionHelper::begin(pool).await?;
database::update_something(&mut tx, task_id).await?;

// 组装基础数据 (使用默认值)
let mut task_card = TaskAssembler::task_to_card_basic(&task);
// task_card.schedules = None (默认值,未填充)

// ❌ 在事务内写入 SSE (数据不完整!)
let event = DomainEvent::new("task.updated", "task", task_id, json!({
    "task": task_card,  // schedules = None ❌
}));
outbox_repo.append_in_tx(&mut tx, &event).await?;

TransactionHelper::commit(tx).await?;

// 之后才填充完整数据
task_card.schedules = assemble_schedules(pool, task_id).await?;

// 返回 HTTP (数据完整)
Ok(Response { task: task_card })  // schedules = Some([...]) ✅
```

**✅ 正确模式**: 先填充完整数据,再发送 SSE

```rust
// ✅ 正确
// 1. 业务事务: 只处理核心数据修改
let mut tx = TransactionHelper::begin(pool).await?;
database::update_something(&mut tx, task_id).await?;
let mut task_card = TaskAssembler::task_to_card_basic(&task);
TransactionHelper::commit(tx).await?;

// 2. ⚠️ 填充所有完整数据 (在 SSE 之前!)
task_card.schedules = assemble_schedules(pool, task_id).await?;
task_card.area = get_area_summary(pool, area_id).await?;
// ... 填充所有需要的关联数据

// 3. ✅ 写入 SSE (在新事务中,数据完整)
let mut outbox_tx = TransactionHelper::begin(pool).await?;
let event = DomainEvent::new("task.updated", "task", task_id, json!({
    "task": task_card,  // schedules = Some([...]) ✅
}));
outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
TransactionHelper::commit(outbox_tx).await?;

// 4. ✅ 返回 HTTP (与 SSE 数据一致)
Ok(Response { task: task_card })
```

#### 4.2 SSE 开发检查清单

- [ ] ✅ 业务事务提交后,是否填充了所有关联数据?
- [ ] ✅ SSE 事件载荷中的数据是否完整?
- [ ] ✅ SSE 推送的数据与 HTTP 响应是否一致?
- [ ] ✅ 是否使用了独立的 outbox 事务?
- [ ] ✅ `schedules` 字段是否已填充?
- [ ] ✅ `area` 字段是否已填充?

---

### Step 5: 注册路由

#### 5.1 端点模块导出

**文件**: `src-tauri/src/features/endpoints/xxx/mod.rs`

```rust
/// XXX endpoints
/// XXX 相关的 HTTP 端点

pub use create_xxx::handle as create_xxx;
pub use list_xxx::handle as list_xxx;
pub use update_xxx::handle as update_xxx;
pub use delete_xxx::handle as delete_xxx;

mod create_xxx;
mod list_xxx;
mod update_xxx;
mod delete_xxx;
```

#### 5.2 Feature 路由

**文件**: `src-tauri/src/features/xxx.rs`

```rust
/// XXX 功能模块
use axum::{
    routing::{get, post, patch, delete},
    Router,
};

use crate::startup::AppState;
use crate::features::endpoints::xxx as endpoints;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_xxx))
        .route("/", post(endpoints::create_xxx))
        .route("/:id", get(endpoints::get_xxx))
        .route("/:id", patch(endpoints::update_xxx))
        .route("/:id", delete(endpoints::delete_xxx))
}
```

#### 5.3 全局端点模块

**文件**: `src-tauri/src/features/endpoints/mod.rs`

```rust
/// 所有 HTTP 端点
pub mod area;
pub mod xxx;  // ← 添加
// ... 其他端点模块
```

#### 5.4 全局路由

**文件**: `src-tauri/src/features/mod.rs`

```rust
pub mod areas;
pub mod xxx;  // ← 添加功能模块

pub mod endpoints;  // 端点模块声明

pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .nest("/areas", areas::create_routes())
        .nest("/xxx", xxx::create_routes())  // ← 添加路由
        // ... 其他路由
}
```

---

### Step 6: 编写 API 文档

**文件**: `src-tauri/src/features/xxx/API_SPEC.md`

参考其他功能的 API_SPEC.md,包含:

- 功能概述
- 端点清单
- 每个端点的完整 CABC 文档

**注意**: CABC 文档应该先写在端点文件的注释中,然后可以使用 `doc-composer` 工具自动生成 API 文档。

---

## 前端开发完整流程

### Step 1: 创建 DTO 类型

**文件**: `src/types/dtos.ts`

```typescript
export interface Entity {
  id: string
  name: string
  created_at: string
  updated_at: string
}
```

---

### Step 2: 创建 Pinia Store

#### 2.1 Store 模块化结构

**推荐模式** (参考 `stores/task/`):

```
stores/xxx/
├── index.ts           # Store 组合
├── core.ts            # State & Getters
├── crud-operations.ts # Create/Update/Delete
├── view-operations.ts # Fetch/Query
└── event-handlers.ts  # SSE 订阅
```

#### 2.2 Core (State & Getters)

**文件**: `src/stores/xxx/core.ts`

```typescript
import { ref, computed } from 'vue'

// ==================== State ====================
export const entities = ref(new Map<string, Entity>())

// ==================== Getters ====================
export const allEntities = computed(() => Array.from(entities.value.values()))

export const getEntityById = computed(() => (id: string) => entities.value.get(id))

// ==================== Mutations ====================
export function addOrUpdateEntity(entity: Entity) {
  const newMap = new Map(entities.value)
  newMap.set(entity.id, entity)
  entities.value = newMap // ✅ 创建新对象触发响应式
}

export function removeEntity(id: string) {
  const newMap = new Map(entities.value)
  newMap.delete(id)
  entities.value = newMap
}

export function clearAll() {
  entities.value = new Map()
}
```

#### 2.3 CRUD Operations

**文件**: `src/stores/xxx/crud-operations.ts`

**⚠️ 重要：后端响应数据格式**

后端所有成功响应都使用 `ApiResponse` 包装：

```typescript
interface ApiResponse<T> {
  data: T // 实际数据
  timestamp: string // 响应时间戳
  request_id: string | null
}
```

**前端必须从 `response.data` 中提取实际数据！**

```typescript
import { apiBaseUrl } from '@/composables/useApiConfig'
import { addOrUpdateEntity, removeEntity } from './core'

export async function createEntity(payload: CreateEntityPayload): Promise<Entity> {
  const response = await fetch(`${apiBaseUrl.value}/xxx`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    throw new Error('Failed to create entity')
  }

  // ✅ 正确：提取 .data 字段
  const responseData = await response.json()
  const entity: Entity = responseData.data
  addOrUpdateEntity(entity)
  return entity
}

export async function updateEntity(id: string, payload: UpdateEntityPayload): Promise<Entity> {
  const response = await fetch(`${apiBaseUrl.value}/xxx/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    throw new Error('Failed to update entity')
  }

  // ✅ 正确：提取 .data 字段
  const responseData = await response.json()
  const entity: Entity = responseData.data
  addOrUpdateEntity(entity)
  return entity
}

export async function deleteEntity(id: string): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/xxx/${id}`, {
    method: 'DELETE',
  })

  if (!response.ok) {
    throw new Error('Failed to delete entity')
  }

  removeEntity(id)
}
```

#### 2.4 View Operations

**文件**: `src/stores/xxx/view-operations.ts`

```typescript
import { apiBaseUrl } from '@/composables/useApiConfig'
import { addOrUpdateEntity, clearAll } from './core'

export async function fetchAllEntities(): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/xxx`)

  if (!response.ok) {
    throw new Error('Failed to fetch entities')
  }

  const entities: Entity[] = await response.json() // ⚠️ 直接解析数组

  clearAll()
  entities.forEach(addOrUpdateEntity)
}
```

#### 2.5 SSE Event Handlers

**文件**: `src/stores/xxx/event-handlers.ts`

```typescript
import { getEventSubscriber } from '@/services/events'
import { addOrUpdateEntity, removeEntity } from './core'

export function initEventSubscriptions() {
  const subscriber = getEventSubscriber()
  if (!subscriber) return

  subscriber.on('xxx.created', handleEntityCreatedEvent)
  subscriber.on('xxx.updated', handleEntityUpdatedEvent)
  subscriber.on('xxx.deleted', handleEntityDeletedEvent)
}

function handleEntityCreatedEvent(event: any) {
  const entity = event.payload?.entity
  if (entity) {
    addOrUpdateEntity(entity)
  }
}

function handleEntityUpdatedEvent(event: any) {
  const entity = event.payload?.entity
  if (entity) {
    addOrUpdateEntity(entity)
  }
}

function handleEntityDeletedEvent(event: any) {
  const entityId = event.payload?.entity_id
  if (entityId) {
    removeEntity(entityId)
  }
}
```

#### 2.6 Store 组合

**文件**: `src/stores/xxx/index.ts`

```typescript
import { defineStore } from 'pinia'
import * as core from './core'
import * as crud from './crud-operations'
import * as view from './view-operations'
import * as events from './event-handlers'

export const useEntityStore = defineStore('entity', () => {
  return {
    // State & Getters
    entities: core.entities,
    allEntities: core.allEntities,
    getEntityById: core.getEntityById,

    // CRUD Actions
    createEntity: crud.createEntity,
    updateEntity: crud.updateEntity,
    deleteEntity: crud.deleteEntity,

    // View Actions
    fetchAllEntities: view.fetchAllEntities,

    // SSE
    initEventSubscriptions: events.initEventSubscriptions,
  }
})
```

#### 2.7 初始化 SSE 订阅

**文件**: `src/composables/useApiConfig.ts`

```typescript
// 在 API 准备就绪后初始化所有 SSE 订阅
import { useEntityStore } from '@/stores/xxx'

const entityStore = useEntityStore()
entityStore.initEventSubscriptions() // ← 添加
```

---

### Step 3: 注册 SSE 事件监听器

**文件**: `src/services/events.ts`

```typescript
// 在 EventSource 中添加新的事件监听器
this.eventSource.addEventListener('xxx.created', (e: MessageEvent) => {
  this.handleEvent('xxx.created', e.data)
})

this.eventSource.addEventListener('xxx.updated', (e: MessageEvent) => {
  this.handleEvent('xxx.updated', e.data)
})

this.eventSource.addEventListener('xxx.deleted', (e: MessageEvent) => {
  this.handleEvent('xxx.deleted', e.data)
})
```

---

### Step 4: 创建 UI 组件

#### 4.1 列表/管理组件

**文件**: `src/components/parts/EntityManager.vue`

```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { useEntityStore } from '@/stores/xxx'

const entityStore = useEntityStore()

onMounted(async () => {
  await entityStore.fetchAllEntities()
})

async function handleCreate(name: string) {
  await entityStore.createEntity({ name })
}

async function handleUpdate(id: string, name: string) {
  await entityStore.updateEntity(id, { name })
}

async function handleDelete(id: string) {
  await entityStore.deleteEntity(id)
}
</script>

<template>
  <div>
    <ul>
      <li v-for="entity in entityStore.allEntities" :key="entity.id">
        {{ entity.name }}
        <button @click="handleDelete(entity.id)">Delete</button>
      </li>
    </ul>
  </div>
</template>
```

---

### Step 5: 添加路由

**文件**: `src/router/index.ts`

```typescript
{
  path: '/xxx',
  name: 'xxx',
  component: () => import('../views/XxxView.vue'),
}
```

---

## 公共资源完整清单

### 后端共享资源概览

后端共享资源分为两大类：

1. **基础设施层** (`infra/`): 技术性基础组件
2. **业务共享层** (`features/shared/`): 业务逻辑复用

---

### 1. 基础设施层资源 (`infra/`)

#### 📌 核心错误和结果类型 (`infra/core/error.rs`)

```rust
use crate::infra::core::{AppError, AppResult, DbError, ValidationError};

// AppError - 应用错误枚举
pub enum AppError {
    DatabaseError(DbError),
    ValidationFailed(Vec<ValidationError>),
    NotFound(String),
    Conflict(String),
    // ...
}

// AppResult - 应用结果类型别名
pub type AppResult<T> = Result<T, AppError>;

// ValidationError - 验证错误结构
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

// 便捷方法
impl AppError {
    pub fn validation_error(field: &str, message: &str, code: &str) -> Self;
}
```

#### 📌 HTTP 响应构建 (`infra/http/responses.rs`)

```rust
use crate::infra::http::{success_response, created_response};

// 200 OK 响应
pub fn success_response<T: Serialize>(data: T) -> impl IntoResponse

// 201 Created 响应
pub fn created_response<T: Serialize>(data: T) -> impl IntoResponse

// ApiResponse 包装结构
pub struct ApiResponse<T> {
    pub data: T,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
}
```

#### 📌 依赖注入抽象 (`infra/ports/`)

```rust
use crate::infra::ports::{Clock, IdGenerator, SystemClock, UuidV4Generator};

// Clock trait - 时钟抽象
pub trait Clock {
    fn now_utc(&self) -> DateTime<Utc>;
}

// IdGenerator trait - ID 生成器抽象
pub trait IdGenerator {
    fn new_uuid(&self) -> Uuid;
}

// 从 AppState 获取
let id = app_state.id_generator().new_uuid();
let now = app_state.clock().now_utc();
```

#### 📌 工具函数 (`infra/core/utils/`)

```rust
// 排序算法 (LexoRank)
use crate::infra::core::utils::{
    generate_initial_sort_order,
    get_rank_after,
    get_rank_before,
    get_mid_lexo_rank,
};

// 时间工具
use crate::infra::core::utils::time_utils;
```

---

### 2. 业务共享层资源 (`features/shared/`)

#### 📦 Repositories (`features/shared/repositories/`)

**Repository Traits** (`traits.rs`)

```rust
use crate::features::shared::{Repository, QueryableRepository, BatchRepository};

// 基础 CRUD trait
#[async_trait]
pub trait Repository<Entity, ID = Uuid> {
    async fn find_by_id_in_tx(tx: &mut Transaction<'_, Sqlite>, id: ID) -> AppResult<Option<Entity>>;
    async fn find_by_id(pool: &SqlitePool, id: ID) -> AppResult<Option<Entity>>;
    async fn insert_in_tx(tx: &mut Transaction<'_, Sqlite>, entity: &Entity) -> AppResult<()>;
    async fn update_in_tx(tx: &mut Transaction<'_, Sqlite>, entity: &Entity) -> AppResult<()>;
    async fn soft_delete_in_tx(tx: &mut Transaction<'_, Sqlite>, id: ID) -> AppResult<()>;
    async fn hard_delete_in_tx(tx: &mut Transaction<'_, Sqlite>, id: ID) -> AppResult<()>;
}
```

**Transaction Helper** (`transaction.rs`)

```rust
use crate::features::shared::TransactionHelper;

// 开始事务 (统一错误处理)
pub async fn begin(pool: &SqlitePool) -> AppResult<Transaction<'_, Sqlite>>

// 提交事务 (统一错误处理)
pub async fn commit(tx: Transaction<'_, Sqlite>) -> AppResult<()>
```

**具体 Repository 实现**:

- `AreaRepository` - Area 数据访问
- `TaskRepository` - Task 数据访问
- `TaskScheduleRepository` - TaskSchedule 数据访问
- `TaskRecurrenceRepository` - TaskRecurrence 数据访问
- `TaskRecurrenceLinkRepository` - 循环任务关联
- `TaskTimeBlockLinkRepository` - 任务-时间块关联
- `TimeBlockRepository` - TimeBlock 数据访问

**使用示例**:

```rust
use crate::features::shared::{TaskRepository, TransactionHelper};

let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id).await?;
TransactionHelper::commit(tx).await?;
```

#### 🏗️ Assemblers (`features/shared/assemblers/`)

**Assemblers** 负责将数据库记录组装成 DTO

**可用的 Assemblers**:

- `TaskAssembler` - 组装 TaskCardDto、TaskDetailDto
- `LinkedTaskAssembler` - 组装 LinkedTaskSummary（任务摘要）
- `TimeBlockAssembler` - 组装 TimeBlockViewDto
- `ViewTaskCardAssembler` - 批量组装 TaskCard（包括 area、schedule_status）

**使用示例**:

```rust
use crate::features::shared::{TaskAssembler, TimeBlockAssembler};

// 创建基础 TaskCard
let task_card = TaskAssembler::task_to_card_basic(&task);

// 创建完整 TaskCard
let task_card = TaskAssembler::task_to_card_full(&task, schedule_status, area, schedule_info);

// 组装 TimeBlock视图
let time_block_view = TimeBlockAssembler::assemble_view(&time_block, pool).await?;
```

#### ✅ Validators (`features/shared/validators/`)

**Validators** 负责数据验证逻辑

**可用的 Validators**:

- `TaskValidator` - Task 创建/更新请求验证
- `TimeBlockValidator` - TimeBlock 创建/更新请求验证

**使用示例**:

```rust
use crate::features::shared::{TaskValidator, TimeBlockValidator};

// 验证创建任务请求
TaskValidator::validate_create_request(&request)?;

// 验证更新任务请求
TaskValidator::validate_update_request(&request)?;

// 验证时间块请求
TimeBlockValidator::validate_create_request(&request)?;
```

#### 🔧 Services (`features/shared/services/`)

**Services** 提供跨功能的业务逻辑

**可用的 Services**:

- `AiClassificationService` - AI 分类服务
- `RecurrenceInstantiationService` - 循环任务实例化服务
- `TimeBlockConflictChecker` - 时间块冲突检测

**使用示例**:

```rust
use crate::features::shared::TimeBlockConflictChecker;

// 检查时间冲突
TimeBlockConflictChecker::check_in_tx(
    &mut tx,
    start_time,
    end_time,
    Some(exclude_id),
).await?;
```

---

#### 详细 API 参考

以下是主要 Repositories 的详细 API：

##### `TaskRepository` (`features/shared/repositories/task_repository.rs`)

```rust
// 在事务中查询任务
pub async fn find_by_id_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
) -> AppResult<Option<Task>>

// 非事务查询任务
pub async fn find_by_id(
    pool: &SqlitePool,
    task_id: Uuid,
) -> AppResult<Option<Task>>

// 插入任务
pub async fn insert_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task: &Task,
) -> AppResult<()>

// 更新任务
pub async fn update_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
    request: &UpdateTaskRequest,
) -> AppResult<Task>

// 软删除任务
pub async fn soft_delete_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
) -> AppResult<()>

// 设置任务为已完成
pub async fn set_completed_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
    completed_at: DateTime<Utc>,
) -> AppResult<()>

// 重新打开任务
pub async fn set_reopened_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
    updated_at: DateTime<Utc>,
) -> AppResult<()>
```

**`TaskScheduleRepository`** (`repositories/task_schedule_repository.rs`)

```rust
// 检查任务是否有日程
pub async fn has_any_schedule(
    executor: impl sqlx::Executor<'_, Database = Sqlite>,
    task_id: Uuid,
) -> AppResult<bool>

// 检查某天是否有日程
pub async fn has_schedule_for_day_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
    scheduled_day: NaiveDate,
) -> AppResult<bool>

// 创建日程记录
pub async fn create_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
    scheduled_day: NaiveDate,
) -> AppResult<()>

// 更新当天日程为已完成
pub async fn update_today_to_completed_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
    now: DateTime<Utc>,
) -> AppResult<()>

// 删除未来日程
pub async fn delete_future_schedules_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
    now: DateTime<Utc>,
) -> AppResult<()>

// 删除任务的所有日程
pub async fn delete_all_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
) -> AppResult<()>

// 获取任务的所有日程记录
pub async fn get_all_for_task(
    pool: &SqlitePool,
    task_id: Uuid,
) -> AppResult<Vec<TaskSchedule>>
```

**`TaskTimeBlockLinkRepository`** (`repositories/task_time_block_link_repository.rs`)

```rust
// 创建任务到时间块的链接
pub async fn link_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
    block_id: Uuid,
) -> AppResult<()>

// 删除任务的所有链接
pub async fn delete_all_for_task_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
) -> AppResult<()>

// 删除时间块的所有链接
pub async fn delete_all_for_block_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: Uuid,
) -> AppResult<()>

// 查询任务链接的所有时间块
pub async fn find_linked_time_blocks_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: Uuid,
) -> AppResult<Vec<TimeBlock>>

// 检查时间块是否独占链接某任务
pub async fn is_exclusive_link_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: Uuid,
    task_id: Uuid,
) -> AppResult<bool>

// 统计时间块剩余链接任务数
pub async fn count_remaining_tasks_in_block_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: Uuid,
) -> AppResult<i64>
```

##### `TimeBlockRepository` (`features/shared/repositories/time_block_repository.rs`)

```rust
// 在事务中查询时间块
pub async fn find_by_id_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: Uuid,
) -> AppResult<Option<TimeBlock>>

// 非事务查询时间块
pub async fn find_by_id(
    pool: &SqlitePool,
    block_id: Uuid,
) -> AppResult<Option<TimeBlock>>

// 插入时间块
pub async fn insert_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block: &TimeBlock,
) -> AppResult<()>

// 更新时间块
pub async fn update_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: Uuid,
    request: &UpdateTimeBlockRequest,
    updated_at: DateTime<Utc>,
) -> AppResult<TimeBlock>

// 软删除时间块
pub async fn soft_delete_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: Uuid,
) -> AppResult<()>

// 截断时间块到指定时间
pub async fn truncate_to_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: Uuid,
    end_time: DateTime<Utc>,
) -> AppResult<()>

// 查询时间范围内的时间块
pub async fn find_in_range(
    pool: &SqlitePool,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> AppResult<Vec<TimeBlock>>

// 检查时间块是否存在
pub async fn exists_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: Uuid,
) -> AppResult<bool>
```

---

### 前端共享资源

#### API 配置

**`src/composables/useApiConfig.ts`**

```typescript
import { apiBaseUrl, waitForApiReady } from '@/composables/useApiConfig'

// ✅ 正确: 使用动态端口
const response = await fetch(`${apiBaseUrl.value}/tasks`)

// ❌ 错误: 硬编码端口
const response = await fetch('http://127.0.0.1:3538/api/tasks')
```

#### SSE 服务

**`src/services/events.ts`**

```typescript
import { getEventSubscriber } from '@/services/events'

// 在 Store 中订阅事件
const subscriber = getEventSubscriber()
if (subscriber) {
  subscriber.on('task.created', handleTaskCreatedEvent)
}
```

---

## 数据结构修改影响分析

### 修改 Schema 的完整影响链

当你修改数据库 Schema 时,需要同步更新以下所有层次:

```
数据库 Schema (SQLite)
    ↓
后端实体 (Rust entities)
    ↓
后端请求 DTO (Request DTOs)
    ↓
后端响应 DTO (Response DTOs)
    ↓
Assembler (实体到 DTO 的转换)
    ↓
Repository (数据库读写逻辑)
    ↓
端点处理 (API endpoints)
    ↓
前端类型定义 (TypeScript types)
    ↓
Pinia Store (状态管理)
    ↓
Vue 组件 (UI)
```

### 添加字段检查清单

**后端 (必须全部完成)**:

- [ ] **Schema**: 在 `migrations/xxx.sql` 添加字段
- [ ] **Entity**: 更新 `entities/xxx/model.rs` 的 `Entity` struct
- [ ] **EntityRow**: 更新 `entities/xxx/model.rs` 的 `XxxRow` struct
- [ ] **TryFrom**: 更新 `TryFrom<XxxRow>` 实现
- [ ] **Request DTO**: 更新 `entities/xxx/request_dtos.rs`
- [ ] **Response DTO**: 更新 `entities/xxx/response_dtos.rs`
- [ ] **Assembler**: 更新装配器的转换逻辑
- [ ] **Repository**: 更新所有 SQL SELECT/INSERT/UPDATE 语句
- [ ] **⚠️ 跨功能检查**: 搜索是否有其他模块也使用该 DTO

**前端 (必须全部完成)**:

- [ ] **DTO**: 更新 `src/types/dtos.ts`
- [ ] **Store**: 更新 Payload 类型
- [ ] **UI**: 更新组件显示和编辑逻辑

### 跨功能依赖检查

**重要**: 某些实体/DTO 可能被多个功能模块使用!

**查找跨功能依赖**:

```bash
# 查找所有组装该 DTO 的位置
grep -rn "TimeBlockViewDto {" src-tauri/src/features

# 查找所有查询该表的 SQL
grep -rn "SELECT.*FROM time_blocks" src-tauri/src/features
```

**示例: TimeBlock 的跨功能依赖**

- **主功能端点**: `features/endpoints/time_blocks/`
- **共享装配器**: `features/shared/assemblers/time_block_assembler.rs`
- **共享 Repository**: `features/shared/repositories/time_block_repository.rs`
- **关联 Repository**: `features/shared/repositories/task_time_block_link_repository.rs`

修改 TimeBlock 实体时,必须同时更新所有这些位置!

---

## 关键经验教训

### 1. 永远不要硬编码 API 端口 (2025-10-07)

**问题**: 拖拽链接功能无法连接后端

**原因**: 硬编码了端口号,但 Tauri sidecar 使用动态端口

**错误代码**:

```typescript
// ❌ 错误: 硬编码端口
const response = await fetch(
  `http://127.0.0.1:3538/api/time-blocks/${id}/link-task`,
  { ... }
)
```

**正确代码**:

```typescript
// ✅ 正确: 使用动态端口
import { apiBaseUrl } from '@/composables/useApiConfig'

const response = await fetch(
  `${apiBaseUrl.value}/time-blocks/${id}/link-task`,
  { ... }
)
```

---

### 2. 前后端枚举格式不一致导致状态不更新 (2025-10-07)

**问题**: 点击在场按钮后,按钮不会变色

**原因**: 后端输入输出使用不同的枚举格式

后端有两个枚举:

- **输入**: `Outcome` (UPPERCASE: `PLANNED`, `PRESENCE_LOGGED`)
- **输出**: `DailyOutcome` (snake_case: `planned`, `presence_logged`)

**解决方案**:

```typescript
// ✅ 正确: 接收时使用 snake_case (来自 DTO)
const isPresenceLogged = computed(() => {
  return currentScheduleOutcome.value === 'presence_logged'
})

// ✅ 正确: 发送时使用 UPPERCASE (API 输入)
const newOutcome = newCheckedValue ? 'PRESENCE_LOGGED' : 'PLANNED'
await taskStore.updateSchedule(taskId, date, { outcome: newOutcome })
```

---

### 3. 新增字段时必须打通完整数据流 (2025-10-07)

**问题**: 预期时间字段显示 "NaNmin" 且无法持久化

**原因**: 虽然数据库有字段,但 DTO 和 Assembler 缺少映射

**数据流断点**:

```
数据库 (tasks.estimated_duration)
    ↓ ✅ Task 实体有字段
    ↓ ❌ TaskCardDto 缺少字段 ← 第一个断点
    ↓ ❌ Assembler 未映射 ← 第二个断点
    ↓ ✅ 前端 DTO 有字段
    ↓ ✅ UI 显示 (但收到 undefined,显示 NaN)
    ↓ ❌ Update 端点未处理 ← 第三个断点
    ✗ 无法写回数据库
```

**解决方案**: 完整的数据流检查清单 (见上一章)

---

### 4. SSE 事件链的 7 层问题叠加 (2025-10-08)

**问题**: 链接任务到时间块后,时间块不继承 area,卡片不显示时间指示器

**7 层问题**:

1. **业务逻辑缺陷**: 基于 title 判断孤儿时间块,而非 source_type
2. **Store 缺失 SSE**: TimeBlockStore 完全没有事件订阅代码
3. **端点无 SSE**: create_from_task 端点没有发送 SSE 事件
4. **EventSource 未注册**: events.ts 没有 addEventListener
5. **未更新 area_id**: link_task 没有继承任务的 area_id
6. **SSE Payload 不完整**: 只含 ID,无完整数据
7. **API 不存在**: 前端调用不存在的 `/api/time-blocks?ids=X`

**核心教训**:

> SSE 实时更新功能像一条完整的链条,从后端发送 → 网络传输 → EventSource 接收 → Store 处理 → UI 更新,任何一环断裂都会导致功能失效。新增功能时必须验证整条链路的完整性。

**SSE 事件链完整性检查清单**:

**后端 (Rust)**:

- [ ] 端点发送 SSE 事件 (EventOutbox)
- [ ] SSE payload 包含完整数据,不只是 ID
- [ ] 事件类型命名一致 (如 time_blocks.linked)

**中间层 (events.ts)**:

- [ ] EventSource.addEventListener 注册了该事件类型
- [ ] handleEvent 正确解析和分发

**前端 Store**:

- [ ] Store 实现了 initEventSubscriptions
- [ ] Store 订阅了所有相关事件
- [ ] Event handler 正确处理数据
- [ ] useApiConfig.ts 中调用了 initEventSubscriptions

**测试验证**:

- [ ] 控制台可以看到 SSE 事件日志
- [ ] Store handler 被正确调用
- [ ] UI 实时更新,无需手动刷新

---

### 5. 孤儿时间块删除逻辑的业务缺陷 (2025-10-08)

**错误设计**: 基于 `time_block.title == deleted_task.title` 判断是否删除

**Bug 场景**:

```
1. 任务 A 创建时间块 K (title="任务A")
2. 链接任务 B 到时间块 K
3. 删除任务 A → K 保留 (还有任务 B) ✅
4. 删除任务 B → K 保留 (title "任务A" ≠ "任务B") ❌
   结果: 孤儿时间块!
```

**正确方案**: 使用命名空间化的 `source_info.source_type`

```rust
pub struct SourceInfo {
    pub source_type: String,        // "native::from_task" | "native::manual" | "external::*"
    pub created_by_task_id: Option<Uuid>,
}

// 删除时判断
if source_info.source_type == "native::from_task" {
    return Ok(true);  // 孤儿 + 自动创建 = 删除
}
Ok(false)  // 其他来源一律保留
```

**教训**:

- ❌ 不要使用易变的业务数据 (如标题) 作为逻辑判断依据
- ✅ 使用明确的元数据 (source_type) 标记来源和意图
- ✅ 采用命名空间化设计,便于未来扩展

---

### 6. 前端未正确提取后端响应数据导致功能失败 (2025-10-09)

**问题**: 从模板创建任务时，任务创建成功但前端报错"未返回任务数据"，界面不显示新任务

**现象**:

```typescript
// 后端实际返回
{
  "data": { id: "...", title: "..." },  // ✅ TaskCard 在这里
  "timestamp": "2025-10-09T...",
  "request_id": null
}

// 前端错误处理
const taskCard = await response.json()  // ❌ 得到整个包装对象
if (!taskCard.id) {  // ❌ taskCard.id 是 undefined！
  throw new Error('未返回任务数据')
}
```

**根本原因**:

后端统一使用 `ApiResponse<T>` 包装所有成功响应:

```rust
// src-tauri/src/infra/http/responses.rs
pub struct ApiResponse<T> {
    pub data: T,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
}

// src-tauri/src/infra/http/responses.rs
pub fn created_response<T: serde::Serialize>(data: T) -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(ApiResponse::success(data)),
    )
}
```

前端必须从 `response.data` 提取实际数据，但很多地方直接使用了 `await response.json()`。

**正确方案**:

```typescript
// ❌ 错误
const entity = await response.json()
return entity

// ✅ 正确
const responseData = await response.json()
const entity = responseData.data // 提取 data 字段
return entity
```

**影响范围**: 所有 POST/PATCH 请求（GET 请求也使用 `ApiResponse` 包装）

**修复检查清单**:

- [ ] 所有 `createXxx` 函数
- [ ] 所有 `updateXxx` 函数
- [ ] 所有 `fetchXxx` 函数（如果返回单个对象）
- [ ] 特殊端点（如 `createTaskFromTemplate`）

**教训**:

- ✅ 前后端约定必须明确文档化
- ✅ 在开发手册中明确说明响应格式
- ✅ 新增 API 调用时参考现有正确实现
- ✅ 添加详细日志帮助快速定位问题

**调试技巧**:

```typescript
// 添加详细日志
const responseData = await response.json()
console.log('Raw response:', responseData)
console.log('Has data field:', !!responseData?.data)
console.log('Data keys:', responseData?.data ? Object.keys(responseData.data) : [])
```

---

### 7. 三态字段序列化/反序列化问题 (2025-10-09)

**问题**: 更新请求中的可空字段（如 `area_id`）无法正确设置为 NULL

**背景**:

在 PATCH 请求中，我们需要区分三种状态：

1. **不更新该字段** - 前端不发送该字段
2. **设置为 NULL** - 前端发送 `null`
3. **设置为新值** - 前端发送具体值

这需要使用 `Option<Option<T>>` 类型（嵌套 Option）。

**错误实现**:

```rust
// ❌ 错误: 无法区分"不更新"和"设为 NULL"
#[derive(Deserialize)]
pub struct UpdateRequest {
    pub area_id: Option<Uuid>,  // None 既可能是"不发送"也可能是"发送 null"
}
```

**序列化问题**:

如果不添加自定义反序列化器，serde 无法正确处理嵌套 Option：

```rust
// ❌ 错误: 缺少自定义反序列化器
pub struct UpdateRequest {
    pub area_id: Option<Option<Uuid>>,  // serde 默认行为不正确
}

// 前端发送 { "area_id": null }
// serde 可能解析为 None (不更新) 而非 Some(None) (设为 NULL)
```

**正确实现**:

**1. 定义自定义反序列化器**:

```rust
use serde::Deserialize;

/// 自定义反序列化器，用于正确处理三态字段
/// - 字段缺失 → None (不更新)
/// - 字段为 null → Some(None) (设为 NULL)
/// - 字段有值 → Some(Some(value)) (设为值)
fn deserialize_nullable_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    use serde::Deserialize;
    Ok(Some(Option::deserialize(deserializer)?))
}
```

**2. 在 DTO 中使用**:

```rust
#[derive(Debug, Deserialize, Default)]
pub struct UpdateTemplateRequest {
    pub title: Option<String>,  // 非空字段，用普通 Option

    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub glance_note_template: Option<Option<String>>,  // 可空字段

    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub area_id: Option<Option<Uuid>>,  // 可空字段
}
```

**3. 在 Repository 中正确绑定**:

```rust
// ❌ 错误: 将 None 转为空字符串，无法设置 NULL
if let Some(ref area_id_opt) = request.area_id {
    bindings.push(area_id_opt.map(|id| id.to_string()).unwrap_or_default());
}

// ✅ 正确: 保持 Option 类型，让 SQLx 正确处理 NULL
if let Some(ref area_id_opt) = request.area_id {
    let bind_val: Option<String> = area_id_opt.map(|id| id.to_string());
    q = q.bind(bind_val);  // SQLx 会将 None 转为 SQL NULL
}
```

**参考实现**:

查看以下文件获取完整示例：

- `src-tauri/src/entities/task/request_dtos.rs` - Task 的三态字段实现
- `src-tauri/src/entities/template/request_dtos.rs` - Template 的三态字段实现
- `src-tauri/src/entities/time_block/request_dtos.rs` - TimeBlock 的三态字段实现
- `src-tauri/src/features/shared/repositories/task_repository.rs` - Task 的绑定逻辑

**完整数据流示例**:

```typescript
// 前端: 更新模板，设置 area_id 为 null
await updateTemplate(templateId, {
  title: '新标题', // 更新为新值
  area_id: null, // 设置为 NULL
  // glance_note 字段不发送 → 不更新
})
```

```rust
// 后端: 接收请求
pub struct UpdateTemplateRequest {
    pub title: Option<String>,                              // Some("新标题")
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub area_id: Option<Option<Uuid>>,                      // Some(None)
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub glance_note_template: Option<Option<String>>,       // None
}

// 后端: 构建 SQL
let mut set_clauses = vec![];
if request.title.is_some() { set_clauses.push("title = ?"); }        // ✅ 添加
if request.area_id.is_some() { set_clauses.push("area_id = ?"); }    // ✅ 添加
if request.glance_note_template.is_some() { /*不添加*/ }             // ❌ 跳过

// 后端: 绑定参数
if let Some(ref title) = request.title {
    q = q.bind(title);  // "新标题"
}
if let Some(ref area_id_opt) = request.area_id {
    let bind_val: Option<String> = area_id_opt.map(|id| id.to_string());
    q = q.bind(bind_val);  // None → SQL NULL
}
// glance_note_template 没有绑定

// 最终 SQL
UPDATE templates SET title = ?, area_id = ?, updated_at = ? WHERE id = ?
// 绑定值: ["新标题", NULL, "2025-10-09...", "template-uuid"]
```

**开发检查清单**:

为所有 `UpdateXxxRequest` 添加三态字段支持时：

- [ ] 确定哪些字段是可空的（Schema 中允许 NULL）
- [ ] 为可空字段使用 `Option<Option<T>>` 类型
- [ ] 添加 `#[serde(default, deserialize_with = "deserialize_nullable_field")]` 标注
- [ ] 为 DTO 添加 `#[derive(Default)]`
- [ ] 复制 `deserialize_nullable_field` 函数（如果文件中没有）
- [ ] 在 Repository 绑定逻辑中使用 `Option<String>` 而非 `.unwrap_or_default()`
- [ ] 在验证逻辑中使用双重模式匹配 `if let Some(Some(value))`

**常见错误**:

```rust
// ❌ 错误 1: 忘记自定义反序列化器
pub area_id: Option<Option<Uuid>>,  // 缺少 #[serde(...)]

// ❌ 错误 2: 绑定时使用 unwrap_or_default
bindings.push(area_id_opt.unwrap_or_default());  // 将 None 变成空字符串

// ❌ 错误 3: 验证时单层模式匹配
if let Some(duration) = request.duration {  // 应该是 Some(Some(duration))
    if duration <= 0 { ... }
}
```

**教训**:

- ✅ 所有 Update DTO 的可空字段必须统一使用三态逻辑
- ✅ 参考 Task/Template/TimeBlock 的实现保持一致性
- ✅ 绑定参数时必须保持类型为 `Option<T>`，让数据库驱动处理 NULL
- ✅ 不要使用 `Vec<String>` 统一绑定所有参数（无法表达 NULL）

---

## 开发检查清单

### 后端开发检查清单

**开发前**:

- [ ] 查看数据库 Schema (`migrations/xxx.sql`)
- [ ] 查看共享资源清单,确认可复用的 Repository/Assembler
- [ ] 选择参考实现 (Area/Task/TimeBlock)

**实体层**:

- [ ] 创建 Entity struct
- [ ] 创建 EntityRow struct
- [ ] 实现 TryFrom<EntityRow>
- [ ] 创建 Request DTOs
- [ ] 创建 Response DTOs
- [ ] 导出模块

**端点层 (SFC)**:

- [ ] 编写完整的 CABC 文档
- [ ] 实现 HTTP Handler
- [ ] 实现 Validation (如需要)
- [ ] 实现 Business Logic
- [ ] 实现 Database Access
- [ ] 使用正确的 trait 方法 (`new_uuid()`, `now_utc()`)
- [ ] 使用 TransactionHelper
- [ ] 复用共享资源,不重复实现
- [ ] 查询实际状态,不依赖默认值
- [ ] 填充完整数据后才写入 SSE
- [ ] SSE 和 HTTP 返回相同数据

**路由注册**:

- [ ] 在 feature 的 mod.rs 中注册端点
- [ ] 在 features/mod.rs 中注册 feature

**文档**:

- [ ] 编写 API_SPEC.md

**测试**:

- [ ] 运行 `cargo check`
- [ ] 运行 `cargo clippy`
- [ ] 测试 API (curl/Postman)
- [ ] 测试 SSE 事件
- [ ] 测试完整数据流

---

### 前端开发检查清单

**类型层**:

- [ ] 在 `src/types/dtos.ts` 添加 interface

**Store 层**:

- [ ] 创建 core.ts (State & Getters)
- [ ] 创建 crud-operations.ts
- [ ] 创建 view-operations.ts
- [ ] 创建 event-handlers.ts
- [ ] 在 index.ts 组合所有模块
- [ ] 在 useApiConfig.ts 初始化 SSE 订阅

**SSE 层**:

- [ ] 在 events.ts 注册 addEventListener

**UI 层**:

- [ ] 创建管理/列表组件
- [ ] 创建编辑/详情组件
- [ ] 添加路由
- [ ] 添加导航链接

**测试**:

- [ ] 检查 linter 错误
- [ ] 测试 CRUD 操作
- [ ] 测试 SSE 实时更新
- [ ] 测试完整工作流

---

### 数据结构修改检查清单

**当你添加/修改字段时,必须检查**:

**后端**:

- [ ] Schema: migrations/xxx.sql
- [ ] Entity: entities/xxx/model.rs (Entity + EntityRow + TryFrom)
- [ ] Request DTO: entities/xxx/request_dtos.rs
- [ ] Response DTO: entities/xxx/response_dtos.rs
- [ ] Assembler: features/shared/assemblers/xxx_assembler.rs (如果使用共享 Assembler)
- [ ] Repository: 所有 SELECT/INSERT/UPDATE SQL
- [ ] 跨功能装配器: `grep -rn "XxxDto {" src-tauri/src/features`
- [ ] 跨功能 Repository: `grep -rn "SELECT.*FROM xxx" src-tauri/src/features`

**前端**:

- [ ] DTO: src/types/dtos.ts
- [ ] Store: src/stores/xxx.ts
- [ ] UI: 显示和编辑逻辑

---

## 常见问题与调试

### Q1: 我应该从哪个文件开始看代码?

**A**: 按照这个顺序:

1. `migrations/xxx.sql` - 理解数据结构
2. `entities/task/model.rs` - 理解实体
3. `features/tasks/endpoints/create_task.rs` - 理解端点 (SFC 模式)
4. `src/types/dtos.ts` - 理解前端数据
5. `src/stores/task.ts` - 理解状态管理
6. `src/components/parts/kanban/KanbanTaskCard.vue` - 理解 UI

---

### Q2: 如何确保数据一致性?

**A**: 遵循这些原则:

1. **后端返回真实状态**: 查询 DB,不用默认值
2. **后端返回完整数据**: 包含受影响的关联对象
3. **先填充后发送**: SSE 之前填充所有关联数据
4. **前端正确提取数据**: ⚠️ **必须从 `responseData.data` 提取** (见 Q7)
5. **前端创建新对象**: `new Map(...)` 触发响应式

---

### Q3: 如何调试响应式更新问题?

**A**: 检查链路:

1. **API 返回了什么?** (Network tab)
2. **Store 更新了吗?** (`console.log` 或 Vue DevTools)
3. **Getter 重新计算了吗?** (添加 `console.log`)
4. **Computed 触发了吗?** (添加 `console.log`)

---

### Q4: 如何调试 SSE 问题?

**A**: 按顺序检查:

1. **后端是否发送?** 查看后端日志、数据库 event_outbox 表
2. **网络传输?** 浏览器 DevTools → Network → EventStream
3. **EventSource 接收?** 查看 `addEventListener` 是否注册
4. **Store 订阅?** `initEventSubscriptions` 是否调用
5. **Handler 执行?** 添加 `console.log` 确认被调用
6. **数据处理?** 验证 payload 结构和内容

---

### Q5: 遇到编译错误怎么办?

**A**: 常见编译错误:

**错误 1**: `no column found for name: xxx`

- **原因**: 忘记在 SQL SELECT 中添加新字段
- **解决**: 更新所有查询该表的 SQL

**错误 2**: `missing field 'xxx' in initializer`

- **原因**: Assembler 或 DTO 初始化缺少字段
- **解决**: 更新装配器和所有 DTO 初始化

**错误 3**: `method not found in IdGenerator`

- **原因**: 使用了错误的方法名
- **解决**: 使用 `new_uuid()` 而非 `generate()`

---

### Q6: 如何找到重复的代码?

**A**: 使用以下命令:

```bash
# 查找所有组装 DTO 的位置
grep -rn "TaskCardDto {" src-tauri/src/features

# 查找所有查询某表的 SQL
grep -rn "SELECT.*FROM tasks" src-tauri/src

# 查找所有 SSE 发送点
grep -rn "DomainEvent::new" src-tauri/src
```

---

### Q7: 为什么后端返回了数据但前端报错"未返回数据"?

**A**: 检查是否正确提取了 `ApiResponse` 包装的数据

**症状**:

```typescript
const task = await response.json()
console.log(task) // { data: {...}, timestamp: "...", request_id: null }
console.log(task.id) // undefined ❌
```

**原因**: 后端所有成功响应都使用 `ApiResponse<T>` 包装

**解决方案**:

```typescript
// ❌ 错误
const entity = await response.json()
return entity // 返回的是整个包装对象

// ✅ 正确
const responseData = await response.json()
const entity = responseData.data // 提取 data 字段
return entity
```

**快速检查**:

```typescript
// 在浏览器 Network Tab 中查看响应
// 如果看到 { data: {...}, timestamp: "...", request_id: null }
// 那么就需要提取 .data
```

**影响范围**:

- ✅ 所有 POST 请求 (创建资源)
- ✅ 所有 PATCH 请求 (更新资源)
- ✅ 所有 GET 请求 (获取单个资源)
- ❌ DELETE 请求通常返回 204 No Content

**相关教训**: 见 [关键经验教训 #6](#6-前端未正确提取后端响应数据导致功能失败-2025-10-09)

---

### Q8: `infra/` 和 `features/shared/` 有什么区别？

**A**: 两者的核心区别在于**关注点**

**`infra/` - 基础设施层**:

- **关注点**: 技术实现（How）
- **职责**: HTTP、数据库、日志、事件等技术性组件
- **无业务语义**: 不包含任何业务规则
- **示例**: `AppError`, `success_response`, `Clock`, `IdGenerator`

**`features/shared/` - 业务共享层**:

- **关注点**: 业务逻辑（What）
- **职责**: Repositories、Assemblers、Services、Validators
- **包含业务语义**: 理解领域概念（Task、TimeBlock 等）
- **示例**: `TaskRepository`, `TaskValidator`, `TaskAssembler`

**错误示例**:

```rust
// ❌ 错误: shared 已重命名为 infra
use crate::shared::core::AppError;

// ❌ 错误: 混淆业务层和基础设施层
use crate::features::shared::AppError;  // AppError 在 infra 中
use crate::infra::TaskRepository;       // TaskRepository 在 features/shared 中
```

**正确示例**:

```rust
// ✅ 基础设施导入
use crate::infra::core::{AppError, AppResult};
use crate::infra::http::success_response;

// ✅ 业务共享导入
use crate::features::shared::{TaskRepository, TaskValidator};
```

**快速判断**:

- 如果涉及 Task、TimeBlock、Area 等领域概念 → `features/shared/`
- 如果是通用错误、HTTP、日志等技术工具 → `infra/`

**相关章节**: 见 [后端架构概览](#后端架构概览)

---

## 总结

### 开发新功能的核心步骤

1. **理解架构** - 清楚 `infra/` 和 `features/shared/` 的区别
2. **查看 Schema** - 理解数据结构
3. **查看共享资源** - 避免重复实现
4. **参考类似功能** - 复用模式
5. **遵循 SFC 规范** - 统一代码结构
6. **使用共享层** - Repository、Validator、Assembler
7. **填充完整数据** - 确保数据真实性
8. **SSE 一致性** - 与 HTTP 返回相同数据
9. **完整测试** - 端到端验证

### 记住这些原则

- ✅ **分层清晰**: `infra/` (技术) vs `features/shared/` (业务)
- ✅ **Schema 优先**: 先看数据库,不要猜测
- ✅ **复用优先**: 使用共享资源,不要重复
- ✅ **数据真实**: 查询实际状态,不用默认值
- ✅ **SSE 一致**: 先填充数据,再发送事件
- ✅ **文档驱动**: 代码必须与 CABC 文档一致
- ✅ **响应提取**: 前端必须从 `responseData.data` 提取数据
- ✅ **正确导入**: 基础设施从 `infra`，业务逻辑从 `features/shared`

### 架构原则

**Cutie 采用清晰的分层架构**:

```
infra/              ← 基础设施（技术实现）
features/
  ├── shared/       ← 业务共享层（业务逻辑复用）
  ├── endpoints/    ← HTTP 端点层（API handlers）
  └── *.rs          ← 功能模块（路由定义）
entities/           ← 领域模型（DTOs）
```

**依赖方向**: endpoints → shared → infra

**关键区别**:

- `infra`: AppError, success_response, Clock, IdGenerator
- `features/shared`: TaskRepository, TaskValidator, TaskAssembler

### 遇到问题时

1. **查文档** (本手册、后端架构概览、SFC_SPEC、LESSONS_LEARNED)
2. **看代码** (参考类似功能)
3. **检查清单** (确保没有遗漏步骤)
4. **查 Schema** (确认数据库结构)
5. **理解分层** (确认模块应该放在哪一层)
6. **调试数据流** (使用 console.log 和 DevTools)

### 快速参考

**基础设施层导入**:

```rust
use crate::infra::core::{AppError, AppResult};
use crate::infra::http::{success_response, created_response};
use crate::infra::ports::{Clock, IdGenerator};
```

**业务共享层导入**:

```rust
use crate::features::shared::{
    TaskRepository,
    TaskValidator,
    TaskAssembler,
    TransactionHelper,
};
```

**端点文件位置**:

- 端点实现: `features/endpoints/xxx/create_xxx.rs`
- 端点导出: `features/endpoints/xxx/mod.rs`
- 路由定义: `features/xxx.rs`

---

**记住: Cutie 的架构是经过深思熟虑的,遵循规范可以避免 90% 的问题!** 📚✨

**版本历史**:

- v1.0 (2025-10-08): 初版
- v2.0 (2025-10-12): 更新架构（`shared` → `infra`，新增 `features/shared/validators`）

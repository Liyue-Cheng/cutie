# Cutie 后端开发手册

## 一、后端架构概览

### 1.1 技术栈

- **Rust 2021 Edition**
- **Axum** - Web 框架
- **SQLx** - 数据库查询（编译时检查）
- **SQLite** - 数据库
- **Tokio** - 异步运行时
- **Serde** - 序列化/反序列化
- **SSE (Server-Sent Events)** - 实时通信

### 1.2 项目结构

```
src-tauri/src/
  entities/          # 数据实体定义
  features/          # 功能模块
    endpoints/       # API 端点（单文件组件）
    shared/          # 共享逻辑
    {domain}.rs      # 路由注册
  infra/             # 基础设施
    core/            # 核心类型和错误
    http/            # HTTP 响应格式
    events/          # SSE 事件系统
  migrations/        # 数据库迁移
  startup/           # 应用启动配置
```

### 1.3 请求响应流程

```
HTTP Request
  ↓
Axum Router
  ↓
端点 Handler (handle)
  ↓
业务逻辑层 (logic::execute)
  ↓
获取写入许可 (write_permit)
  ↓
开启事务 (begin)
  ↓
数据库操作层 (database::*)
  ↓
写入 Event Outbox (事务内)
  ↓
提交事务 (commit)
  ↓
包装响应 (success_response)
  ↓
HTTP Response
  ↓ (异步)
SSE Dispatcher 读取 Event Outbox
  ↓
推送 SSE 事件到所有客户端
```

---

## 二、单文件端点组件开发

### 2.1 文件结构

**位置**：`src/features/endpoints/{domain}/{action}.rs`

**命名规则**：

- 动词开头，蛇形命名：`create_task.rs`, `update_schedule.rs`
- 一个文件一个端点
- 包含完整的业务逻辑

### 2.2 完整模板

```rust
/// 端点功能描述 - 单文件组件
///
/// HTTP_METHOD /api/{domain}/{path}

// ==================== CABC 文档 ====================
/*
CABC for `endpoint_name`

## 1. 端点签名
POST /api/domain/action

## 2. 预期行为简介
### 2.1 用户故事
> 作为用户，我想要...以便...

### 2.2 核心业务逻辑
简要描述业务逻辑

## 3. 输入输出规范
### 3.1 请求
**URL Parameters**: id (UUID)
**Request Body**: { field: "value" }

### 3.2 响应
**201 Created**: 返回 DTO
**400 Bad Request**: 验证失败
**404 Not Found**: 资源不存在

## 4. 验证规则
- field 不能为空
- field 长度不超过 100

## 5. 业务逻辑详解
1. 验证输入
2. 查询依赖数据
3. 创建实体
4. 保存到数据库
5. 发送 SSE 事件

## 6. 边界情况
- 资源不存在: 404
- 并发冲突: 409

## 7. 预期副作用
### 数据库操作:
- SELECT: 查询表
- INSERT: 插入表
- 事务边界: begin() → commit()

### SSE 事件:
- domain.created

## 8. 契约
### 前置条件:
- 必要的关联资源存在

### 后置条件:
- 新资源已创建
- SSE 事件已推送
*/

// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::domain::{Entity, EntityDto},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 请求/响应结构 ====================
#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateResponse {
    pub id: Uuid,
    pub title: String,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateRequest,
    ) -> AppResult<EntityDto> {
        // 1. 验证
        validate(&request)?;

        // 2. 获取依赖
        let entity_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 3. 获取写入许可（防止并发冲突）
        let _permit = app_state.acquire_write_permit().await;

        // 4. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 5. 创建实体
        let entity = Entity {
            id: entity_id,
            title: request.title.clone(),
            description: request.description.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
            is_deleted: false,
        };

        // 6. 保存到数据库
        database::insert(&mut tx, &entity).await?;

        // 7. 写入 Event Outbox（在事务内）
        events::write_created_event(&mut tx, app_state, &entity).await?;

        // 8. 提交事务
        tx.commit().await?;

        // 9. 返回 DTO
        Ok(EntityDto::from(entity))
    }

    fn validate(request: &CreateRequest) -> AppResult<()> {
        if request.title.trim().is_empty() {
            return Err(AppError::ValidationError {
                field: "title".to_string(),
                message: "标题不能为空".to_string(),
            });
        }
        Ok(())
    }
}

// ==================== 数据库层 ====================
mod database {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    pub async fn insert(
        tx: &mut Transaction<'_, Sqlite>,
        entity: &Entity,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO entities (id, title, description, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entity.id)
        .bind(&entity.title)
        .bind(&entity.description)
        .bind(&entity.created_at)
        .bind(&entity.updated_at)
        .bind(entity.is_deleted)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}

// ==================== 事件层 ====================
mod events {
    use super::*;
    use crate::infra::events::{
        models::DomainEvent,
        outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
    };

    pub async fn write_created_event(
        tx: &mut Transaction<'_, Sqlite>,
        app_state: &AppState,
        entity: &Entity,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let event = DomainEvent::new(
            "domain.created",
            "entity",
            entity.id.to_string(),
            serde_json::to_value(EntityDto::from(entity.clone())).unwrap(),
        );

        outbox_repo.insert_in_tx(&mut **tx, event).await?;
        Ok(())
    }
}
```

### 2.3 关键点说明

#### success_response 包装

**✅ 正确**：

```rust
use crate::infra::http::error_handler::success_response;

pub async fn handle(...) -> Response {
    match logic::execute(...).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}
```

**❌ 错误**：

```rust
Ok(dto) => Json(dto).into_response(),  // 缺少标准响应包装
```

#### 写入许可

```rust
// 获取写入许可（确保写操作串行执行，防止竞态条件）
let _permit = app_state.acquire_write_permit().await;
```

**作用**：

- 防止并发写入冲突
- 确保数据一致性
- 自动在作用域结束时释放

#### 事务管理

```rust
// 开启事务
let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

// 数据库操作...
database::insert(&mut tx, &entity).await?;

// Event Outbox 写入（必须在事务内）
events::write_created_event(&mut tx, app_state, &entity).await?;

// 提交事务（如果失败会自动回滚）
tx.commit().await?;
```

**注意**：Event Outbox 必须在事务内写入，确保原子性。

---

## 三、路由注册

### 3.1 功能模块路由文件

**位置**：`src/features/{domain}.rs`

```rust
/// Domain 功能模块
use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use crate::startup::AppState;

// 引入端点
mod endpoints {
    pub use crate::features::endpoints::domain::*;
}

/// 创建路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list))              // GET /domain
        .route("/", post(endpoints::create))           // POST /domain
        .route("/:id", get(endpoints::get_by_id))      // GET /domain/:id
        .route("/:id", patch(endpoints::update))       // PATCH /domain/:id
        .route("/:id", delete(endpoints::delete))      // DELETE /domain/:id
        .route("/:id/action", post(endpoints::action)) // POST /domain/:id/action
}
```

### 3.2 主路由注册

**位置**：`src/features/mod.rs`

```rust
pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .nest("/tasks", tasks::create_routes())
        .nest("/schedules", schedules::create_routes())
        .nest("/templates", templates::create_routes())
        .nest("/domain", domain::create_routes())  // 新增
}
```

---

## 四、数据实体定义

### 4.1 实体结构

**位置**：`src/entities/{domain}.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 数据库行结构（用于 SQLx 查询）
#[derive(Debug, Clone, FromRow)]
pub struct EntityRow {
    pub id: String,  // SQLite 存储为 TEXT
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_deleted: bool,
}

/// 内部实体结构（业务逻辑使用）
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

/// DTO 结构（API 响应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,  // ISO 8601
    pub updated_at: String,  // ISO 8601
}

// ==================== 类型转换 ====================

impl TryFrom<EntityRow> for Entity {
    type Error = String;

    fn try_from(row: EntityRow) -> Result<Self, Self::Error> {
        Ok(Entity {
            id: Uuid::parse_str(&row.id).map_err(|e| e.to_string())?,
            title: row.title,
            description: row.description,
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc),
            is_deleted: row.is_deleted,
        })
    }
}

impl From<Entity> for EntityDto {
    fn from(entity: Entity) -> Self {
        EntityDto {
            id: entity.id,
            title: entity.title,
            description: entity.description,
            created_at: entity.created_at.to_rfc3339(),
            updated_at: entity.updated_at.to_rfc3339(),
        }
    }
}
```

### 4.2 类型约定

**数据库 ↔ Rust 映射**：

- `TEXT` ↔ `String`
- `INTEGER` ↔ `i64` / `Option<i64>`
- `BOOLEAN` ↔ `bool`
- `TEXT (UUID)` ↔ `String` → `Uuid`
- `TEXT (RFC 3339)` ↔ `String` → `DateTime<Utc>`
- `TEXT (JSON)` ↔ `String` → `serde_json::Value`

---

## 五、数据库操作

### 5.1 查询操作

**单条记录**：

```rust
pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> AppResult<Option<Entity>> {
    let row = sqlx::query_as::<_, EntityRow>(
        r#"
        SELECT * FROM entities
        WHERE id = ? AND is_deleted = FALSE
        "#,
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.into()))?;

    match row {
        Some(r) => Ok(Some(r.try_into().map_err(|e: String| {
            AppError::DatabaseError(crate::infra::core::DbError::ParseError(e))
        })?)),
        None => Ok(None),
    }
}
```

**多条记录**：

```rust
pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<Entity>> {
    let rows = sqlx::query_as::<_, EntityRow>(
        r#"
        SELECT * FROM entities
        WHERE is_deleted = FALSE
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.into()))?;

    rows.into_iter()
        .map(|r| r.try_into().map_err(|e: String| {
            AppError::DatabaseError(crate::infra::core::DbError::ParseError(e))
        }))
        .collect()
}
```

### 5.2 插入操作

```rust
pub async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    entity: &Entity,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO entities (id, title, description, created_at, updated_at, is_deleted)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(entity.id.to_string())
    .bind(&entity.title)
    .bind(&entity.description)
    .bind(entity.created_at.to_rfc3339())
    .bind(entity.updated_at.to_rfc3339())
    .bind(entity.is_deleted)
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.into()))?;

    Ok(())
}
```

### 5.3 更新操作

```rust
pub async fn update(
    tx: &mut Transaction<'_, Sqlite>,
    entity: &Entity,
) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE entities
        SET title = ?, description = ?, updated_at = ?
        WHERE id = ? AND is_deleted = FALSE
        "#,
    )
    .bind(&entity.title)
    .bind(&entity.description)
    .bind(entity.updated_at.to_rfc3339())
    .bind(entity.id.to_string())
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            entity_type: "Entity".to_string(),
            entity_id: entity.id.to_string(),
        });
    }

    Ok(())
}
```

### 5.4 软删除操作

```rust
pub async fn soft_delete(
    tx: &mut Transaction<'_, Sqlite>,
    id: Uuid,
    deleted_at: DateTime<Utc>,
) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE entities
        SET is_deleted = TRUE, updated_at = ?
        WHERE id = ? AND is_deleted = FALSE
        "#,
    )
    .bind(deleted_at.to_rfc3339())
    .bind(id.to_string())
    .execute(&mut **tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            entity_type: "Entity".to_string(),
            entity_id: id.to_string(),
        });
    }

    Ok(())
}
```

---

## 六、SSE 事件系统

### 6.1 Event Outbox 模式

**目的**：保证事件的可靠投递

**流程**：

1. 业务逻辑在事务内写入 Event Outbox
2. 事务提交后，事件持久化到数据库
3. SSE Dispatcher 定期扫描未分发的事件
4. 推送到所有连接的客户端
5. 标记事件为已分发

### 6.2 写入事件

```rust
use crate::infra::events::{
    models::DomainEvent,
    outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
};

pub async fn write_event(
    tx: &mut Transaction<'_, Sqlite>,
    app_state: &AppState,
    entity: &Entity,
) -> AppResult<()> {
    let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

    // 创建事件
    let event = DomainEvent::new(
        "domain.created",        // 事件类型
        "entity",                // 聚合类型
        entity.id.to_string(),   // 聚合 ID
        serde_json::to_value(EntityDto::from(entity.clone())).unwrap(),
    );

    // 在事务内写入
    outbox_repo.insert_in_tx(&mut **tx, event).await?;

    Ok(())
}
```

### 6.3 事件命名规范

**格式**：`domain.action`（小写，点分隔）

**示例**：

```
task.created
task.updated
task.deleted
schedule.created
template.created
```

---

## 七、错误处理

### 7.1 AppError 类型

```rust
pub enum AppError {
    NotFound {
        entity_type: String,
        entity_id: String,
    },
    ValidationError {
        field: String,
        message: String,
    },
    DatabaseError(DbError),
    Conflict {
        message: String,
    },
    InternalError {
        message: String,
    },
}
```

### 7.2 错误使用示例

```rust
// 404 Not Found
return Err(AppError::NotFound {
    entity_type: "Task".to_string(),
    entity_id: task_id.to_string(),
});

// 400 Validation Error
return Err(AppError::ValidationError {
    field: "title".to_string(),
    message: "标题不能为空".to_string(),
});

// 500 Database Error
return Err(AppError::DatabaseError(DbError::ConnectionError(e)));

// 409 Conflict
return Err(AppError::Conflict {
    message: "该日期已存在日程".to_string(),
});
```

### 7.3 自动错误响应

`AppError` 实现了 `IntoResponse`，会自动转换为标准错误响应：

```json
{
  "error_type": "NotFound",
  "message": "Task with id xxx not found",
  "details": {
    "entity_type": "Task",
    "entity_id": "xxx"
  },
  "code": "NOT_FOUND",
  "timestamp": "2025-10-16T13:00:00Z"
}
```

---

## 八、数据库迁移

### 8.1 迁移文件命名

**格式**：`{timestamp}_{description}.sql`

**示例**：

```
20241001000000_initial_schema.sql
20241016130000_add_template_category.sql
```

### 8.2 迁移文件结构

```sql
-- 迁移描述
--
-- 详细说明

-- 创建表
CREATE TABLE entities (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE
);

-- 创建索引
CREATE INDEX idx_entities_updated_at ON entities(updated_at);
CREATE INDEX idx_entities_is_deleted ON entities(is_deleted);
```

### 8.3 字段类型约定

**ID 字段**：

```sql
id TEXT PRIMARY KEY NOT NULL  -- UUID 字符串
```

**时间字段**：

```sql
created_at TEXT NOT NULL  -- RFC 3339 格式
updated_at TEXT NOT NULL
deleted_at TEXT           -- 可空
```

**布尔字段**：

```sql
is_deleted BOOLEAN NOT NULL DEFAULT FALSE
is_completed BOOLEAN NOT NULL DEFAULT FALSE
```

**JSON 字段**：

```sql
metadata TEXT  -- JSON 字符串
subtasks TEXT  -- JSON 数组
```

---

## 九、最佳实践

### 9.1 代码组织

**分层清晰**：

```rust
// HTTP 层
pub async fn handle(...) -> Response { /* ... */ }

// 业务逻辑层
mod logic {
    pub async fn execute(...) -> AppResult<...> { /* ... */ }
}

// 数据库层
mod database {
    pub async fn insert(...) -> AppResult<()> { /* ... */ }
}

// 事件层
mod events {
    pub async fn write_event(...) -> AppResult<()> { /* ... */ }
}
```

### 9.2 命名规范

**文件名**：蛇形命名

```
create_task.rs
update_schedule.rs
delete_template.rs
```

**函数名**：蛇形命名

```rust
pub async fn find_by_id(...) -> AppResult<...>
pub async fn list_all(...) -> AppResult<Vec<...>>
pub async fn insert(...) -> AppResult<()>
```

**类型名**：帕斯卡命名

```rust
pub struct TaskRow { /* ... */ }
pub struct TaskDto { /* ... */ }
pub enum AppError { /* ... */ }
```

### 9.3 错误处理

**使用 ?** 操作符：

```rust
let entity = database::find_by_id(pool, id).await?;
```

**转换错误类型**：

```rust
.map_err(|e| AppError::DatabaseError(e.into()))?
```

**提供上下文**：

```rust
.ok_or_else(|| AppError::NotFound {
    entity_type: "Task".to_string(),
    entity_id: id.to_string(),
})?
```

### 9.4 事务管理

**始终使用事务**：

```rust
let mut tx = TransactionHelper::begin(pool).await?;
// 数据库操作...
tx.commit().await?;
```

**在事务内写入 Event Outbox**：

```rust
database::insert(&mut tx, &entity).await?;
events::write_event(&mut tx, app_state, &entity).await?;
tx.commit().await?;
```

---

## 十、调试与日志

### 10.1 使用 tracing

```rust
use tracing::{debug, info, warn, error};

// Info 级别
info!("Entity created: id={}", entity.id);

// Debug 级别
debug!("Query executed: {:?}", query);

// Warning 级别
warn!("Deprecated endpoint called: {}", path);

// Error 级别
error!("Database error: {}", err);
```

### 10.2 性能日志

```rust
let start = std::time::Instant::now();
// 执行操作...
tracing::debug!(
    "[PERF] Operation took {:.3}ms",
    start.elapsed().as_secs_f64() * 1000.0
);
```

---

## 十一、开发检查清单

**创建新端点前**：

- [ ] 确定 HTTP 方法和路径
- [ ] 确定请求/响应结构
- [ ] 确定需要哪些数据库操作
- [ ] 确定需要发送哪些 SSE 事件

**开发过程**：

- [ ] 编写完整的 CABC 文档
- [ ] 定义请求/响应结构体
- [ ] 实现 HTTP 处理器
- [ ] 实现业务逻辑层
- [ ] 实现数据库操作层
- [ ] 实现事件写入层
- [ ] 注册路由

**测试验证**：

- [ ] `cargo check` 编译通过
- [ ] `cargo test` 测试通过
- [ ] 使用 `success_response` 包装响应
- [ ] 获取写入许可
- [ ] 在事务内写入 Event Outbox
- [ ] 正确的错误处理
- [ ] API 文档已更新

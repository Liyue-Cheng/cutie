# Cutie 后端公共资源

## 一、基础设施

### 1.1 AppState

**位置**: `src/startup.rs`

**用途**: 应用全局状态，提供所有依赖

**可用方法**:

```rust
// 数据库连接池
app_state.db_pool()

// ID 生成器
let id = app_state.id_generator().new_uuid()

// 时钟
let now = app_state.clock().now_utc()

// 写入许可
let _permit = app_state.acquire_write_permit().await

// Event Publisher (SSE)
app_state.event_publisher()
```

### 1.2 ID 生成器

**用途**: 生成 UUID

**使用方法**:

```rust
let id = app_state.id_generator().new_uuid()
// => Uuid
```

### 1.3 时钟服务

**用途**: 获取当前时间（便于测试）

**使用方法**:

```rust
let now = app_state.clock().now_utc()
// => DateTime<Utc>

// 转换为 RFC 3339 字符串
let now_str = now.to_rfc3339()
// => "2025-10-16T13:00:00Z"
```

---

## 二、错误处理

### 2.1 AppError

**位置**: `src/infra/core/error.rs`

**类型定义**:

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

**使用方法**:

```rust
// 404 Not Found
return Err(AppError::NotFound {
    entity_type: "Task".to_string(),
    entity_id: id.to_string(),
});

// 400 Validation Error
return Err(AppError::ValidationError {
    field: "title".to_string(),
    message: "标题不能为空".to_string(),
});

// 500 Database Error
.map_err(|e| AppError::DatabaseError(e.into()))?

// 409 Conflict
return Err(AppError::Conflict {
    message: "资源已存在".to_string(),
});
```

### 2.2 AppResult

**定义**:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

**使用方法**:

```rust
pub async fn my_function() -> AppResult<MyDto> {
    // 使用 ? 操作符传播错误
    let entity = database::find(id).await?;
    Ok(dto)
}
```

---

## 三、HTTP 响应

### 3.1 成功响应

**位置**: `src/infra/http/error_handler.rs`

**success_response**:

```rust
use crate::infra::http::error_handler::success_response;

pub async fn handle(...) -> Response {
    match logic::execute(...).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}
```

**响应格式**:

```json
{
  "data": {
    /* DTO */
  },
  "timestamp": "2025-10-16T13:00:00Z",
  "request_id": null
}
```

### 3.2 创建响应 (201)

**created_response**:

```rust
use crate::infra::http::error_handler::created_response;

Ok(dto) => created_response(dto).into_response()
```

### 3.3 无内容响应 (204)

```rust
use axum::http::StatusCode;

Ok(()) => StatusCode::NO_CONTENT.into_response()
```

---

## 四、数据库操作

### 4.1 事务助手

**位置**: `src/features/shared/transaction.rs`

**使用方法**:

```rust
use crate::features::shared::TransactionHelper;

// 开启事务
let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

// 执行操作
database::insert(&mut tx, &entity).await?;
events::write_event(&mut tx, app_state, &entity).await?;

// 提交事务（失败自动回滚）
tx.commit().await?;
```

### 4.2 查询模式

**单条记录**:

```rust
let row = sqlx::query_as::<_, EntityRow>(
    "SELECT * FROM entities WHERE id = ?"
)
.bind(id.to_string())
.fetch_optional(pool)
.await
.map_err(|e| AppError::DatabaseError(e.into()))?;
```

**多条记录**:

```rust
let rows = sqlx::query_as::<_, EntityRow>(
    "SELECT * FROM entities WHERE is_deleted = FALSE"
)
.fetch_all(pool)
.await
.map_err(|e| AppError::DatabaseError(e.into()))?;
```

**插入**:

```rust
sqlx::query(
    "INSERT INTO entities (id, title, created_at) VALUES (?, ?, ?)"
)
.bind(entity.id)
.bind(&entity.title)
.bind(&entity.created_at)
.execute(&mut **tx)
.await
.map_err(|e| AppError::DatabaseError(e.into()))?;
```

**更新**:

```rust
let result = sqlx::query(
    "UPDATE entities SET title = ? WHERE id = ?"
)
.bind(&entity.title)
.bind(entity.id)
.execute(&mut **tx)
.await
.map_err(|e| AppError::DatabaseError(e.into()))?;

if result.rows_affected() == 0 {
    return Err(AppError::NotFound { /* ... */ });
}
```

### 4.3 Repository 模式

**位置**: `src/features/shared/repositories/`

**TaskRepository 示例**:

```rust
use crate::features::shared::repositories::TaskRepository;

// 通过 ID 查找
let task = TaskRepository::find_by_id(pool, task_id).await?
    .ok_or_else(|| AppError::NotFound { /* ... */ })?;

// 查询多个
let tasks = TaskRepository::find_by_ids(pool, &task_ids).await?;
```

---

## 五、SSE 事件系统

### 5.1 Event Outbox

**位置**: `src/infra/events/outbox.rs`

**写入事件**:

```rust
use crate::infra::events::{
    models::DomainEvent,
    outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
};

pub async fn write_created_event(
    tx: &mut Transaction<'_, Sqlite>,
    app_state: &AppState,
    entity: &Entity,
) -> AppResult<()> {
    let outbox_repo = SqlxEventOutboxRepository::new(
        app_state.db_pool().clone()
    );

    // 创建事件
    let event = DomainEvent::new(
        "entity.created",              // 事件类型
        "entity",                      // 聚合类型
        entity.id.to_string(),         // 聚合 ID
        serde_json::to_value(dto).unwrap(),  // Payload
    );

    // 在事务内写入
    outbox_repo.insert_in_tx(&mut **tx, event).await?;

    Ok(())
}
```

### 5.2 DomainEvent 结构

```rust
pub struct DomainEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

impl DomainEvent {
    pub fn new(
        event_type: &str,
        aggregate_type: &str,
        aggregate_id: String,
        payload: serde_json::Value,
    ) -> Self { /* ... */ }
}
```

### 5.3 事件命名规范

**格式**: `domain.action`（小写，点分隔）

**示例**:

```
task.created
task.updated
task.deleted
schedule.created
template.created
```

---

## 六、数据类型转换

### 6.1 标准转换模式

**EntityRow → Entity**:

```rust
impl TryFrom<EntityRow> for Entity {
    type Error = String;

    fn try_from(row: EntityRow) -> Result<Self, Self::Error> {
        Ok(Entity {
            id: Uuid::parse_str(&row.id)
                .map_err(|e| e.to_string())?,
            title: row.title,
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc),
            /* ... */
        })
    }
}
```

**Entity → EntityDto**:

```rust
impl From<Entity> for EntityDto {
    fn from(entity: Entity) -> Self {
        EntityDto {
            id: entity.id,
            title: entity.title,
            created_at: entity.created_at.to_rfc3339(),
            /* ... */
        }
    }
}
```

### 6.2 常用转换函数

**UUID 转换**:

```rust
// String → Uuid
let uuid = Uuid::parse_str(&id_str)
    .map_err(|e| AppError::ValidationError { /* ... */ })?;

// Uuid → String
let id_str = uuid.to_string();
```

**时间转换**:

```rust
// String → DateTime<Utc>
let dt = DateTime::parse_from_rfc3339(&time_str)
    .map_err(|e| AppError::ValidationError { /* ... */ })?
    .with_timezone(&Utc);

// DateTime<Utc> → String
let time_str = dt.to_rfc3339();
```

**JSON 转换**:

```rust
// String → serde_json::Value
let json: serde_json::Value = serde_json::from_str(&json_str)
    .map_err(|e| AppError::ValidationError { /* ... */ })?;

// serde_json::Value → String
let json_str = serde_json::to_string(&json)
    .map_err(|e| AppError::InternalError { /* ... */ })?;
```

---

## 七、验证工具

### 7.1 基本验证

```rust
fn validate_title(title: &str) -> AppResult<()> {
    if title.trim().is_empty() {
        return Err(AppError::ValidationError {
            field: "title".to_string(),
            message: "标题不能为空".to_string(),
        });
    }

    if title.len() > 255 {
        return Err(AppError::ValidationError {
            field: "title".to_string(),
            message: "标题不能超过 255 个字符".to_string(),
        });
    }

    Ok(())
}
```

### 7.2 日期验证

```rust
fn validate_date(date_str: &str) -> AppResult<()> {
    // 验证格式 YYYY-MM-DD
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    if !re.is_match(date_str) {
        return Err(AppError::ValidationError {
            field: "date".to_string(),
            message: "日期格式必须为 YYYY-MM-DD".to_string(),
        });
    }
    Ok(())
}
```

### 7.3 UUID 验证

```rust
fn validate_uuid(id_str: &str) -> AppResult<Uuid> {
    Uuid::parse_str(id_str).map_err(|_| AppError::ValidationError {
        field: "id".to_string(),
        message: "无效的 UUID 格式".to_string(),
    })
}
```

---

## 八、共享查询函数

### 8.1 存在性检查

```rust
pub async fn entity_exists(pool: &SqlitePool, id: Uuid) -> AppResult<bool> {
    let result = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM entities WHERE id = ? AND is_deleted = FALSE)"
    )
    .bind(id.to_string())
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.into()))?;

    Ok(result)
}
```

### 8.2 计数查询

```rust
pub async fn count_entities(pool: &SqlitePool) -> AppResult<i64> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM entities WHERE is_deleted = FALSE"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.into()))?;

    Ok(count)
}
```

### 8.3 批量查询

```rust
pub async fn find_by_ids(
    pool: &SqlitePool,
    ids: &[Uuid],
) -> AppResult<Vec<Entity>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let id_strs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    let placeholders = vec!["?"; ids.len()].join(",");
    let query_str = format!(
        "SELECT * FROM entities WHERE id IN ({}) AND is_deleted = FALSE",
        placeholders
    );

    let mut query = sqlx::query_as::<_, EntityRow>(&query_str);
    for id_str in id_strs {
        query = query.bind(id_str);
    }

    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

    rows.into_iter()
        .map(|r| r.try_into().map_err(|e: String| {
            AppError::DatabaseError(DbError::ParseError(e))
        }))
        .collect()
}
```

---

## 九、日志工具

### 9.1 使用 tracing

```rust
use tracing::{debug, info, warn, error, instrument};

// Info 日志
info!("Entity created: id={}", entity.id);

// Debug 日志
debug!("Executing query: {:?}", query);

// Warning 日志
warn!("Rate limit approaching: {}/100", count);

// Error 日志
error!("Database connection failed: {}", err);
```

### 9.2 Instrument 宏

```rust
#[instrument(skip(app_state))]
pub async fn execute(
    app_state: &AppState,
    request: CreateRequest,
) -> AppResult<EntityDto> {
    // 函数执行自动记录日志
}
```

### 9.3 性能追踪

```rust
let start = std::time::Instant::now();

// 执行操作...

tracing::debug!(
    "[PERF] Operation took {:.3}ms",
    start.elapsed().as_secs_f64() * 1000.0
);
```

---

## 十、测试工具

### 10.1 测试数据库

```rust
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // 运行迁移
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();

    pool
}
```

### 10.2 测试 AppState

```rust
use crate::startup::AppState;

async fn create_test_app_state() -> AppState {
    let pool = setup_test_db().await;
    AppState::new(pool)
}
```

### 10.3 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_entity() {
        let app_state = create_test_app_state().await;

        let request = CreateRequest {
            title: "Test Entity".to_string(),
        };

        let result = logic::execute(&app_state, request).await;
        assert!(result.is_ok());

        let dto = result.unwrap();
        assert_eq!(dto.title, "Test Entity");
    }
}
```

---

## 十一、配置管理

### 11.1 环境变量

```rust
use std::env;

// 读取环境变量
let port = env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string());

// 带默认值
let db_url = env::var("DATABASE_URL")
    .unwrap_or_else(|_| "sqlite:cutie.db".to_string());
```

### 11.2 配置结构

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("Invalid PORT"),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:cutie.db".to_string()),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
        }
    }
}
```

---

## 十二、常用宏

### 12.1 自动实现 FromRow

```rust
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct EntityRow {
    pub id: String,
    pub title: String,
    pub created_at: String,
}
```

### 12.2 Serde 序列化

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDto {
    pub id: Uuid,
    pub title: String,
}
```

### 12.3 自动 Debug

```rust
#[derive(Debug)]
pub struct MyStruct {
    field: String,
}
```

---

## 十三、常见模式

### 13.1 Option 转 Result

```rust
// 将 Option 转换为 Result
entity.ok_or_else(|| AppError::NotFound {
    entity_type: "Entity".to_string(),
    entity_id: id.to_string(),
})?
```

### 13.2 多个错误类型

```rust
// 使用 map_err 转换错误类型
sqlx::query("...")
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.into()))?
```

### 13.3 条件检查

```rust
// 提前返回错误
if !entity_exists(pool, id).await? {
    return Err(AppError::NotFound { /* ... */ });
}
```

---

## 十四、数据库约定

### 14.1 字段类型

```sql
-- UUID 字符串
id TEXT PRIMARY KEY NOT NULL

-- 时间戳 (RFC 3339)
created_at TEXT NOT NULL
updated_at TEXT NOT NULL
deleted_at TEXT

-- 布尔值
is_deleted BOOLEAN NOT NULL DEFAULT FALSE

-- 整数
estimated_duration INTEGER

-- JSON
metadata TEXT
subtasks TEXT
```

### 14.2 索引规范

```sql
-- 常用查询字段
CREATE INDEX idx_entities_updated_at ON entities(updated_at);

-- 软删除标志
CREATE INDEX idx_entities_is_deleted ON entities(is_deleted);

-- 外键
CREATE INDEX idx_entities_area_id ON entities(area_id);

-- 唯一约束
CREATE UNIQUE INDEX idx_entities_name ON entities(name);
```

### 14.3 命名规范

- 表名：复数形式 (`entities`, `tasks`)
- 索引：`idx_{table}_{column}`
- 外键：`{referenced_table}_id`
- 布尔字段：`is_{condition}`, `has_{property}`

---

## 十五、开发检查清单

**端点开发**:

- [ ] CABC 文档完整
- [ ] 使用 `success_response` 包装
- [ ] 获取写入许可
- [ ] 在事务内操作
- [ ] 写入 Event Outbox
- [ ] 正确的错误处理
- [ ] 添加日志

**数据库操作**:

- [ ] 使用参数化查询（防止 SQL 注入）
- [ ] 正确的错误转换
- [ ] 软删除而非硬删除
- [ ] 更新 updated_at 字段

**类型转换**:

- [ ] Row → Entity (TryFrom)
- [ ] Entity → Dto (From)
- [ ] UUID 和时间正确转换

**测试**:

- [ ] 单元测试通过
- [ ] 集成测试通过
- [ ] `cargo check` 无警告
- [ ] `cargo clippy` 无警告

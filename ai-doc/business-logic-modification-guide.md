# Cutie 后端业务逻辑新增/变更指南

## 概述

本文档提供了在Cutie后端系统中新增或修改业务逻辑的完整指南。遵循本指南可以确保新功能与现有架构保持一致，并维持代码质量标准。

## 修改原则

### 1. 分层修改原则

- **自下而上**: 从数据层开始，逐层向上修改
- **接口优先**: 先定义接口，再实现具体逻辑
- **测试驱动**: 每层修改都必须有对应的测试

### 2. 向后兼容原则

- **API兼容**: 新版本API必须向后兼容
- **数据兼容**: 数据库变更必须支持数据迁移
- **配置兼容**: 配置变更必须有合理的默认值

### 3. 文档同步原则

- **CABC更新**: 所有业务逻辑变更必须更新CABC文档
- **API文档**: HTTP API变更必须更新OpenAPI规范
- **架构文档**: 架构变更必须更新设计文档

## 新增业务功能流程

### Step 1: 需求分析和设计

#### 1.1 业务需求梳理

```markdown
## 功能需求模板

- **功能名称**: [功能的简短描述]
- **业务价值**: [为用户带来的价值]
- **用户场景**: [具体的使用场景]
- **输入输出**: [数据输入和输出规范]
- **边界条件**: [异常情况和边界处理]
- **性能要求**: [响应时间和吞吐量要求]
```

#### 1.2 影响分析

- **数据模型影响**: 是否需要新增或修改数据表
- **API影响**: 是否需要新增或修改API端点
- **业务逻辑影响**: 影响哪些现有的业务服务
- **前端影响**: 对前端的影响和集成要求

#### 1.3 技术设计

- **架构设计**: 确定涉及的架构层级
- **接口设计**: 定义新的接口和数据结构
- **实现策略**: 确定实现方案和技术选型

### Step 2: 数据层修改（如需要）

#### 2.1 数据库Schema变更

```sql
-- 新增表的模板
CREATE TABLE new_entity (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    -- 业务字段
    field1 TEXT,
    field2 INTEGER,
    -- 关联字段
    parent_id TEXT,
    area_id TEXT,
    -- 审计字段
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    -- 外键约束
    FOREIGN KEY (parent_id) REFERENCES parent_table(id),
    FOREIGN KEY (area_id) REFERENCES areas(id)
);

-- 索引创建
CREATE INDEX idx_new_entity_updated_at ON new_entity(updated_at);
CREATE INDEX idx_new_entity_is_deleted ON new_entity(is_deleted);
CREATE INDEX idx_new_entity_parent_id ON new_entity(parent_id);
```

#### 2.2 迁移脚本创建

```bash
# 创建新的迁移文件
touch migrations/$(date +%Y%m%d%H%M%S)_add_new_feature.sql
```

#### 2.3 数据模型定义

```rust
// src/core/models/new_entity.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewEntity {
    pub id: Uuid,
    pub name: String,
    pub field1: Option<String>,
    pub field2: Option<i32>,
    pub parent_id: Option<Uuid>,
    pub area_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl NewEntity {
    pub fn new(
        id: Uuid,
        name: String,
        field1: Option<String>,
        field2: Option<i32>,
        parent_id: Option<Uuid>,
        area_id: Option<Uuid>,
        now: DateTime<Utc>,
    ) -> Result<Self, String> {
        // 验证逻辑
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if name.len() > 255 {
            return Err("Name too long".to_string());
        }

        Ok(Self {
            id,
            name,
            field1,
            field2,
            parent_id,
            area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        })
    }

    pub fn update_name(&mut self, new_name: String, now: DateTime<Utc>) -> Result<(), String> {
        if new_name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        self.name = new_name;
        self.updated_at = now;
        Ok(())
    }
}
```

### Step 3: 仓库层修改

#### 3.1 仓库接口定义

```rust
// src/repositories/new_entity_repository.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Transaction;
use crate::common::error::DbError;
use crate::core::models::NewEntity;

#[async_trait]
pub trait NewEntityRepository: Send + Sync {
    /// 创建新实体
    async fn create(&self, tx: &mut Transaction<'_>, entity: &NewEntity) -> Result<NewEntity, DbError>;

    /// 根据ID查找
    async fn find_by_id(&self, id: Uuid) -> Result<Option<NewEntity>, DbError>;

    /// 更新实体
    async fn update(&self, tx: &mut Transaction<'_>, entity: &NewEntity) -> Result<NewEntity, DbError>;

    /// 软删除
    async fn soft_delete(&self, tx: &mut Transaction<'_>, id: Uuid) -> Result<(), DbError>;

    /// 查找所有实体
    async fn find_all(&self) -> Result<Vec<NewEntity>, DbError>;

    /// 根据父ID查找
    async fn find_by_parent(&self, parent_id: Uuid) -> Result<Vec<NewEntity>, DbError>;

    /// 搜索实体
    async fn search(&self, query: &str, limit: Option<i64>) -> Result<Vec<NewEntity>, DbError>;
}
```

#### 3.2 SQLx实现

```rust
// src/repositories/sqlx_new_entity_repository.rs
use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::{NewEntityRepository, Transaction};
use crate::common::error::DbError;
use crate::core::models::NewEntity;

pub struct SqlxNewEntityRepository {
    pool: Arc<SqlitePool>,
}

impl SqlxNewEntityRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NewEntityRepository for SqlxNewEntityRepository {
    async fn create(&self, tx: &mut Transaction<'_>, entity: &NewEntity) -> Result<NewEntity, DbError> {
        let query = r#"
            INSERT INTO new_entities (
                id, name, field1, field2, parent_id, area_id,
                created_at, updated_at, is_deleted
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(entity.id.to_string())
            .bind(&entity.name)
            .bind(&entity.field1)
            .bind(entity.field2)
            .bind(entity.parent_id.map(|id| id.to_string()))
            .bind(entity.area_id.map(|id| id.to_string()))
            .bind(entity.created_at.to_rfc3339())
            .bind(entity.updated_at.to_rfc3339())
            .bind(entity.is_deleted)
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::SqlxError)?;

        Ok(row_to_new_entity(row)?)
    }

    // 其他方法实现...
}

fn row_to_new_entity(row: sqlx::sqlite::SqliteRow) -> Result<NewEntity, DbError> {
    // 行到实体的转换逻辑
}
```

#### 3.3 内存测试实现

```rust
// 在 src/repositories/memory_repositories.rs 中添加
impl NewEntityRepository for MemoryRepositories {
    async fn create(&self, _tx: &mut Transaction<'_>, entity: &NewEntity) -> Result<NewEntity, DbError> {
        let mut entities = self.new_entities.lock().await;

        if entities.contains_key(&entity.id) {
            return Err(DbError::ConstraintViolation("Entity already exists".to_string()));
        }

        entities.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    // 其他方法实现...
}
```

### Step 4: 服务层修改

#### 4.1 DTO定义

```rust
// 在 src/services/dtos.rs 中添加
#[derive(Debug, Clone)]
pub struct CreateNewEntityData {
    pub name: String,
    pub field1: Option<String>,
    pub field2: Option<i32>,
    pub parent_id: Option<Uuid>,
    pub area_id: Option<Uuid>,
}

impl CreateNewEntityData {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push(ValidationError::new("name", "Name cannot be empty", "NAME_EMPTY"));
        }

        if self.name.len() > 255 {
            errors.push(ValidationError::new("name", "Name too long", "NAME_TOO_LONG"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

#### 4.2 服务实现

```rust
// src/services/new_entity_service.rs
use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::{AppError, AppResult};
use crate::core::models::NewEntity;
use crate::ports::{Clock, IdGenerator};
use crate::repositories::NewEntityRepository;
use super::dtos::{CreateNewEntityData, UpdateNewEntityData};

pub struct NewEntityService {
    repository: Arc<dyn NewEntityRepository>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl NewEntityService {
    pub fn new(
        repository: Arc<dyn NewEntityRepository>,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            repository,
            clock,
            id_generator,
        }
    }

    /// 创建新实体
    ///
    /// **CABC文档**:
    /// - **函数签名**: `pub async fn create_entity(&self, data: CreateNewEntityData) -> AppResult<NewEntity>`
    /// - **预期行为简介**: 创建一个新的实体，并进行完整的验证
    /// - **输入输出规范**:
    ///   - **前置条件**: data.name不能为空且长度小于256
    ///   - **后置条件**: 成功时返回新创建的NewEntity对象
    /// - **边界情况**:
    ///   - data验证失败: 返回AppError::ValidationFailed
    ///   - parent_id不存在: 返回AppError::NotFound
    /// - **预期副作用**: 向new_entities表插入一条记录
    pub async fn create_entity(&self, data: CreateNewEntityData) -> AppResult<NewEntity> {
        // 1. 验证输入
        data.validate().map_err(AppError::ValidationFailed)?;

        // 2. 生成ID和时间戳
        let id = self.id_generator.new_uuid();
        let now = self.clock.now_utc();

        // 3. 创建实体对象
        let entity = NewEntity::new(
            id,
            data.name,
            data.field1,
            data.field2,
            data.parent_id,
            data.area_id,
            now,
        ).map_err(AppError::StringError)?;

        // 4. 启动事务并持久化
        let mut tx = self.repository.begin_transaction().await?;

        // 5. 验证关联实体存在（如果有）
        if let Some(parent_id) = data.parent_id {
            if self.repository.find_by_id(parent_id).await?.is_none() {
                return Err(AppError::not_found("Parent", parent_id.to_string()));
            }
        }

        // 6. 创建实体
        let created_entity = self.repository.create(&mut tx, &entity).await?;

        // 7. 提交事务
        tx.commit().await?;

        Ok(created_entity)
    }

    // 其他方法的CABC文档和实现...
}
```

### Step 5: 网络层修改

#### 5.1 HTTP载荷定义

```rust
// 在 src/handlers/payloads.rs 中添加
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNewEntityPayload {
    pub name: String,
    pub field1: Option<String>,
    pub field2: Option<i32>,
    pub parent_id: Option<Uuid>,
    pub area_id: Option<Uuid>,
}

impl From<CreateNewEntityPayload> for crate::services::CreateNewEntityData {
    fn from(payload: CreateNewEntityPayload) -> Self {
        Self {
            name: payload.name,
            field1: payload.field1,
            field2: payload.field2,
            parent_id: payload.parent_id,
            area_id: payload.area_id,
        }
    }
}
```

#### 5.2 HTTP处理器实现

```rust
// src/handlers/new_entity_handlers.rs
use axum::{
    extract::{Path, Query, State},
    response::Json,
};

use crate::startup::AppState;
use crate::common::error::AppError;
use super::{
    payloads::CreateNewEntityPayload,
    error_handler::{success_response, created_response, no_content_response},
};

/// 创建实体处理器
///
/// **端点**: `POST /new-entities`
pub async fn create_new_entity_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateNewEntityPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Creating new entity with name: {}", payload.name);

    let create_data = crate::services::CreateNewEntityData::from(payload);
    let created_entity = app_state.new_entity_service.create_entity(create_data).await?;

    log::info!("New entity created successfully: {}", created_entity.id);

    Ok(created_response(created_entity))
}

// 其他处理器实现...
```

#### 5.3 路由配置

```rust
// src/routes/new_entity_routes.rs
use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::startup::AppState;
use crate::handlers::new_entity_handlers::*;

pub fn create_new_entity_routes() -> Router<AppState> {
    Router::new()
        .route("/new-entities", post(create_new_entity_handler))
        .route("/new-entities", get(get_new_entities_handler))
        .route("/new-entities/:id", get(get_new_entity_handler))
        .route("/new-entities/:id", put(update_new_entity_handler))
        .route("/new-entities/:id", delete(delete_new_entity_handler))
}
```

### Step 6: 依赖注入更新

#### 6.1 AppState扩展

```rust
// 在 src/startup/app_state.rs 中添加
pub struct AppState {
    // 现有字段...
    pub new_entity_service: Arc<NewEntityService>,
}

impl AppState {
    pub async fn new_production(config: AppConfig) -> Result<Self, AppError> {
        // 现有初始化...

        // 创建新实体仓库
        let new_entity_repository = Arc::new(SqlxNewEntityRepository::new(db_pool.clone()));

        // 创建新实体服务
        let new_entity_service = Arc::new(NewEntityService::new(
            new_entity_repository,
            clock.clone(),
            id_generator.clone(),
        ));

        Ok(Self {
            // 现有字段...
            new_entity_service,
        })
    }
}
```

#### 6.2 模块导出更新

```rust
// 更新各个mod.rs文件
// src/core/models/mod.rs
pub mod new_entity;
pub use new_entity::*;

// src/repositories/mod.rs
pub mod new_entity_repository;
pub use new_entity_repository::*;

// src/services/mod.rs
pub mod new_entity_service;
pub use new_entity_service::*;

// src/handlers/mod.rs
pub mod new_entity_handlers;
pub use new_entity_handlers::*;

// src/routes/mod.rs
pub mod new_entity_routes;
pub use new_entity_routes::*;
```

## 修改现有业务逻辑流程

### Step 1: 影响分析

#### 1.1 确定修改范围

- **服务层**: 哪些服务需要修改
- **仓库层**: 哪些仓库接口需要扩展
- **数据层**: 是否需要数据库变更
- **API层**: 哪些端点需要修改

#### 1.2 向后兼容性检查

- **API兼容性**: 现有API是否会被破坏
- **数据兼容性**: 现有数据是否需要迁移
- **行为兼容性**: 现有行为是否会改变

### Step 2: 测试先行

#### 2.1 编写失败测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_business_logic() {
        // 准备测试环境
        let service = create_test_service().await;

        // 测试新的业务逻辑
        let result = service.new_method(test_data).await;

        // 验证期望的行为
        assert!(result.is_ok());
        let entity = result.unwrap();
        assert_eq!(entity.new_field, expected_value);
    }

    #[tokio::test]
    async fn test_edge_case_handling() {
        // 测试边界情况
    }
}
```

#### 2.2 运行测试确认失败

```bash
cargo test test_new_business_logic
# 应该失败，因为功能还未实现
```

### Step 3: 实现修改

#### 3.1 数据层修改（如需要）

```sql
-- 添加新字段的迁移
ALTER TABLE existing_table ADD COLUMN new_field TEXT;
CREATE INDEX idx_existing_table_new_field ON existing_table(new_field);
```

#### 3.2 仓库层修改

```rust
// 在现有仓库接口中添加新方法
#[async_trait]
pub trait ExistingRepository: Send + Sync {
    // 现有方法...

    /// 新增的仓库方法
    async fn new_method(&self, param: Type) -> Result<ReturnType, DbError>;
}

// 在SQLx实现中添加具体实现
impl ExistingRepository for SqlxExistingRepository {
    async fn new_method(&self, param: Type) -> Result<ReturnType, DbError> {
        // 实现逻辑
    }
}

// 在内存实现中添加测试实现
impl ExistingRepository for MemoryRepositories {
    async fn new_method(&self, param: Type) -> Result<ReturnType, DbError> {
        // 测试实现
    }
}
```

#### 3.3 服务层修改

```rust
// 在现有服务中添加新方法
impl ExistingService {
    /// 新的业务方法
    ///
    /// **CABC文档**:
    /// - **函数签名**: `pub async fn new_method(&self, input: InputType) -> AppResult<OutputType>`
    /// - **预期行为简介**: [详细描述业务行为]
    /// - **输入输出规范**: [输入输出的详细规范]
    /// - **边界情况**: [所有边界情况的处理]
    /// - **预期副作用**: [数据库操作和其他副作用]
    pub async fn new_method(&self, input: InputType) -> AppResult<OutputType> {
        // 1. 输入验证
        input.validate().map_err(AppError::ValidationFailed)?;

        // 2. 业务逻辑实现
        let mut tx = self.repository.begin_transaction().await?;

        // 3. 数据操作
        let result = self.repository.new_method(&mut tx, input).await?;

        // 4. 提交事务
        tx.commit().await?;

        Ok(result)
    }
}
```

#### 3.4 网络层修改

```rust
// 添加新的HTTP处理器
pub async fn new_endpoint_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<NewPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Processing new endpoint request");

    let result = app_state.existing_service.new_method(payload.into()).await?;

    Ok(success_response(result))
}

// 更新路由配置
pub fn create_existing_routes() -> Router<AppState> {
    Router::new()
        // 现有路由...
        .route("/existing/:id/new-action", post(new_endpoint_handler))
}
```

### Step 4: 测试验证

#### 4.1 运行单元测试

```bash
cargo test
# 所有测试应该通过
```

#### 4.2 集成测试

```bash
cargo test --test integration_tests
```

#### 4.3 API测试

```bash
# 使用HTTP客户端测试新端点
curl -X POST http://localhost:8080/api/new-entities \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Entity"}'
```

## 常见修改场景

### 场景1: 新增字段到现有实体

#### 必须修改的文件

1. **数据库迁移**:

   ```sql
   -- migrations/YYYYMMDDHHMMSS_add_field_to_entity.sql
   ALTER TABLE entities ADD COLUMN new_field TEXT;
   ```

2. **数据模型**:

   ```rust
   // src/core/models/entity.rs
   pub struct Entity {
       // 现有字段...
       pub new_field: Option<String>, // 新字段
   }
   ```

3. **仓库实现**:

   ```rust
   // src/repositories/sqlx_entity_repository.rs
   // 更新所有SQL查询，包含新字段
   ```

4. **服务DTO**:

   ```rust
   // src/services/dtos.rs
   pub struct UpdateEntityData {
       // 现有字段...
       pub new_field: Option<Option<String>>, // 新字段
   }
   ```

5. **HTTP载荷**:

   ```rust
   // src/handlers/payloads.rs
   pub struct UpdateEntityPayload {
       // 现有字段...
       pub new_field: Option<Option<String>>, // 新字段
   }
   ```

6. **测试更新**:
   - 所有相关的单元测试
   - 集成测试
   - API测试

### 场景2: 新增业务规则

#### 必须修改的文件

1. **服务层**:

   ```rust
   // src/services/entity_service.rs
   impl EntityService {
       pub async fn existing_method(&self, input: Input) -> AppResult<Output> {
           // 1. 现有验证...

           // 2. 新增的业务规则验证
           if !self.validate_new_rule(&input).await? {
               return Err(AppError::validation_error("field", "New rule violation", "NEW_RULE_FAILED"));
           }

           // 3. 现有逻辑...
       }

       async fn validate_new_rule(&self, input: &Input) -> AppResult<bool> {
           // 新规则的验证逻辑
       }
   }
   ```

2. **测试更新**:

   ```rust
   #[tokio::test]
   async fn test_new_business_rule() {
       // 测试新规则的各种情况
   }
   ```

3. **文档更新**:
   - 更新CABC文档
   - 更新API文档

### 场景3: 新增API端点

#### 必须修改的文件

1. **HTTP处理器**:

   ```rust
   // src/handlers/entity_handlers.rs
   pub async fn new_endpoint_handler(
       State(app_state): State<AppState>,
       Path(id): Path<Uuid>,
   ) -> Result<impl axum::response::IntoResponse, AppError> {
       // 处理器实现
   }
   ```

2. **路由配置**:

   ```rust
   // src/routes/entity_routes.rs
   pub fn create_entity_routes() -> Router<AppState> {
       Router::new()
           // 现有路由...
           .route("/entities/:id/new-action", post(new_endpoint_handler))
   }
   ```

3. **API文档**:
   ```rust
   // 更新 src/routes/api_router.rs 中的端点统计
   ```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::create_test_service;

    #[tokio::test]
    async fn test_business_logic() {
        let service = create_test_service().await;
        let result = service.method(input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validation_failure() {
        let service = create_test_service().await;
        let result = service.method(invalid_input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::ValidationFailed(_)));
    }

    #[tokio::test]
    async fn test_edge_cases() {
        // 边界情况测试
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_flow() {
        // 端到端流程测试
    }
}
```

### API测试

```bash
# 创建测试脚本
#!/bin/bash
# test_new_api.sh

echo "Testing new API endpoint..."

# 测试创建
RESPONSE=$(curl -s -X POST http://localhost:8080/api/new-entities \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Entity"}')

echo "Create response: $RESPONSE"

# 提取ID并测试获取
ID=$(echo $RESPONSE | jq -r '.data.id')
curl -s "http://localhost:8080/api/new-entities/$ID"
```

## 错误处理扩展

### 新增错误类型

```rust
// 在 src/common/error.rs 中添加
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // 现有错误类型...

    /// 新的业务错误
    #[error("Business rule violation: {message}")]
    BusinessRuleViolation { message: String },

    /// 资源限制错误
    #[error("Resource limit exceeded: {resource_type}")]
    ResourceLimitExceeded { resource_type: String },
}

impl AppError {
    pub fn business_rule_violation(message: impl Into<String>) -> Self {
        Self::BusinessRuleViolation {
            message: message.into(),
        }
    }

    pub fn resource_limit_exceeded(resource_type: impl Into<String>) -> Self {
        Self::ResourceLimitExceeded {
            resource_type: resource_type.into(),
        }
    }
}
```

### HTTP错误映射扩展

```rust
// 在 src/handlers/error_handler.rs 中添加
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_response) = match self {
            // 现有映射...

            AppError::BusinessRuleViolation { message } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorResponse::new("BusinessRuleViolation".to_string(), message)
            ),

            AppError::ResourceLimitExceeded { resource_type } => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorResponse::new(
                    "ResourceLimitExceeded".to_string(),
                    format!("Resource limit exceeded: {}", resource_type)
                )
            ),
        };

        (status_code, Json(error_response)).into_response()
    }
}
```

## 性能优化指南

### 数据库查询优化

```rust
// 添加新的索引
CREATE INDEX idx_table_new_query ON table(field1, field2);

// 优化查询语句
async fn optimized_query(&self) -> Result<Vec<Entity>, DbError> {
    let query = r#"
        SELECT * FROM entities
        WHERE field1 = ?1
        AND field2 > ?2
        ORDER BY created_at DESC
        LIMIT ?3
    "#;
    // 实现...
}
```

### 缓存策略

```rust
// 添加缓存层
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct CachedEntityService {
    inner: Arc<EntityService>,
    cache: Arc<RwLock<HashMap<Uuid, Entity>>>,
}

impl CachedEntityService {
    pub async fn get_entity(&self, id: Uuid) -> AppResult<Option<Entity>> {
        // 先检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(entity) = cache.get(&id) {
                return Ok(Some(entity.clone()));
            }
        }

        // 缓存未命中，查询数据库
        let entity = self.inner.get_entity(id).await?;

        // 更新缓存
        if let Some(ref entity) = entity {
            let mut cache = self.cache.write().await;
            cache.insert(id, entity.clone());
        }

        Ok(entity)
    }
}
```

## 版本管理

### CABC版本控制

```rust
/// 版本化的业务方法
///
/// **CABC v1.1** (变更说明: 添加了新的验证规则)
/// - **函数签名**: `pub async fn method_v1_1(&self, input: Input) -> AppResult<Output>`
/// - **变更内容**:
///   - 添加了新的输入验证
///   - 增强了错误处理
///   - 保持向后兼容
pub async fn method(&self, input: Input) -> AppResult<Output> {
    // v1.1 实现
}
```

### API版本控制

```rust
// 版本化的API路由
pub fn create_versioned_routes() -> Router<AppState> {
    Router::new()
        .nest("/v1", create_v1_routes())
        .nest("/v2", create_v2_routes())
}
```

## 监控和日志

### 业务指标监控

```rust
// 添加业务指标收集
impl EntityService {
    pub async fn create_entity(&self, data: CreateEntityData) -> AppResult<Entity> {
        let start_time = std::time::Instant::now();

        let result = self.create_entity_impl(data).await;

        let duration = start_time.elapsed();
        log::info!("Entity creation took {:?}", duration);

        // 记录业务指标
        if duration > std::time::Duration::from_millis(100) {
            log::warn!("Slow entity creation: {:?}", duration);
        }

        result
    }
}
```

### 错误监控

```rust
// 添加错误统计
impl AppError {
    pub fn log_error(&self) {
        match self {
            AppError::DatabaseError(e) => {
                log::error!("Database error occurred: {}", e);
                // 发送到监控系统
            }
            AppError::ValidationFailed(errors) => {
                log::warn!("Validation failed: {} errors", errors.len());
            }
            _ => {}
        }
    }
}
```

## 部署和运维

### 配置管理

```toml
# config/production.toml
[app]
environment = "production"
log_level = "info"

[database]
max_connections = 50
connection_timeout = 30

[server]
port = 8080
cors_enabled = true
```

### 健康检查

```rust
// 扩展健康检查
impl AppState {
    pub async fn health_check(&self) -> Result<HealthStatus, AppError> {
        let mut details = HashMap::new();

        // 数据库健康检查
        match self.check_database_health().await {
            Ok(_) => details.insert("database".to_string(), "healthy".to_string()),
            Err(e) => details.insert("database".to_string(), format!("unhealthy: {}", e)),
        };

        // 新服务健康检查
        match self.new_service.health_check().await {
            Ok(_) => details.insert("new_service".to_string(), "healthy".to_string()),
            Err(e) => details.insert("new_service".to_string(), format!("unhealthy: {}", e)),
        };

        // 确定整体状态
        let overall = if details.values().all(|v| v == "healthy") {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        Ok(HealthStatus { overall, details })
    }
}
```

## 总结

本指南提供了在Cutie后端系统中新增或修改业务逻辑的完整流程。关键要点：

1. **遵循分层架构**: 自下而上的修改顺序
2. **测试先行**: 先写测试，再实现功能
3. **文档同步**: 代码和文档必须同步更新
4. **向后兼容**: 保持API和数据的向后兼容性
5. **质量保证**: 每个修改都必须通过完整的测试验证

通过遵循这些原则和流程，可以确保系统的持续健康发展，并维持高质量的代码标准。

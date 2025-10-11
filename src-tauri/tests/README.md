# Cutie 测试套件

测试代码与业务代码完全分离，组织在独立的 `tests/` 目录中。

## 📁 目录结构

```
tests/
├── lib.rs                      # 测试入口文件
├── infrastructure/             # 测试基础设施
│   ├── mod.rs                 # 导出所有测试工具
│   ├── database.rs            # TestDb 和 create_test_db
│   ├── fixtures.rs            # 测试数据构造器 (TaskFixture, AreaFixture)
│   ├── http_client.rs         # TestClient HTTP 客户端
│   └── test_helpers.rs        # 辅助函数 (create_test_app_state)
├── unit/                       # 单元测试
│   ├── mod.rs
│   └── task_repository_tests.rs   # TaskRepository CRUD 测试
├── endpoint/                   # 端点测试（单个 HTTP 端点）
│   ├── mod.rs
│   └── create_task_tests.rs   # POST /tasks 端点测试
└── integration/                # 业务集成测试（多端点协同）
    ├── mod.rs
    └── task_lifecycle_tests.rs # 任务完整生命周期测试
```

## 🎯 测试分类

### 1. **基础设施测试** (`infrastructure/`)

测试工具和辅助函数，供其他测试使用：

- `TestDb`: 自动创建临时 SQLite 数据库
- `TaskFixture/AreaFixture`: 快速构建测试数据
- `TestClient`: 简化 HTTP 请求
- `create_test_app_state`: 创建测试用 AppState

### 2. **单元测试** (`unit/`)

测试单个组件（Repository、Assembler 等）的逻辑：

- ✅ `test_insert_and_find_by_id` - 插入和查询
- ✅ `test_update_task` - 更新任务
- ✅ `test_delete_task` - 软删除
- ✅ `test_list_non_deleted_tasks` - 查询未删除任务

**特点**：

- 直接测试数据访问层
- 使用 `TransactionHelper` 手动管理事务
- 不涉及 HTTP 层

### 3. **端点测试** (`endpoint/`)

测试单个 HTTP 端点的行为（请求/响应）：

- ✅ `test_create_task_success` - 创建任务成功
- ✅ `test_create_task_validation_error` - 空标题验证
- ✅ `test_create_task_with_long_title_error` - 超长标题验证

**特点**：

- 测试 HTTP 接口层
- 验证状态码、响应格式
- 验证输入验证逻辑

### 4. **集成测试** (`integration/`)

测试多个端点协同工作的完整业务流程：

- ✅ `test_task_lifecycle_create_update_complete` - 创建→更新→完成
- ✅ `test_task_deletion_workflow` - 删除工作流
- ✅ `test_multiple_tasks_creation_and_retrieval` - 批量创建

**特点**：

- 测试端到端业务逻辑
- 验证数据持久化
- 验证状态转换

## 🚀 运行测试

```bash
# 运行所有测试
cargo test --tests

# 运行单元测试
cargo test --test lib unit::

# 运行端点测试
cargo test --test lib endpoint::

# 运行集成测试
cargo test --test lib integration::

# 运行特定测试
cargo test --test lib test_task_lifecycle

# 显示测试输出
cargo test --test lib -- --nocapture
```

## 📝 编写新测试

### 单元测试模板

```rust
use explore_lib::features::shared::TransactionHelper;
use explore_lib::features::tasks::shared::repositories::TaskRepository;

mod infrastructure {
    pub use crate::infrastructure::*;
}
use infrastructure::{create_test_db, TaskFixture};

#[tokio::test]
async fn test_your_repository_function() {
    // Arrange
    let test_db = create_test_db().await.unwrap();
    let task = TaskFixture::new().title("Test").build();

    // Act
    let mut tx = TransactionHelper::begin(test_db.pool()).await.unwrap();
    TaskRepository::insert_in_tx(&mut tx, &task).await.unwrap();
    TransactionHelper::commit(tx).await.unwrap();

    // Assert
    let found = TaskRepository::find_by_id(test_db.pool(), task.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.title, "Test");
}
```

### 端点测试模板

```rust
use axum::{http::StatusCode, Router};
use explore_lib::{entities::CreateTaskRequest, features::tasks};

mod infrastructure {
    pub use crate::infrastructure::*;
}
use infrastructure::{create_test_app_state, create_test_db, TestClient};

#[tokio::test]
async fn test_your_endpoint() {
    // Arrange
    let test_db = create_test_db().await.unwrap();
    let app_state = create_test_app_state(test_db.pool().clone());
    let router = Router::new()
        .nest("/tasks", tasks::create_routes())
        .with_state(app_state);
    let client = TestClient::new(router);

    let request = CreateTaskRequest { /* ... */ };

    // Act
    let response = client.post("/tasks", &request).await;

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);
    let body: serde_json::Value = response.json().await;
    assert_eq!(body["data"]["title"], "Test Task");
}
```

### 集成测试模板

```rust
use axum::{http::StatusCode, Router};
use explore_lib::{entities::CreateTaskRequest, features::tasks};

mod infrastructure {
    pub use crate::infrastructure::*;
}
use infrastructure::{create_test_app_state, create_test_db, TestClient};

#[tokio::test]
async fn test_your_business_workflow() {
    // Arrange
    let test_db = create_test_db().await.unwrap();
    let app_state = create_test_app_state(test_db.pool().clone());
    let router = Router::new()
        .nest("/tasks", tasks::create_routes())
        .with_state(app_state);
    let client = TestClient::new(router);

    // Step 1: Create
    let create_response = client.post("/tasks", &request).await;
    let task_id = create_response.json().await["data"]["id"].as_str().unwrap();

    // Step 2: Update
    let update_response = client.patch(&format!("/tasks/{}", task_id), &update_req).await;

    // Step 3: Verify
    assert_eq!(update_response.status(), StatusCode::OK);
}
```

## ✅ 测试统计

- **单元测试**: 4 个 ✅
- **端点测试**: 3 个 ✅
- **集成测试**: 3 个 ✅
- **总计**: 10 个自定义测试 + 82 个库测试 = **92 个测试全部通过**

## 🎯 测试覆盖目标

### 已完成

- ✅ TaskRepository CRUD 操作
- ✅ 任务创建端点验证
- ✅ 任务完整生命周期

### 待扩展

- [ ] ScheduleRepository 测试
- [ ] TimeBlockRepository 测试
- [ ] 更多端点测试（更新、删除、查询）
- [ ] 并发测试
- [ ] 性能基准测试

## 📚 参考资料

- [Rust 测试最佳实践](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio 异步测试](https://tokio.rs/tokio/topics/testing)
- [Axum 测试指南](https://docs.rs/axum/latest/axum/testing/index.html)

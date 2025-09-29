# Cutie 后端架构重构

## 概述

本次重构将后端项目从分层架构（Layered Architecture）改为功能切片架构（Slicing by Feature），以提高代码的内聚性和可维护性。

## 新的项目结构

```
src/
├── shared/              # 共享模块
│   ├── core/           # 核心领域模型、错误类型、工具函数
│   ├── database/       # 数据库连接、通用仓库trait
│   └── http/           # HTTP中间件、错误处理、通用响应
├── features/           # 功能模块
│   ├── tasks/          # 任务管理功能
│   ├── schedules/      # 日程管理功能
│   ├── time_blocks/    # 时间块管理功能
│   ├── templates/      # 模板系统功能
│   ├── areas/          # 领域管理功能
│   └── ordering/       # 排序管理功能
└── [旧代码保留用于参考]
```

## 架构原则

### 1. 功能切片（Feature Slicing）

每个功能模块都包含：

- `repository.rs` - 数据访问层
- `service.rs` - 业务逻辑层
- `handlers.rs` - HTTP处理器
- `payloads.rs` - 请求/响应载荷
- `mod.rs` - 模块配置和路由

### 2. 共享模块（Shared Modules）

#### shared/core

- **models/**: 核心领域实体（Task, Area, TimeBlock等）
- **error.rs**: 统一错误类型和处理
- **utils/**: 通用工具函数（排序、时间、模板等）

#### shared/database

- **connection.rs**: 数据库连接管理
- **traits.rs**: 通用仓库trait定义
- **pagination.rs**: 分页支持

#### shared/http

- **middleware.rs**: HTTP中间件
- **error_handler.rs**: 错误处理
- **responses.rs**: 通用响应结构
- **extractors.rs**: 请求提取器

### 3. 依赖规则

- ✅ `features` 可以依赖 `shared`
- ✅ `shared` 内部模块可以互相依赖
- ❌ `features` 之间**绝对不能**互相依赖
- ❌ `shared` 不能依赖 `features`

## 功能模块示例：Tasks

### 文件结构

```
features/tasks/
├── mod.rs          # 模块配置和路由创建
├── repository.rs   # 任务数据访问层
├── service.rs      # 任务业务逻辑层
├── handlers.rs     # 任务HTTP处理器
└── payloads.rs     # 任务请求/响应载荷
```

### 使用方式

```rust
// 在主应用中组装功能模块
let task_routes = features::tasks::create_routes(pool.clone());
let app = Router::new().nest("/api", task_routes);
```

## 优势

### 1. 高内聚，低耦合

- 每个功能的所有代码都在一个模块内
- 功能之间完全解耦，便于独立开发和测试

### 2. 单文件组件模式

- 类似Vue单文件组件的理念
- 一个API的所有代码都在一个地方
- 便于理解和维护

### 3. 可复用的共享层

- 数据库连接、错误处理、中间件等通用组件
- 避免代码重复，提供一致的基础设施

### 4. 易于扩展

- 新增功能只需添加新的feature模块
- 不影响现有功能的代码

## 迁移计划

1. ✅ 创建新的项目结构
2. ✅ 实现shared模块
3. ✅ 实现tasks功能模块
4. 🔄 逐步迁移其他功能模块
5. 📋 更新主应用文件
6. 📋 清理旧代码

## 测试策略

每个功能模块都包含：

- 单元测试（service层业务逻辑）
- 集成测试（repository层数据访问）
- HTTP测试（handlers层接口）

## 开发指南

### 添加新功能

1. 在`features/`下创建新目录
2. 按照模板创建必要文件
3. 实现业务逻辑
4. 在主路由中注册

### 修改现有功能

1. 定位到对应的feature模块
2. 在该模块内进行所有修改
3. 确保不影响其他模块

### 共享代码

1. 通用逻辑放在`shared/`模块
2. 确保不依赖具体的业务逻辑
3. 提供清晰的接口和文档

## 注意事项

1. **严格遵循依赖规则**：功能模块之间绝对不能互相依赖
2. **保持接口稳定**：shared模块的接口变更要谨慎
3. **测试覆盖**：每个功能模块都要有充分的测试
4. **文档同步**：及时更新相关文档和注释

## 未来规划

1. 考虑引入领域事件（Domain Events）进行模块间通信
2. 实现插件化架构，支持功能模块的动态加载
3. 优化构建过程，支持按功能模块的增量编译


# Cutie 后端开发报告

## 1. 概述

本文档旨在详细记录 Cutie 应用后端部分的开发过程、设计思路和最终实现。后端基于 Rust 和 Tauri 框架构建，负责所有业务逻辑、数据持久化和与前端的交互。开发严格遵循了 [`reference/overall.md`](reference/overall.md) 中定义的心智模型和技术规范。

## 2. 设计方式

后端的整体设计遵循了“分层”和“模块化”的核心思想，以确保代码的高内聚、低耦合，并为未来的功能扩展和维护提供便利。

- **分层架构**:
  - **命令层 (`commands`)**: 作为后端API的入口，直接与前端交互。该层负责接收前端请求、验证参数，并调用核心逻辑。
  - **功能特性层 (`features`)**: 用于封装独立的、复杂的功能模块，如AI助手。
  - **核心逻辑层 (`core`)**: 包含应用的业务核心，如数据库交互 (`db`) 和数据模型定义 (`models`)。这一层封装了所有与数据相关的操作，使上层业务逻辑保持简洁。
- **模块化设计**:
  - **按实体/功能划分**: 每个核心数据实体（如 `Project`, `Task`）都拥有自己独立的命令文件，同时，像AI助手这样的复杂功能也被组织在自己的模块中。这使得功能按领域被清晰地组织起来，易于查找和管理。
  - **数据库抽象**: 数据库的初始化、连接池管理和迁移逻辑被统一封装在 [`src-tauri/src/core/db.rs`](src-tauri/src/core/db.rs) 中，对上层业务代码透明。

## 3. 实现功能

本次开发完成了 Cutie 后端的全部基础功能，并集成了一个内部的AI助手模块。

- **数据库初始化与迁移**:
  - 应用启动时，会自动检查并创建 SQLite 数据库文件 (`cutie.db`)。
  - 使用 `sqlx-cli` 的迁移机制，通过执行 [`src-tauri/migrations/20250921123300_initial_schema.sql`](src-tauri/migrations/20250921123300_initial_schema.sql) 文件来自动创建所有数据表和索引。
- **数据模型定义**:
  - 在 [`src-tauri/src/core/models.rs`](src-tauri/src/core/models.rs) 中，为数据库中的每一个表定义了对应的 Rust `struct`，并使用 `serde` 来支持与前端的数据交换。
- **全覆盖的CRUD命令**:
  - 为所有核心实体（`Project`, `Task`, `Checkpoint`, `Activity`, `Tag`）实现了完整的创建、读取、更新和删除操作，并通过Tauri命令暴露给前端。
- **关系链接功能**:
  - 实现了用于管理实体间多对多关系的核心功能（如任务与活动、项目与标签等）。
- **AI 助手模块**:
  - 在 [`src-tauri/src/features/ai_handler.rs`](src-tauri/src/features/ai_handler.rs) 中实现了一个AI助手模块。
  - 该模块能够从数据库的 `settings` 表中动态读取 `fast` 和 `slow` 两种模型的配置（API Key, Base URL, Model ID）。
  - 实现了一个核心函数 `stream_chat_completion`，能够根据指定模型配置，向 OpenAI API 发起流式聊天请求。此功能目前仅供后端内部使用。

## 4. 新建与修改的文件清单

### 4.1 新建文件

- **数据库迁移**:
  - [`src-tauri/migrations/20250921123300_initial_schema.sql`](src-tauri/migrations/20250921123300_initial_schema.sql)
- **核心模块**:
  - [`src-tauri/src/core/mod.rs`](src-tauri/src/core/mod.rs)
  - [`src-tauri/src/core/db.rs`](src-tauri/src/core/db.rs)
  - [`src-tauri/src/core/models.rs`](src-tauri/src/core/models.rs)
- **命令模块**:
  - [`src-tauri/src/commands/mod.rs`](src-tauri/src/commands/mod.rs)
  - [`src-tauri/src/commands/project_commands.rs`](src-tauri/src/commands/project_commands.rs)
  - [`src-tauri/src/commands/task_commands.rs`](src-tauri/src/commands/task_commands.rs)
  - [`src-tauri/src/commands/checkpoint_commands.rs`](src-tauri/src/commands/checkpoint_commands.rs)
  - [`src-tauri/src/commands/activity_commands.rs`](src-tauri/src/commands/activity_commands.rs)
  - [`src-tauri/src/commands/tag_commands.rs`](src-tauri/src/commands/tag_commands.rs)
  - [`src-tauri/src/commands/link_commands.rs`](src-tauri/src/commands/link_commands.rs)
- **功能特性模块**:
  - [`src-tauri/src/features/mod.rs`](src-tauri/src/features/mod.rs)
  - [`src-tauri/src/features/ai_handler.rs`](src-tauri/src/features/ai_handler.rs)
- **文档**:
  - [`ai-doc/backend-development-report.md`](ai-doc/backend-development-report.md) (本报告)
  - [`reference/api-reference.md`](reference/api-reference.md)

### 4.2 修改的文件

- [`src-tauri/Cargo.toml`](src-tauri/Cargo.toml): 添加了 `sqlx`, `tokio`, `chrono`, `uuid`, `thiserror`, `anyhow`, `async-openai`, `futures-util`, `async-stream` 等依赖。
- [`src-tauri/src/main.rs`](src-tauri/src/main.rs): 将 `main` 函数修改为 `async` 以支持 `tokio` 运行时。
- [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs):
  - 声明了 `core`, `commands`, `features` 模块。
  - 在应用启动时加入了数据库初始化和迁移的逻辑。
  - 使用 `.invoke_handler()` 注册了所有需要对前端暴露的Tauri命令。

## 5. 总结

后端开发任务已成功完成。当前后端系统健壮、功能完备，并为前端提供了清晰、一致的API。后续开发可以基于此稳固的基础，继续构建前端UI和更高级的业务逻辑。

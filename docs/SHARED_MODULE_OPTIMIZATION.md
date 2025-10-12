# Shared 模块优化方案

## 📋 概述

本文档描述了 `features/shared` 模块的完整优化方案，旨在提供清晰、一致且易用的模块导出结构。

## 🎯 优化目标

1. **统一导出策略**：所有常用类型都在顶层重新导出，避免深层路径访问
2. **命名空间清晰**：通过分组和注释保持代码可读性
3. **易于使用**：开发者只需一行 `use crate::features::shared::XXX` 即可导入任何类型
4. **易于维护**：新增类型时遵循清晰的分类规则

## 📁 目录结构

```
features/shared/
├── mod.rs                          # 主模块文件，负责重新导出所有类型
├── repositories/                   # 数据访问层
│   ├── mod.rs                      # 导出所有 Repository
│   ├── area_repository.rs
│   ├── task_repository.rs
│   ├── task_recurrence_repository.rs
│   ├── task_recurrence_link_repository.rs
│   ├── task_schedule_repository.rs
│   ├── task_time_block_link_repository.rs
│   ├── time_block_repository.rs
│   └── transaction.rs              # TransactionHelper
├── assemblers/                     # 数据组装层
│   ├── mod.rs                      # 导出所有 Assembler
│   ├── assembler.rs                # TaskAssembler
│   ├── task_card_assembler.rs      # ViewTaskCardAssembler
│   ├── linked_task_assembler.rs    # LinkedTaskAssembler
│   └── time_block_assembler.rs     # TimeBlockAssembler
├── services/                       # 业务服务层
│   ├── mod.rs                      # 导出所有 Service
│   ├── ai_classification_service.rs
│   ├── recurrence_instantiation_service.rs
│   └── conflict_checker.rs         # TimeBlockConflictChecker
└── validators/                     # 验证器层（预留）
    └── mod.rs
```

## 📦 导出策略

### 1. 顶层导出（shared/mod.rs）

所有常用类型都在 `shared/mod.rs` 中重新导出，按职责分为三大类：

#### 数据访问层（Repositories）

```rust
pub use repositories::{
    AreaRepository,
    TaskRecurrenceLinkRepository,
    TaskRecurrenceRepository,
    TaskRepository,
    TaskScheduleRepository,
    TaskTimeBlockLinkRepository,
    TimeBlockRepository,
    TransactionHelper,
};
```

#### 数据组装层（Assemblers）

```rust
pub use assemblers::{
    LinkedTaskAssembler,
    TaskAssembler,
    TimeBlockAssembler,
    ViewTaskCardAssembler,
};
```

#### 业务服务层（Services）

```rust
pub use services::{
    AiClassificationService,
    RecurrenceInstantiationService,
    TimeBlockConflictChecker,
};
```

### 2. 子模块导出

每个子模块（repositories、assemblers、services）的 `mod.rs` 负责：

1. 声明内部模块
2. 重新导出所有公开类型

示例（repositories/mod.rs）：

```rust
pub mod area_repository;
pub mod task_repository;
// ...

pub use area_repository::AreaRepository;
pub use task_repository::TaskRepository;
// ...
```

## 🔧 使用方式

### ✅ 推荐用法

```rust
// 1. 直接从 shared 导入（最简洁）
use crate::features::shared::{
    TaskRepository,
    TaskAssembler,
    TransactionHelper,
};

// 2. 按类别分组导入（语义清晰）
use crate::features::shared::{
    // Repositories
    TaskRepository,
    TaskScheduleRepository,

    // Assemblers
    TaskAssembler,
    ViewTaskCardAssembler,

    // Services
    RecurrenceInstantiationService,
};
```

### ❌ 避免用法

```rust
// ❌ 不要使用深层路径
use crate::features::shared::repositories::TaskRepository;

// ❌ 不要使用通配符导入（除非特殊情况）
use crate::features::shared::*;
```

## 📊 优化前后对比

### 优化前

```rust
// 导入路径不一致，需要记住哪些在顶层，哪些在子模块
use crate::features::shared::{
    TaskAssembler,                              // 顶层
    TransactionHelper,                          // 顶层
};
use crate::features::shared::repositories::{
    TaskRepository,                             // 子模块
    TaskScheduleRepository,                     // 子模块
};
```

### 优化后

```rust
// 所有类型统一从顶层导入
use crate::features::shared::{
    TaskAssembler,
    TaskRepository,
    TaskScheduleRepository,
    TransactionHelper,
};
```

## 🔍 维护指南

### 添加新的 Repository

1. 在 `repositories/` 下创建新文件
2. 在 `repositories/mod.rs` 中声明并导出
3. 在 `shared/mod.rs` 中添加到 Repositories 分组

```rust
// repositories/mod.rs
pub mod new_repository;
pub use new_repository::NewRepository;

// shared/mod.rs
pub use repositories::{
    AreaRepository,
    NewRepository,  // 添加在这里，保持字母顺序
    TaskRepository,
    // ...
};
```

### 添加新的 Assembler

同样的流程，添加到 `assemblers/` 目录和相应的导出位置。

### 添加新的 Service

同样的流程，添加到 `services/` 目录和相应的导出位置。

## ✨ 优势总结

1. **开发体验提升**
   - 统一的导入方式，无需记忆复杂的路径结构
   - IDE 自动补全更友好

2. **代码可读性提升**
   - 清晰的分组注释
   - 按字母顺序排列，易于查找

3. **维护性提升**
   - 新增类型时有明确的添加位置
   - 遵循统一的命名规范

4. **可扩展性**
   - 预留了 `validators` 层
   - 可以轻松添加新的分类

## 🔗 相关文档

- [架构设计](./ARCHITECTURE.md)
- [开发指南](../references/DEVELOPMENT_GUIDELINES.md)
- [如何添加功能](../references/HOW_TO_ADD_FEATURES.md)

## 📝 更新日志

- 2025-10-11: 完成 shared 模块重构，统一导出策略

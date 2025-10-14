# 基础设施架构（Infra Layer）

## 📁 目录结构

```
src/
├── infra/                    # 基础设施层（与业务无关）
│   ├── commandBus/           # 命令总线系统
│   │   ├── CommandBus.ts     # 核心命令总线
│   │   ├── types.ts          # 命令类型定义
│   │   ├── handlers/         # 命令处理器
│   │   │   ├── index.ts
│   │   │   ├── taskHandlers.ts
│   │   │   ├── scheduleHandlers.ts
│   │   │   └── timeBlockHandlers.ts
│   │   ├── index.ts
│   │   └── README.md
│   │
│   ├── transaction/          # 事务处理系统
│   │   ├── TransactionProcessor.ts  # 统一事务处理器
│   │   └── index.ts
│   │
│   ├── correlation/          # Correlation ID 系统
│   │   ├── correlationId.ts  # ID 生成器
│   │   └── index.ts
│   │
│   ├── events/               # SSE 事件系统
│   │   ├── EventSubscriber.ts  # SSE 订阅器
│   │   └── index.ts
│   │
│   ├── logging/              # 日志系统
│   │   ├── Logger.ts         # 日志核心
│   │   ├── loggerSettings.ts # 日志配置
│   │   └── index.ts
│   │
│   └── errors/               # 错误处理系统
│       ├── errorHandler.ts   # 全局错误处理
│       └── index.ts
│
├── stores/                   # 状态管理层（业务相关）
│   ├── task/
│   ├── timeblock/
│   └── ...
│
└── components/               # 组件层
```

## 🎯 分层原则

### **Infra 层（基础设施）**

- ✅ 与具体业务无关
- ✅ 可复用于其他项目
- ✅ 提供通用功能
- ❌ 不知道 Task、TimeBlock 等业务概念

### **Store 层（状态管理）**

- ✅ 包含业务状态（Task、TimeBlock）
- ✅ 依赖 Infra 层
- ❌ 不包含 UI 逻辑

### **Component 层（组件）**

- ✅ UI 渲染和用户交互
- ✅ 依赖 Store 和 Infra
- ❌ 不直接调用 API

## 📦 各模块职责

### **1. commandBus（命令总线）**

```typescript
// CPU 类比：指令译码器 + 指令总线
- CommandBus.ts: 核心总线，分发命令
- types.ts: 指令集定义（ISA）
- handlers/: 执行单元（Execution Units）
```

### **2. transaction（事务处理）**

```typescript
// CPU 类比：Reorder Buffer + Commit Unit
- TransactionProcessor.ts:
  - 去重（基于 correlation_id）
  - 应用主资源和副作用
  - TTL 自动清理
```

### **3. correlation（关联追踪）**

```typescript
// CPU 类比：Transaction ID Generator
- correlationId.ts:
  - 生成唯一 ID
  - ID 存储管理
```

### **4. events（SSE 事件系统）**

```typescript
// CPU 类比：中断控制器（Interrupt Controller）
- EventSubscriber.ts:
  - SSE 连接管理
  - 事件分发
  - 自动重连
```

### **5. logging（日志系统）**

```typescript
// CPU 类比：调试跟踪单元（Debug Trace Unit）
- Logger.ts: 结构化日志生成
- loggerSettings.ts: 预设配置
```

### **6. errors（错误处理）**

```typescript
// CPU 类比：异常处理单元（Exception Handler）
- errorHandler.ts: 全局错误捕获
```

## 🔧 迁移计划

### Phase 1: 创建 infra 目录结构

```bash
src/infra/
├── commandBus/
├── transaction/
├── correlation/
├── events/
├── logging/
└── errors/
```

### Phase 2: 移动文件

```
services/commandBus/*       → infra/commandBus/
services/transactionProcessor.ts → infra/transaction/
services/correlationId.ts   → infra/correlation/
services/logger.ts          → infra/logging/
services/loggerSettings.ts  → infra/logging/
services/errorHandler.ts    → infra/errors/
services/events.ts          → infra/events/
```

### Phase 3: 更新导入路径

```typescript
// 修改前
import { commandBus } from '@/commandBus'
import { logger } from '@/infra/logging/logger'

// 修改后
import { commandBus } from '@/infra/commandBus'
import { logger } from '@/infra/logging'
```

### Phase 4: 创建统一导出

```typescript
// src/infra/index.ts
export * from './commandBus'
export * from './transaction'
export * from './correlation'
export * from './events'
export * from './logging'
export * from './errors'
```

## 🎯 架构优势

1. **清晰分层**：基础设施与业务逻辑分离
2. **可复用性**：infra 层可用于其他项目
3. **易于理解**：一眼看出哪些是基础设施
4. **便于测试**：可以单独测试基础设施
5. **符合 CPU 架构**：硬件层（infra）vs 应用层（stores/components）

---

要我继续执行迁移吗？还是你想手动调整目录结构？

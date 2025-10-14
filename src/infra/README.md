# 基础设施层（Infra Layer）

## 📁 目录结构

```
src/infra/
├── commandBus/          # 指令总线（Instruction Bus）
│   ├── CommandBus.ts    # 核心命令总线
│   ├── types.ts         # 命令类型定义（ISA）
│   ├── handlers/        # 命令处理器（Execution Units）
│   │   ├── taskHandlers.ts
│   │   ├── scheduleHandlers.ts
│   │   └── timeBlockHandlers.ts
│   └── index.ts
│
├── transaction/         # 事务处理器（Reorder Buffer + Commit Unit）
│   ├── transactionProcessor.ts
│   └── index.ts
│
├── correlation/         # 关联追踪（Transaction ID Generator）
│   ├── correlationId.ts
│   └── index.ts
│
├── events/              # SSE 事件系统（Interrupt Controller）
│   ├── events.ts
│   └── index.ts
│
├── logging/             # 日志系统（Debug Trace Unit）
│   ├── logger.ts
│   ├── loggerSettings.ts
│   └── index.ts
│
└── errors/              # 错误处理（Exception Handler）
    ├── errorHandler.ts
    └── index.ts
```

## 🎯 设计原则

### **1. 与业务无关**

- ✅ 不包含 Task、TimeBlock 等业务概念
- ✅ 可复用于其他项目
- ✅ 提供通用的技术能力

### **2. CPU 硬件类比**

```
commandBus       = 指令总线（Instruction Bus）
transaction      = 重排序缓冲（Reorder Buffer）
correlation      = 事务ID生成器（Transaction ID Generator）
events           = 中断控制器（Interrupt Controller）
logging          = 调试跟踪单元（Debug Trace Unit）
errors           = 异常处理单元（Exception Handler）
```

### **3. 单向依赖**

```
Components → Stores → Infra
                ↑         ↓
                └─────────┘
                （只依赖，不循环）
```

## 📦 各模块说明

### **commandBus（指令总线）**

- 接收来自组件的命令
- 译码并分发到对应的 Handler
- 统一的错误处理和日志

### **transaction（事务处理器）**

- 统一处理 HTTP 和 SSE 响应
- 基于 correlation_id 去重
- 自动应用主资源和所有副作用
- TTL 自动清理已处理事务

### **correlation（关联追踪）**

- 生成唯一的 correlation ID
- 追踪活跃的请求
- 防止内存泄漏

### **events（SSE 事件系统）**

- 管理 SSE 连接
- 事件订阅和分发
- 自动重连机制

### **logging（日志系统）**

- 结构化日志生成
- 日志级别控制
- 标签过滤
- 采样和性能监控

### **errors（错误处理）**

- 全局错误捕获
- Vue 错误处理器
- 错误上报

## 🚀 使用示例

```typescript
// 导入基础设施
import { commandBus } from '@/infra/commandBus'
import { transactionProcessor } from '@/infra/transaction'
import { generateCorrelationId } from '@/infra/correlation'
import { logger, LogTags } from '@/infra/logging'

// 或者统一导入
import { commandBus, transactionProcessor, logger } from '@/infra'
```

---

**版本**: 1.0  
**最后更新**: 2024-10-14

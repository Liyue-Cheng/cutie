# CPU系统解耦分析与npm包化方案

## 📊 当前耦合程度分析

### 1. 依赖关系总览

```
CPU系统 (src/cpu/)
├── 核心层 (Pipeline, Stages) - ⚠️ 轻度耦合
│   ├── Vue (响应式状态)
│   └── correlation ID生成
├── ISA层 (指令集) - ❌ 重度耦合
│   ├── 项目Stores (task, schedule, timeblock等)
│   ├── 项目DTOs类型
│   └── transactionProcessor
├── 工具层 (utils/request) - ❌ 重度耦合
│   ├── @/stores/shared (API函数)
│   └── @/infra/logging/logger
└── 日志层 (logging) - ✅ 无耦合
    └── 完全独立
```

### 2. 详细依赖清单

#### 2.1 核心流水线系统（可移植性：★★★★☆）

| 文件            | 外部依赖                            | 耦合程度 |
| --------------- | ----------------------------------- | -------- |
| `Pipeline.ts`   | `vue` (ref)                         | 轻度     |
| `stages/IF.ts`  | `@/infra/correlation/correlationId` | 轻度     |
| `stages/SCH.ts` | 无                                  | 无       |
| `stages/EX.ts`  | 无                                  | 无       |
| `stages/RES.ts` | 无                                  | 无       |
| `stages/WB.ts`  | 无                                  | 无       |
| `types.ts`      | 无                                  | 无       |

**评估**：核心流水线架构设计良好，仅有2个轻度依赖，易于解耦。

#### 2.2 工具层（可移植性：★★☆☆☆）

| 文件               | 外部依赖                              | 耦合程度 |
| ------------------ | ------------------------------------- | -------- |
| `utils/request.ts` | `@/stores/shared` (apiGet, apiPost等) | 重度     |
| `utils/request.ts` | `@/infra/logging/logger`              | 中度     |

**问题**：

- 硬编码依赖项目的HTTP客户端实现
- 无法替换为其他HTTP库

#### 2.3 ISA指令集（可移植性：★☆☆☆☆）

| 指令集文件                  | 外部依赖                                     | 耦合程度 |
| --------------------------- | -------------------------------------------- | -------- |
| `isa/debug-isa.ts`          | 无                                           | 无       |
| `isa/task-isa.ts`           | `useTaskStore`, `transactionProcessor`, DTOs | 重度     |
| `isa/schedule-isa.ts`       | `useTaskStore`, DTOs                         | 重度     |
| `isa/timeblock-isa.ts`      | `useTimeBlockStore`, `useTaskStore`, DTOs    | 重度     |
| `isa/template-isa.ts`       | `useTemplateStore`, `useTaskStore`, DTOs     | 重度     |
| `isa/recurrence-isa.ts`     | `useRecurrenceStore`, DTOs                   | 重度     |
| `isa/viewpreference-isa.ts` | `useViewStore`                               | 重度     |

**问题**：

- 所有业务ISA都直接访问项目stores
- 依赖项目特定的DTO类型
- 无法在其他项目中复用

#### 2.4 日志系统（可移植性：★★★★★）

| 文件        | 外部依赖 | 耦合程度 |
| ----------- | -------- | -------- |
| `logging/*` | 无       | 无       |

**优势**：完全独立，可直接移植。

---

## 🎯 解耦策略

### 策略1：分层架构（推荐）

将CPU系统分为3层：

```
┌─────────────────────────────────────────┐
│   业务层 ISA（保留在项目中）            │
│   - task-isa.ts                         │
│   - schedule-isa.ts                     │
│   - 直接使用项目stores和types           │
└──────────────────┬──────────────────────┘
                   │ 实现接口
┌──────────────────▼──────────────────────┐
│   CPU核心包 (@your-org/cpu-pipeline)    │
│   - Pipeline                            │
│   - Stages (IF, SCH, EX, RES, WB)      │
│   - ISA类型定义                         │
│   - 抽象接口（HTTP、Logger、Store）     │
└──────────────────┬──────────────────────┘
                   │ 依赖注入
┌──────────────────▼──────────────────────┐
│   适配器层（保留在项目中）              │
│   - HttpAdapter                         │
│   - LoggerAdapter                       │
│   - CorrelationIdAdapter                │
└─────────────────────────────────────────┘
```

**优势**：

- CPU核心系统完全独立
- 业务ISA留在项目中，保持灵活性
- 通过适配器注入项目依赖

### 策略2：完全独立包（最彻底）

将所有内容都变成独立包，项目只使用不修改。

**缺点**：

- 灵活性降低
- 业务ISA难以适配不同项目

### 策略3：插件架构（最灵活）

CPU核心 + 插件系统，ISA作为插件注册。

---

## 📦 npm包化方案（推荐策略1）

### 第1步：定义抽象接口

```typescript
// packages/cpu-pipeline/src/interfaces.ts

/**
 * HTTP客户端接口
 */
export interface IHttpClient {
  get<T>(url: string, config?: RequestConfig): Promise<T>
  post<T>(url: string, data?: any, config?: RequestConfig): Promise<T>
  patch<T>(url: string, data?: any, config?: RequestConfig): Promise<T>
  put<T>(url: string, data?: any, config?: RequestConfig): Promise<T>
  delete<T>(url: string, config?: RequestConfig): Promise<T>
}

/**
 * 日志接口
 */
export interface ILogger {
  debug(tag: string, message: string, data?: any): void
  info(tag: string, message: string, data?: any): void
  warn(tag: string, message: string, data?: any): void
  error(tag: string, message: string, data?: any): void
}

/**
 * CorrelationId生成器接口
 */
export interface ICorrelationIdGenerator {
  generate(): string
}

/**
 * 状态管理接口（可选，用于响应式）
 */
export interface IReactiveState<T> {
  value: T
  setValue(newValue: T): void
  subscribe(callback: (value: T) => void): () => void
}
```

### 第2步：重构核心系统

#### 2.1 Pipeline改造

```typescript
// packages/cpu-pipeline/src/Pipeline.ts

import type { IReactiveState, ILogger } from './interfaces'

export interface PipelineConfig {
  tickInterval?: number
  maxConcurrency?: number
  reactiveStateFactory?: <T>(initialValue: T) => IReactiveState<T>
  logger?: ILogger
}

export class Pipeline {
  private config: Required<PipelineConfig>
  public status: IReactiveState<PipelineStatus>

  constructor(config: PipelineConfig = {}) {
    this.config = {
      tickInterval: config.tickInterval ?? 16,
      maxConcurrency: config.maxConcurrency ?? 10,
      reactiveStateFactory: config.reactiveStateFactory ?? createPlainState,
      logger: config.logger ?? consoleLogger,
    }

    // 使用注入的状态工厂
    this.status = this.config.reactiveStateFactory({
      ifBufferSize: 0,
      schPendingSize: 0,
      schActiveSize: 0,
      totalCompleted: 0,
      totalFailed: 0,
    })
  }

  // ... 其他代码保持不变
}

// 默认实现：普通状态（非响应式）
function createPlainState<T>(initialValue: T): IReactiveState<T> {
  let value = initialValue
  const subscribers: Array<(value: T) => void> = []

  return {
    get value() {
      return value
    },
    setValue(newValue: T) {
      value = newValue
      subscribers.forEach((cb) => cb(value))
    },
    subscribe(callback) {
      subscribers.push(callback)
      return () => {
        const index = subscribers.indexOf(callback)
        if (index > -1) subscribers.splice(index, 1)
      }
    },
  }
}

// 默认日志实现
const consoleLogger: ILogger = {
  debug: (tag, msg, data) => console.debug(`[${tag}] ${msg}`, data),
  info: (tag, msg, data) => console.info(`[${tag}] ${msg}`, data),
  warn: (tag, msg, data) => console.warn(`[${tag}] ${msg}`, data),
  error: (tag, msg, data) => console.error(`[${tag}] ${msg}`, data),
}
```

#### 2.2 Request工具改造

```typescript
// packages/cpu-pipeline/src/utils/request.ts

import type { IHttpClient, ILogger } from '../interfaces'
import type { InstructionContext } from '../types'
import type { RequestConfig, MultiRequestConfig } from '../isa/types'

let httpClient: IHttpClient | null = null
let logger: ILogger | null = null

/**
 * 设置HTTP客户端（必须在使用前调用）
 */
export function setHttpClient(client: IHttpClient): void {
  httpClient = client
}

/**
 * 设置日志器（可选）
 */
export function setLogger(log: ILogger): void {
  logger = log
}

async function executeSingleRequest(
  config: RequestConfig,
  payload: any,
  context: InstructionContext
): Promise<any> {
  if (!httpClient) {
    throw new Error('HttpClient未初始化，请先调用setHttpClient()')
  }

  const url = typeof config.url === 'function' ? config.url(payload) : config.url
  const body = config.body ? config.body(payload) : payload

  const headers = {
    'X-Correlation-ID': context.correlationId,
    ...config.headers,
  }

  logger?.debug('SYSTEM_PIPELINE', 'Executing HTTP request', {
    method: config.method,
    url,
    correlationId: context.correlationId,
  })

  switch (config.method) {
    case 'GET':
      return await httpClient.get(url, { headers })
    case 'POST':
      return await httpClient.post(url, body, { headers })
    case 'PUT':
      return await httpClient.put(url, body, { headers })
    case 'PATCH':
      return await httpClient.patch(url, body, { headers })
    case 'DELETE':
      return await httpClient.delete(url, { headers })
    default:
      throw new Error(`Unsupported HTTP method: ${config.method}`)
  }
}

// executeRequest保持不变
export async function executeRequest(
  config: RequestConfig | MultiRequestConfig,
  payload: any,
  context: InstructionContext
): Promise<any> {
  if (!isMultiRequestConfig(config)) {
    return await executeSingleRequest(config, payload, context)
  }

  const { requests, mode, combineResults } = config
  let results: any[]

  if (mode === 'parallel') {
    results = await Promise.all(requests.map((req) => executeSingleRequest(req, payload, context)))
  } else {
    results = []
    for (const req of requests) {
      const result = await executeSingleRequest(req, payload, context)
      results.push(result)
    }
  }

  if (combineResults) {
    return combineResults(results)
  }

  return results
}

function isMultiRequestConfig(
  config: RequestConfig | MultiRequestConfig
): config is MultiRequestConfig {
  return 'requests' in config && Array.isArray(config.requests)
}
```

#### 2.3 IF阶段改造

```typescript
// packages/cpu-pipeline/src/stages/IF.ts

import type { ICorrelationIdGenerator } from '../interfaces'
import type { QueuedInstruction } from '../types'

let correlationIdGenerator: ICorrelationIdGenerator | null = null

/**
 * 设置CorrelationId生成器（必须在使用前调用）
 */
export function setCorrelationIdGenerator(generator: ICorrelationIdGenerator): void {
  correlationIdGenerator = generator
}

export class InstructionFetchStage {
  private idCounter = 0

  fetchInstruction<TPayload>(
    type: string,
    payload: TPayload,
    source: 'user' | 'system' | 'test' = 'user',
    callSource?: CallSource
  ): QueuedInstruction<TPayload> {
    const instructionId = `instr-${Date.now()}-${++this.idCounter}`

    if (!correlationIdGenerator) {
      throw new Error('CorrelationIdGenerator未初始化')
    }

    const correlationId = correlationIdGenerator.generate()

    const instruction: QueuedInstruction<TPayload> = {
      id: instructionId,
      type,
      payload,
      context: {
        instructionId,
        correlationId,
        timestamp: Date.now(),
        source,
        retryCount: 0,
        callSource,
      },
      status: InstructionStatus.PENDING,
      timestamps: {
        IF: Date.now(),
      },
    }

    return instruction
  }
}
```

### 第3步：在项目中创建适配器

```typescript
// src/cpu-adapters/httpAdapter.ts

import { apiGet, apiPost, apiPatch, apiDelete, apiPut } from '@/stores/shared'
import type { IHttpClient } from '@your-org/cpu-pipeline'

export const httpAdapter: IHttpClient = {
  async get<T>(url: string, config?: any): Promise<T> {
    // 提取correlationId
    const correlationId = config?.headers?.['X-Correlation-ID']
    return await apiGet(url, correlationId)
  },

  async post<T>(url: string, data?: any, config?: any): Promise<T> {
    return await apiPost(url, data, config)
  },

  async patch<T>(url: string, data?: any, config?: any): Promise<T> {
    return await apiPatch(url, data, config)
  },

  async put<T>(url: string, data?: any, config?: any): Promise<T> {
    const correlationId = config?.headers?.['X-Correlation-ID']
    return await apiPut(url, data, correlationId)
  },

  async delete<T>(url: string, config?: any): Promise<T> {
    return await apiDelete(url, config)
  },
}
```

```typescript
// src/cpu-adapters/loggerAdapter.ts

import { logger as projectLogger, LogTags } from '@/infra/logging/logger'
import type { ILogger } from '@your-org/cpu-pipeline'

export const loggerAdapter: ILogger = {
  debug: (tag, msg, data) => projectLogger.debug(LogTags.SYSTEM_PIPELINE, msg, data),
  info: (tag, msg, data) => projectLogger.info(LogTags.SYSTEM_PIPELINE, msg, data),
  warn: (tag, msg, data) => projectLogger.warn(LogTags.SYSTEM_PIPELINE, msg, data),
  error: (tag, msg, data) => projectLogger.error(LogTags.SYSTEM_PIPELINE, msg, data),
}
```

```typescript
// src/cpu-adapters/vueAdapter.ts

import { ref as vueRef } from 'vue'
import type { IReactiveState } from '@your-org/cpu-pipeline'

export function createVueReactiveState<T>(initialValue: T): IReactiveState<T> {
  const state = vueRef(initialValue)
  const subscribers: Array<(value: T) => void> = []

  // 监听Vue的ref变化
  watch(
    state,
    (newValue) => {
      subscribers.forEach((cb) => cb(newValue))
    },
    { deep: true }
  )

  return {
    get value() {
      return state.value
    },
    setValue(newValue: T) {
      state.value = newValue
    },
    subscribe(callback) {
      subscribers.push(callback)
      return () => {
        const index = subscribers.indexOf(callback)
        if (index > -1) subscribers.splice(index, 1)
      }
    },
  }
}
```

```typescript
// src/cpu-adapters/correlationIdAdapter.ts

import { generateCorrelationId } from '@/infra/correlation/correlationId'
import type { ICorrelationIdGenerator } from '@your-org/cpu-pipeline'

export const correlationIdAdapter: ICorrelationIdGenerator = {
  generate: () => generateCorrelationId(),
}
```

### 第4步：初始化CPU系统

```typescript
// src/cpu/index.ts（项目中的CPU初始化）

import {
  Pipeline,
  setHttpClient,
  setLogger,
  setCorrelationIdGenerator,
} from '@your-org/cpu-pipeline'
import { httpAdapter } from '@/cpu-adapters/httpAdapter'
import { loggerAdapter } from '@/cpu-adapters/loggerAdapter'
import { correlationIdAdapter } from '@/cpu-adapters/correlationIdAdapter'
import { createVueReactiveState } from '@/cpu-adapters/vueAdapter'

// 初始化适配器
setHttpClient(httpAdapter)
setLogger(loggerAdapter)
setCorrelationIdGenerator(correlationIdAdapter)

// 创建流水线实例（使用Vue响应式）
export const pipeline = new Pipeline({
  tickInterval: 16,
  maxConcurrency: 10,
  reactiveStateFactory: createVueReactiveState,
  logger: loggerAdapter,
})

// 导出ISA（保留在项目中）
export { ISA } from './isa'

// 开发环境调试
if (import.meta.env.DEV) {
  ;(window as any).cpuPipeline = {
    pipeline,
    dispatch: (type: string, payload: any) => pipeline.dispatch(type, payload),
    start: () => pipeline.start(),
    stop: () => pipeline.stop(),
    reset: () => pipeline.reset(),
    getStatus: () => pipeline.getStatus(),
  }
}
```

### 第5步：ISA保留在项目中

```typescript
// src/cpu/isa/task-isa.ts（保持原样，直接使用项目依赖）

import type { ISADefinition } from '@your-org/cpu-pipeline'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'

export const TaskISA: ISADefinition = {
  'task.create': {
    meta: {
      description: '创建任务',
      category: 'task',
      resourceIdentifier: () => [],
      priority: 5,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.title?.trim()) {
        console.warn('❌ 任务标题不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: '/tasks',
    },

    commit: async (result: TaskCard) => {
      const taskStore = useTaskStore()
      taskStore.addOrUpdateTask_mut(result)
    },
  },

  // ... 其他task指令
}
```

---

## 📁 最终目录结构

```
workspace/
├── packages/
│   └── cpu-pipeline/                    # npm包
│       ├── package.json
│       ├── src/
│       │   ├── interfaces.ts            # 抽象接口
│       │   ├── Pipeline.ts              # 核心流水线
│       │   ├── stages/
│       │   │   ├── IF.ts
│       │   │   ├── SCH.ts
│       │   │   ├── EX.ts
│       │   │   ├── RES.ts
│       │   │   └── WB.ts
│       │   ├── types.ts
│       │   ├── logging/                 # 日志系统
│       │   │   ├── CPULogger.ts
│       │   │   ├── CPUConsole.ts
│       │   │   └── ...
│       │   ├── utils/
│       │   │   └── request.ts           # 使用IHttpClient
│       │   └── index.ts
│       └── README.md
│
└── cutie/                                # 你的项目
    ├── package.json                      # 依赖: @your-org/cpu-pipeline
    ├── src/
    │   ├── cpu-adapters/                 # 适配器层
    │   │   ├── httpAdapter.ts
    │   │   ├── loggerAdapter.ts
    │   │   ├── vueAdapter.ts
    │   │   └── correlationIdAdapter.ts
    │   ├── cpu/                          # 项目特定部分
    │   │   ├── index.ts                  # 初始化CPU
    │   │   └── isa/                      # 业务ISA
    │   │       ├── task-isa.ts
    │   │       ├── schedule-isa.ts
    │   │       ├── timeblock-isa.ts
    │   │       └── index.ts
    │   └── ...
    └── ...
```

---

## 📊 解耦效果对比

| 维度             | 解耦前               | 解耦后             |
| ---------------- | -------------------- | ------------------ |
| **核心系统依赖** | Vue, 项目infra       | 零依赖（接口注入） |
| **可移植性**     | ❌ 无法移植          | ✅ 可用于任何项目  |
| **测试性**       | ⚠️ 需要mock项目依赖  | ✅ 纯函数，易测试  |
| **维护性**       | ⚠️ 业务和框架混合    | ✅ 关注点分离      |
| **灵活性**       | ⚠️ 绑定Vue和项目结构 | ✅ 支持任意框架    |

---

## 🚀 实施步骤

### 阶段1：准备工作（1-2天）

1. ✅ 创建`packages/cpu-pipeline`目录
2. ✅ 定义`interfaces.ts`
3. ✅ 编写适配器实现

### 阶段2：重构核心（3-5天）

1. ✅ 改造`Pipeline.ts`
2. ✅ 改造`utils/request.ts`
3. ✅ 改造`stages/IF.ts`
4. ✅ 验证其他stages无需修改

### 阶段3：迁移ISA（2-3天）

1. ✅ 将ISA移到项目的`src/cpu/isa/`
2. ✅ 更新导入路径
3. ✅ 验证功能正常

### 阶段4：发布npm包（1天）

1. ✅ 完善package.json
2. ✅ 编写README和文档
3. ✅ 发布到npm或私有registry

### 阶段5：清理项目（1天）

1. ✅ 删除`src/cpu/stages`等已移入包的代码
2. ✅ 更新导入为从npm包导入
3. ✅ 全面测试

**总计：8-12天**

---

## 💡 额外建议

### 1. 使用Monorepo管理

```bash
# 推荐使用pnpm workspace
pnpm init
```

`pnpm-workspace.yaml`:

```yaml
packages:
  - 'packages/*'
  - 'cutie'
```

### 2. 添加类型导出

```typescript
// packages/cpu-pipeline/src/index.ts
export * from './interfaces'
export * from './types'
export * from './Pipeline'
export * from './logging'
```

### 3. 编写测试

```typescript
// packages/cpu-pipeline/tests/Pipeline.test.ts
import { Pipeline, setHttpClient, setCorrelationIdGenerator } from '../src'

describe('Pipeline', () => {
  beforeEach(() => {
    setHttpClient(mockHttpClient)
    setCorrelationIdGenerator(mockGenerator)
  })

  it('should dispatch instruction', async () => {
    const pipeline = new Pipeline()
    pipeline.start()

    const result = await pipeline.dispatch('test.instruction', {})
    expect(result).toBeDefined()
  })
})
```

### 4. 提供示例项目

```
packages/
├── cpu-pipeline/          # 核心包
├── cpu-pipeline-vue/      # Vue适配器包（可选）
└── cpu-example/           # 示例项目
```

---

## ❓ FAQ

### Q: 为什么不把ISA也放入npm包？

**A**: ISA高度依赖业务逻辑和项目stores，强行抽象会导致复杂性爆炸。保留在项目中更灵活。

### Q: 如何在React项目中使用？

**A**: 只需实现React版的`createReactiveState`：

```typescript
import { useState, useEffect } from 'react'

export function createReactReactiveState<T>(initialValue: T): IReactiveState<T> {
  const [state, setState] = useState(initialValue)
  const subscribers: Array<(value: T) => void> = []

  return {
    get value() {
      return state
    },
    setValue(newValue: T) {
      setState(newValue)
      subscribers.forEach((cb) => cb(newValue))
    },
    subscribe(callback) {
      subscribers.push(callback)
      return () => {
        const index = subscribers.indexOf(callback)
        if (index > -1) subscribers.splice(index, 1)
      }
    },
  }
}
```

### Q: 性能会受影响吗？

**A**: 几乎没有影响。适配器只是简单的函数调用，开销可忽略。

### Q: 需要改造现有组件代码吗？

**A**: 不需要。组件继续使用`pipeline.dispatch()`，API完全一致。

---

## 📝 总结

| 方案                  | 优点              | 缺点               | 推荐度     |
| --------------------- | ----------------- | ------------------ | ---------- |
| **当前状态**          | 简单直接          | 无法复用，难以测试 | ⭐⭐       |
| **策略1（分层）**     | 核心独立，ISA灵活 | 需要适配器层       | ⭐⭐⭐⭐⭐ |
| **策略2（完全独立）** | 最彻底解耦        | ISA通用性差        | ⭐⭐⭐     |
| **策略3（插件）**     | 最灵活            | 复杂度高           | ⭐⭐⭐⭐   |

**推荐采用策略1**，在保持灵活性的同时实现核心系统的独立性。

---

**生成时间**: 2025-10-24  
**作者**: AI Assistant  
**版本**: 1.0

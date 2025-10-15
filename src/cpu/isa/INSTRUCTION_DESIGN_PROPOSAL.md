# 指令集架构优化方案

## 📊 当前问题

### 代码重复

```typescript
// 每个指令都要写类似的代码
execute: async (payload, context) => {
  return await apiPost('/tasks', payload, {
    headers: { 'X-Correlation-ID': context.correlationId },
  })
}
```

### 难以管理

- 网络请求分散在各个指令中
- 无法统一添加重试、缓存、监控
- 乐观更新逻辑没有标准化

---

## 🎯 解决方案：混合架构

### 核心理念

**80% 标准化（声明式配置） + 20% 灵活性（自定义执行）**

---

## 📝 方案设计

### 1. 扩展指令定义结构

```typescript
// src/cpu/isa/types.ts

export interface InstructionDefinition<TPayload = any, TResult = any> {
  meta: InstructionMeta

  // ==================== 声明式配置（推荐） ====================

  /**
   * HTTP 请求配置（声明式）
   */
  request?: {
    method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
    url: string | ((payload: TPayload) => string) // 支持动态 URL
    body?: (payload: TPayload) => any // 请求体映射
    headers?: Record<string, string> // 额外 headers
  }

  /**
   * 乐观更新配置（声明式）
   */
  optimistic?: {
    enabled: boolean
    apply: (payload: TPayload, context: InstructionContext) => OptimisticSnapshot
    rollback: (snapshot: OptimisticSnapshot) => void
  }

  /**
   * 提交逻辑（声明式）
   */
  commit?: {
    apply: (result: TResult, payload: TPayload, context: InstructionContext) => Promise<void>
  }

  // ==================== 自定义执行（灵活） ====================

  /**
   * 自定义验证（可选）
   */
  validate?: (payload: TPayload, context: InstructionContext) => Promise<boolean>

  /**
   * 自定义执行逻辑（可选，与 request 互斥）
   * 用于复杂场景（如多个请求、条件逻辑等）
   */
  execute?: (payload: TPayload, context: InstructionContext) => Promise<TResult>
}
```

---

## 🔨 实现示例

### 示例 1: 标准 CRUD 操作（声明式）

```typescript
// src/cpu/isa/task-isa.ts

export const TaskISA: ISADefinition = {
  'task.create': {
    meta: {
      description: '创建任务',
      category: 'task',
      resourceIdentifier: () => [],
      priority: 5,
      timeout: 10000,
    },

    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: '/tasks',
      body: (payload) => payload, // 直接传递
    },

    // 🔥 声明式提交配置
    commit: {
      apply: async (result: TaskCard) => {
        const taskStore = useTaskStore()
        taskStore.addOrUpdateTask_mut(result)
      },
    },

    // 🔥 可选：乐观更新
    optimistic: {
      enabled: true,
      apply: (payload) => {
        const tempId = `temp-${Date.now()}`
        const taskStore = useTaskStore()
        taskStore.addOrUpdateTask_mut({
          id: tempId,
          ...payload,
          is_completed: false,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        })
        return { tempId } // 返回快照，用于回滚
      },
      rollback: (snapshot) => {
        const taskStore = useTaskStore()
        taskStore.removeTask_mut(snapshot.tempId)
      },
    },
  },

  'task.update': {
    meta: {
      /* ... */
    },

    request: {
      method: 'PATCH',
      url: (payload) => `/tasks/${payload.task_id}`, // 🔥 动态 URL
      body: (payload) => payload.updates,
    },

    commit: {
      apply: async (result: TaskCard) => {
        const taskStore = useTaskStore()
        taskStore.addOrUpdateTask_mut(result)
      },
    },
  },

  'task.delete': {
    meta: {
      /* ... */
    },

    request: {
      method: 'DELETE',
      url: (payload) => `/tasks/${payload.task_id}`,
    },

    commit: {
      apply: async (_, payload) => {
        const taskStore = useTaskStore()
        taskStore.removeTask_mut(payload.task_id)
      },
    },
  },
}
```

### 示例 2: 复杂逻辑（自定义执行）

```typescript
// 需要多个请求或条件逻辑的场景
export const ComplexISA: ISADefinition = {
  'task.batch_update': {
    meta: {
      /* ... */
    },

    // 🔥 自定义执行（复杂逻辑）
    execute: async (payload, context) => {
      const results = []

      // 1. 并发更新多个任务
      for (const taskId of payload.task_ids) {
        const result = await apiPatch(`/tasks/${taskId}`, payload.updates, {
          headers: { 'X-Correlation-ID': context.correlationId },
        })
        results.push(result)
      }

      // 2. 额外逻辑：更新视图排序
      if (payload.updateSorting) {
        await apiPatch(`/views/${payload.view_key}/sorting`, {
          sorted_task_ids: results.map((t) => t.id),
        })
      }

      return { tasks: results }
    },

    commit: {
      apply: async (result) => {
        const taskStore = useTaskStore()
        for (const task of result.tasks) {
          taskStore.addOrUpdateTask_mut(task)
        }
      },
    },
  },
}
```

---

## 🛠️ EX 阶段实现

```typescript
// src/cpu/stages/EX.ts

import { executeRequest } from '../utils/request'

export class ExecuteStage {
  async execute(instruction: QueuedInstruction): Promise<void> {
    const definition = ISA[instruction.type]
    if (!definition) {
      throw new Error(`Unknown instruction: ${instruction.type}`)
    }

    // 1. 执行验证（可选）
    if (definition.validate) {
      const valid = await definition.validate(instruction.payload, instruction.context)
      if (!valid) {
        throw new Error('Validation failed')
      }
    }

    // 2. 执行乐观更新（可选）
    if (definition.optimistic?.enabled) {
      instruction.optimisticSnapshot = definition.optimistic.apply(
        instruction.payload,
        instruction.context
      )
    }

    instructionTracker.markPhase(instruction.id, PipelineStage.EX)

    try {
      // 3. 执行网络请求或自定义逻辑
      if (definition.request) {
        // 🔥 声明式请求（标准化）
        instruction.result = await executeRequest(
          definition.request,
          instruction.payload,
          instruction.context
        )
      } else if (definition.execute) {
        // 🔥 自定义执行（灵活）
        instruction.result = await definition.execute(instruction.payload, instruction.context)
      } else {
        throw new Error('Neither request nor execute is defined')
      }
    } catch (error) {
      // 4. 失败时回滚乐观更新
      if (instruction.optimisticSnapshot && definition.optimistic?.rollback) {
        definition.optimistic.rollback(instruction.optimisticSnapshot)
      }
      throw error
    }
  }
}
```

---

## 🔧 统一请求工具

```typescript
// src/cpu/utils/request.ts

import { apiGet, apiPost, apiPatch, apiDelete } from '@/stores/shared'
import type { InstructionContext } from '../types'

export async function executeRequest(
  config: {
    method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
    url: string | ((payload: any) => string)
    body?: (payload: any) => any
    headers?: Record<string, string>
  },
  payload: any,
  context: InstructionContext
): Promise<any> {
  const url = typeof config.url === 'function' ? config.url(payload) : config.url
  const body = config.body ? config.body(payload) : payload

  // 统一添加 correlation-id
  const headers = {
    'X-Correlation-ID': context.correlationId,
    ...config.headers,
  }

  switch (config.method) {
    case 'GET':
      return await apiGet(url, { headers })
    case 'POST':
      return await apiPost(url, body, { headers })
    case 'PATCH':
      return await apiPatch(url, body, { headers })
    case 'DELETE':
      return await apiDelete(url, { headers })
    default:
      throw new Error(`Unsupported method: ${config.method}`)
  }
}
```

---

## 📊 优势对比

| 特性           | 当前架构 | 声明式配置  | 自定义执行 |
| -------------- | -------- | ----------- | ---------- |
| **代码重复**   | ❌ 高    | ✅ 低       | ⚠️ 中      |
| **标准化**     | ❌ 低    | ✅ 高       | ⚠️ 中      |
| **灵活性**     | ✅ 高    | ⚠️ 中       | ✅ 高      |
| **易于追踪**   | ❌ 困难  | ✅ 容易     | ⚠️ 中等    |
| **中间件支持** | ❌ 无    | ✅ 易于添加 | ⚠️ 需手动  |
| **学习成本**   | ✅ 低    | ⚠️ 中       | ✅ 低      |

---

## 🚀 迁移策略

### 阶段 1: 实现基础设施

1. 实现 `executeRequest` 工具函数
2. 扩展 `InstructionDefinition` 类型
3. 更新 EX 阶段以支持声明式配置

### 阶段 2: 渐进式迁移

1. 优先迁移简单的 CRUD 操作（80%）
2. 保留复杂逻辑的自定义执行（20%）
3. 验证功能正常

### 阶段 3: 添加高级特性

1. 乐观更新标准化
2. 请求重试
3. 响应缓存
4. 性能监控

---

## 🎯 最佳实践

### ✅ 推荐使用声明式配置：

- 标准 CRUD 操作
- 单个网络请求
- 简单的 Store 更新

### ✅ 推荐使用自定义执行：

- 需要多个网络请求
- 复杂的条件逻辑
- 需要与旧系统集成

### ❌ 避免：

- 在声明式配置中写复杂逻辑
- 在自定义执行中不遵循标准模式（如不传 correlation-id）

---

## 📝 总结

**推荐方案**：混合架构（声明式为主，自定义为辅）

**核心优势**：

1. ✅ 80% 的指令使用声明式配置，减少重复代码
2. ✅ 20% 的复杂场景保留灵活性
3. ✅ 统一的请求入口，便于添加中间件
4. ✅ 清晰的关注点分离
5. ✅ 易于测试和追踪

**下一步**：

1. 实现 `executeRequest` 工具函数
2. 扩展类型定义
3. 更新 EX 阶段
4. 渐进式迁移现有指令

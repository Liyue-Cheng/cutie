# 指令迁移指南

> 如何将现有的 CommandBus 指令迁移到 CPU 流水线系统

## 📋 目录

1. [迁移概述](#迁移概述)
2. [迁移步骤](#迁移步骤)
3. [示例：task.complete 迁移](#示例taskcomplete-迁移)
4. [完整迁移清单](#完整迁移清单)
5. [测试验证](#测试验证)
6. [常见问题](#常见问题)

---

## 迁移概述

### 现有架构 (CommandBus)

```
组件 → commandBus.emit('task.complete', { id })
  → handler(payload)
  → API调用 + Store更新
```

**文件结构：**

```
src/commandBus/
├── types.ts              # Command类型定义
├── CommandBus.ts         # 总线实现
└── handlers/
    ├── taskHandlers.ts   # 任务处理器
    ├── scheduleHandlers.ts
    └── ...
```

### 新架构 (CPU Pipeline)

```
组件 → pipeline.dispatch('task.complete', { id })
  → IF → SCH → EX → RES → WB
  → 支持乐观更新、并发控制、指令追踪
```

**文件结构：**

```
src/cpu/
├── isa/
│   ├── task-isa.ts       # 任务指令集
│   ├── schedule-isa.ts
│   └── index.ts
└── ...
```

---

## 迁移步骤

### 步骤1：创建ISA文件

创建 `src/cpu/isa/task-isa.ts`：

```typescript
import type { ISADefinition } from './types'
import type { TaskCard, TaskTransactionResult } from '@/types/dtos'
import { apiPost, apiDelete, apiPatch } from '@/stores/shared'
import { useTaskStore } from '@/stores/task'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'

export const TaskISA: ISADefinition = {
  // 稍后添加指令...
}
```

### 步骤2：转换单个指令

#### 现有 CommandBus Handler

```typescript
// src/commandBus/handlers/taskHandlers.ts
const handleCompleteTask: CommandHandlerMap['task.complete'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiPost(
    `/tasks/${payload.id}/completion`,
    {},
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}
```

#### 转换为 CPU ISA

```typescript
// src/cpu/isa/task-isa.ts
export const TaskISA: ISADefinition = {
  'task.complete': {
    // ========== 元数据 ==========
    meta: {
      description: '完成任务',
      category: 'task',

      // 🔑 资源标识：同一任务的操作必须顺序执行
      resourceIdentifier: (payload) => [`task:${payload.id}`],

      priority: 7,
      timeout: 10000,
    },

    // ========== 前置验证 ==========
    validate: async (payload, context) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)

      if (!task) {
        console.error('任务不存在:', payload.id)
        return false
      }

      if (task.is_completed) {
        console.warn('任务已完成:', payload.id)
        return false
      }

      return true
    },

    // ========== 执行逻辑 ==========
    execute: async (payload, context) => {
      // 使用 context.correlationId，无需生成
      const result: TaskTransactionResult = await apiPost(
        `/tasks/${payload.id}/completion`,
        {},
        {
          headers: { 'X-Correlation-ID': context.correlationId },
        }
      )

      return result
    },

    // ========== 提交结果 ==========
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}
```

### 步骤3：注册到ISA

在 `src/cpu/isa/index.ts` 中导入：

```typescript
import { DebugISA } from './debug-isa'
import { TaskISA } from './task-isa'

export const ISA: ISADefinition = {
  ...DebugISA,
  ...TaskISA, // ← 添加
}
```

### 步骤4：更新组件调用

#### 旧代码

```vue
<script setup>
import { commandBus } from '@/commandBus'

async function handleComplete(taskId: string) {
  await commandBus.emit('task.complete', { id: taskId })
}
</script>
```

#### 新代码

```vue
<script setup>
import { pipeline } from '@/cpu'

function handleComplete(taskId: string) {
  pipeline.dispatch('task.complete', { id: taskId })
  // 注意：dispatch是同步的，立即返回
  // 实际执行是异步的，不阻塞UI
}
</script>
```

---

## 示例：task.complete 迁移

### 完整对比

<table>
<tr>
<th>CommandBus (旧)</th>
<th>CPU ISA (新)</th>
</tr>
<tr>
<td>

```typescript
// taskHandlers.ts
const handleCompleteTask = async (payload) => {
  const correlationId = generateCorrelationId()

  const result = await apiPost(
    `/tasks/${payload.id}/completion`,
    {},
    { headers: { 'X-Correlation-ID': correlationId } }
  )

  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}
```

</td>
<td>

```typescript
// task-isa.ts
'task.complete': {
  meta: {
    description: '完成任务',
    category: 'task',
    resourceIdentifier: (payload) => [`task:${payload.id}`],
    priority: 7,
    timeout: 10000,
  },

  validate: async (payload) => {
    const task = useTaskStore().getTaskById_Mux(payload.id)
    if (!task) return false
    if (task.is_completed) return false
    return true
  },

  execute: async (payload, context) => {
    return await apiPost(
      `/tasks/${payload.id}/completion`,
      {},
      { headers: { 'X-Correlation-ID': context.correlationId } }
    )
  },

  commit: async (result, payload, context) => {
    await transactionProcessor.applyTaskTransaction(result, {
      correlation_id: context.correlationId,
      source: 'http',
    })
  },
}
```

</td>
</tr>
</table>

### 关键差异

| 方面              | CommandBus      | CPU ISA                 |
| ----------------- | --------------- | ----------------------- |
| **correlationId** | 手动生成        | 自动提供                |
| **并发控制**      | 无              | 通过 resourceIdentifier |
| **前置验证**      | 手动在handler中 | 独立的 validate 函数    |
| **结果处理**      | 在handler中     | 独立的 commit 函数      |
| **追踪**          | 无              | 自动追踪各阶段          |
| **错误处理**      | try-catch       | RES阶段统一处理         |

---

## 完整迁移清单

### 任务指令 (task-isa.ts)

创建 `src/cpu/isa/task-isa.ts`：

```typescript
import type { ISADefinition } from './types'
import type { TaskCard, TaskTransactionResult } from '@/types/dtos'
import { apiPost, apiDelete, apiPatch } from '@/stores/shared'
import { useTaskStore } from '@/stores/task'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'

export const TaskISA: ISADefinition = {
  'task.complete': {
    meta: {
      description: '完成任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 7,
      timeout: 10000,
    },
    validate: async (payload) => {
      const task = useTaskStore().getTaskById_Mux(payload.id)
      return task && !task.is_completed
    },
    execute: async (payload, context) => {
      return await apiPost(
        `/tasks/${payload.id}/completion`,
        {},
        { headers: { 'X-Correlation-ID': context.correlationId } }
      )
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.reopen': {
    meta: {
      description: '重新打开任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 7,
      timeout: 10000,
    },
    validate: async (payload) => {
      const task = useTaskStore().getTaskById_Mux(payload.id)
      return task && task.is_completed
    },
    execute: async (payload, context) => {
      return await apiDelete(`/tasks/${payload.id}/completion`, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.delete': {
    meta: {
      description: '删除任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 5,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      return await apiDelete(`/tasks/${payload.id}`, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.create': {
    meta: {
      description: '创建任务',
      category: 'task',
      resourceIdentifier: () => [], // 创建操作无固定资源
      priority: 5,
      timeout: 10000,
    },
    validate: async (payload) => {
      return !!payload.title?.trim()
    },
    execute: async (payload, context) => {
      return await apiPost('/tasks', payload, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result) => {
      const taskStore = useTaskStore()
      taskStore.addOrUpdateTask_mut(result)
    },
  },

  'task.update': {
    meta: {
      description: '更新任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      return await apiPatch(`/tasks/${payload.id}`, payload.updates, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}
```

### 日程指令 (schedule-isa.ts)

创建 `src/cpu/isa/schedule-isa.ts`：

```typescript
import type { ISADefinition } from './types'
import { apiPost, apiPatch, apiDelete } from '@/stores/shared'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'

export const ScheduleISA: ISADefinition = {
  'schedule.create': {
    meta: {
      description: '创建日程',
      category: 'schedule',
      resourceIdentifier: (payload) => [
        `task:${payload.task_id}`,
        `schedule:${payload.task_id}:${payload.scheduled_day}`,
      ],
      priority: 6,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      return await apiPost('/schedules', payload, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'schedule.delete': {
    meta: {
      description: '删除日程',
      category: 'schedule',
      resourceIdentifier: (payload) => [
        `task:${payload.task_id}`,
        `schedule:${payload.task_id}:${payload.scheduled_day}`,
      ],
      priority: 6,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      return await apiDelete(`/schedules/${payload.task_id}/${payload.scheduled_day}`, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'schedule.update': {
    meta: {
      description: '更新日程',
      category: 'schedule',
      resourceIdentifier: (payload) => [
        `task:${payload.task_id}`,
        `schedule:${payload.task_id}:${payload.scheduled_day}`,
      ],
      priority: 6,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      return await apiPatch(
        `/schedules/${payload.task_id}/${payload.scheduled_day}`,
        payload.updates,
        {
          headers: { 'X-Correlation-ID': context.correlationId },
        }
      )
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}
```

### 时间块指令 (timeblock-isa.ts)

创建 `src/cpu/isa/timeblock-isa.ts`：

```typescript
import type { ISADefinition } from './types'
import type { TimeBlock, TaskTransactionResult } from '@/types/dtos'
import { apiPost, apiPatch, apiDelete } from '@/stores/shared'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'
import { useTimeBlockStore } from '@/stores/timeblock'

export const TimeBlockISA: ISADefinition = {
  'time_block.create': {
    meta: {
      description: '创建时间块',
      category: 'timeblock',
      resourceIdentifier: (payload) => (payload.task_id ? [`task:${payload.task_id}`] : []),
      priority: 6,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      return await apiPost('/time-blocks', payload, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'time_block.update': {
    meta: {
      description: '更新时间块',
      category: 'timeblock',
      resourceIdentifier: (payload) => [`timeblock:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      return await apiPatch(`/time-blocks/${payload.id}`, payload.updates, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'time_block.delete': {
    meta: {
      description: '删除时间块',
      category: 'timeblock',
      resourceIdentifier: (payload) => [`timeblock:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      return await apiDelete(`/time-blocks/${payload.id}`, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
    },
    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}
```

### 更新 ISA 索引

修改 `src/cpu/isa/index.ts`：

```typescript
import type { ISADefinition } from './types'
import { DebugISA } from './debug-isa'
import { TaskISA } from './task-isa'
import { ScheduleISA } from './schedule-isa'
import { TimeBlockISA } from './timeblock-isa'

export const ISA: ISADefinition = {
  ...DebugISA,
  ...TaskISA,
  ...ScheduleISA,
  ...TimeBlockISA,
}

export type { InstructionDefinition, InstructionMeta, ISADefinition } from './types'
```

---

## 测试验证

### 1. 单元测试

创建 `src/cpu/isa/__tests__/task-isa.test.ts`：

```typescript
import { describe, it, expect, vi } from 'vitest'
import { TaskISA } from '../task-isa'

describe('TaskISA', () => {
  describe('task.complete', () => {
    it('应该正确提取资源ID', () => {
      const instruction = TaskISA['task.complete']
      const resourceIds = instruction.meta.resourceIdentifier({ id: 'task-123' })

      expect(resourceIds).toEqual(['task:task-123'])
    })

    it('应该验证任务存在且未完成', async () => {
      const instruction = TaskISA['task.complete']

      // Mock store
      vi.mock('@/stores/task', () => ({
        useTaskStore: () => ({
          getTaskById_Mux: (id) => ({ id, is_completed: false }),
        }),
      }))

      const isValid = await instruction.validate({ id: 'task-123' }, mockContext)
      expect(isValid).toBe(true)
    })
  })
})
```

### 2. 集成测试

在 CPU Debug 页面测试：

```typescript
// 测试脚本
pipeline.start()

// 测试1: 单个任务完成
pipeline.dispatch('task.complete', { id: 'task-1' })

// 测试2: 并发任务完成（不同任务）
pipeline.dispatch('task.complete', { id: 'task-1' })
pipeline.dispatch('task.complete', { id: 'task-2' }) // 应该并行

// 测试3: 冲突检测（同一任务）
pipeline.dispatch('task.complete', { id: 'task-1' })
pipeline.dispatch('task.update', { id: 'task-1', updates: { title: 'new' } }) // 应该等待
```

### 3. 性能对比

```typescript
// CommandBus (旧)
console.time('commandBus')
await commandBus.emit('task.complete', { id: 'task-1' })
await commandBus.emit('task.complete', { id: 'task-2' })
await commandBus.emit('task.complete', { id: 'task-3' })
console.timeEnd('commandBus') // ~3000ms (顺序执行)

// CPU Pipeline (新)
console.time('cpuPipeline')
pipeline.dispatch('task.complete', { id: 'task-1' })
pipeline.dispatch('task.complete', { id: 'task-2' })
pipeline.dispatch('task.complete', { id: 'task-3' })
// 等待所有完成...
console.timeEnd('cpuPipeline') // ~1000ms (并行执行)
```

---

## 常见问题

### Q: 需要迁移所有指令吗？

**A:** 不需要。可以逐步迁移：

1. 先迁移核心、高频的指令（如 task.complete）
2. CommandBus 和 CPU Pipeline 可以共存
3. 组件可以选择使用哪个系统

### Q: 如何处理需要返回值的指令？

**A:** CPU Pipeline 的 `dispatch()` 是同步的，不返回值。如果需要返回值：

**方案1：使用追踪系统**

```typescript
const instrId = pipeline.dispatch('task.create', { title: '新任务' })
const trace = instructionTracker.getTrace(instrId)
// 监听trace.status变化
```

**方案2：保留 CommandBus**

```typescript
// 需要返回值的操作继续使用 CommandBus
const task = await commandBus.emit('task.create', { title: '新任务' })

// 不需要返回值的操作使用 Pipeline（更快）
pipeline.dispatch('task.complete', { id: task.id })
```

### Q: resourceIdentifier 如何设计？

**A:** 遵循以下原则：

```typescript
// 原则1：同一资源的操作要冲突
resourceIdentifier: (payload) => [`task:${payload.id}`]
// task:123 的所有操作都会冲突，顺序执行

// 原则2：不同资源不冲突
'task.complete': { resourceIdentifier: (p) => [`task:${p.id}`] }
'task.complete': { id: 'task-1' }  // 资源: ['task:task-1']
'task.complete': { id: 'task-2' }  // 资源: ['task:task-2']  ← 并行

// 原则3：关联资源都要列出
'schedule.create': {
  resourceIdentifier: (payload) => [
    `task:${payload.task_id}`,      // 任务资源
    `schedule:${payload.task_id}:${payload.scheduled_day}`,  // 日程资源
  ]
}

// 原则4：创建操作通常无资源
'task.create': { resourceIdentifier: () => [] }  // 无冲突，可以并行创建
```

### Q: validate 是必需的吗？

**A:** 不是必需的，但强烈推荐：

```typescript
// 没有 validate
'task.complete': {
  execute: async (payload) => {
    // API会返回404，但已经发送了网络请求
    return await apiPost(`/tasks/${payload.id}/completion`)
  }
}

// 有 validate
'task.complete': {
  validate: async (payload) => {
    const task = useTaskStore().getTaskById_Mux(payload.id)
    return task && !task.is_completed  // ← 提前检查，避免无效请求
  },
  execute: async (payload) => {
    return await apiPost(`/tasks/${payload.id}/completion`)
  }
}
```

### Q: 如何处理依赖关系？

**A:** 通过资源ID实现：

```typescript
// 场景：必须先完成任务，才能归档
'task.complete': { resourceIdentifier: (p) => [`task:${p.id}`] }
'task.archive': { resourceIdentifier: (p) => [`task:${p.id}`] }

// 手动控制顺序
pipeline.dispatch('task.complete', { id: 'task-1' })
pipeline.dispatch('task.archive', { id: 'task-1' })
// ✅ archive会等待complete完成

// 或者在validate中检查
'task.archive': {
  validate: async (payload) => {
    const task = useTaskStore().getTaskById_Mux(payload.id)
    return task?.is_completed  // ← 必须已完成
  }
}
```

---

## 迁移检查清单

- [ ] 创建 `task-isa.ts` 文件
- [ ] 迁移 `task.complete`
- [ ] 迁移 `task.create`
- [ ] 迁移 `task.update`
- [ ] 迁移 `task.delete`
- [ ] 迁移 `task.reopen`
- [ ] 创建 `schedule-isa.ts` 文件
- [ ] 迁移所有 schedule 指令
- [ ] 创建 `timeblock-isa.ts` 文件
- [ ] 迁移所有 timeblock 指令
- [ ] 更新 `isa/index.ts` 导入
- [ ] 更新组件调用（逐步）
- [ ] 编写单元测试
- [ ] 进行集成测试
- [ ] 性能对比测试
- [ ] 更新文档

---

## 下一步

1. **阶段1**：迁移 task.complete（高频操作）
2. **阶段2**：迁移其他任务指令
3. **阶段3**：迁移 schedule 和 timeblock
4. **阶段4**：逐步更新组件调用
5. **阶段5**：移除旧的 CommandBus（可选）

---

**Made with 🚀 by CPU Pipeline System**

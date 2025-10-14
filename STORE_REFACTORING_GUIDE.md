# Store 架构重构指南

> **重构日期**: 2024-10-14  
> **版本**: Task Store V3.0  
> **状态**: ✅ 核心架构完成

---

## 📋 重构目标

**让 Store 只负责数据管理，API 调用由 Command Handler 负责**

### 重构前（V2）

```typescript
组件 → commandBus → Handler → Store.method()
                                    ↓
                              [内部调用API + 更新状态]
```

### 重构后（V3）

```typescript
组件 → commandBus → Handler
                      ↓
                    调用 API
                      ↓
                    Store.mutation() // 纯数据操作
                      ↓
                    组件响应式更新
```

---

## 🏗️ 架构原则

### Store 的职责（更纯粹）

✅ **应该做的**：

- 存储数据（`tasks: Map<string, TaskCard>`）
- 提供 Getters（计算属性）
- 提供 Mutations（纯数据操作）

❌ **不应该做的**：

- 调用 API
- 包含复杂业务逻辑
- 直接处理用户操作

### Handler 的职责（更强大）

✅ **应该做的**：

- 接收命令
- 调用 API 客户端
- 编排业务逻辑
- 调用 Store 的 mutation 更新状态
- 统一的错误处理和日志

---

## 📁 新架构文件结构

```
src/stores/task/
├── index.ts              # Store 定义和导出
├── core.ts               # 核心状态和 Getters
├── mutations.ts          # 纯数据操作方法 ⭐ 新增
├── view-operations.ts    # 视图加载（保留用于初始化）
├── crud-operations.ts    # CRUD操作（保留向后兼容，逐步废弃）
└── event-handlers.ts     # SSE 事件处理

src/services/commandBus/handlers/
└── taskHandlers.ts       # 命令处理器（调用API）⭐ 重构
```

---

## 🎯 核心变化

### 1. 新增 Mutations 层（纯数据操作）

```typescript
// src/stores/task/mutations.ts

export function createMutations(core) {
  /**
   * 添加或更新任务（纯数据操作）
   * ❌ 不调用 API
   * ✅ 只修改内存中的数据
   */
  function addOrUpdateTask_mut(task: TaskCard): void {
    tasks.value.set(task.id, task)
    logger.debug('Task added/updated in store')
  }

  /**
   * 移除任务（纯数据操作）
   */
  function removeTask_mut(taskId: string): void {
    tasks.value.delete(taskId)
    logger.debug('Task removed from store')
  }

  // ... 其他纯数据操作

  return {
    addOrUpdateTask_mut,
    removeTask_mut,
    // ...
  }
}
```

### 2. Handler 直接调用 API

```typescript
// src/services/commandBus/handlers/taskHandlers.ts

const handleCompleteTask = async (payload) => {
  logger.debug('Handling task.complete', { payload })

  // 1. 调用 API（Handler 负责）
  const result = await apiPost(`/tasks/${payload.id}/complete`)
  const task: TaskCard = result.task

  // 2. 更新 Store（纯数据操作）
  const taskStore = useTaskStore()
  taskStore.addOrUpdateTask_mut(task)

  logger.info('Task completed', { taskId: task.id })
}
```

### 3. Store 导出 Mutations

```typescript
// src/stores/task/index.ts

export const useTaskStore = defineStore('task', () => {
  const core = createTaskCore()
  const mutations = createMutations(core) // 新增

  return {
    // State & Getters
    tasks: core.tasks,
    allTasks: core.allTasks,
    getTaskById: core.getTaskById,
    // ...

    // ⭐ Mutations (新增)
    addOrUpdateTask_mut: mutations.addOrUpdateTask_mut,
    removeTask_mut: mutations.removeTask_mut,
    batchAddOrUpdateTasks_mut: mutations.batchAddOrUpdateTasks_mut,
    patchTask_mut: mutations.patchTask_mut,
    // ...

    // Actions (保留向后兼容)
    createTask: crudOps.createTask, // 逐步废弃
    completeTask: crudOps.completeTask, // 逐步废弃
    // ...
  }
})
```

---

## 📊 对比分析

### 代码对比

#### 旧架构（Handler 调用 Store 方法）

```typescript
// Handler
const handleCompleteTask = async (payload) => {
  const taskStore = useTaskStore()
  const task = await taskStore.completeTask(payload.id) // Store内部调用API

  if (!task) {
    throw new Error('Failed to complete task')
  }
}

// Store
async function completeTask(id: string) {
  // 调用API
  const result = await apiPost(`/tasks/${id}/complete`)
  const task = result.task

  // 更新状态
  tasks.value.set(task.id, task)

  return task
}
```

**问题**：

- Store 承担了太多职责（API + 状态）
- 难以测试（需要 mock API）
- 业务逻辑分散

#### 新架构（Handler 调用 API + Mutation）

```typescript
// Handler（承担业务逻辑）
const handleCompleteTask = async (payload) => {
  // 1. 调用API
  const result = await apiPost(`/tasks/${payload.id}/complete`)
  const task = result.task

  // 2. 更新Store
  const taskStore = useTaskStore()
  taskStore.addOrUpdateTask_mut(task)

  logger.info('Task completed')
}

// Store Mutation（纯数据操作）
function addOrUpdateTask_mut(task: TaskCard): void {
  tasks.value.set(task.id, task)
}
```

**优势**：

- ✅ Store 职责单一（只管数据）
- ✅ 业务逻辑集中在 Handler
- ✅ 更容易测试（可以单独测试 mutation）
- ✅ 更清晰的数据流

---

## 🔄 迁移策略

### 阶段 1：渐进式重构（当前）

**保留旧 API**：

```typescript
// Store 同时提供新旧两种方式
export const useTaskStore = defineStore('task', () => {
  return {
    // 新架构（推荐）
    addOrUpdateTask_mut: mutations.addOrUpdateTask_mut,
    removeTask_mut: mutations.removeTask_mut,

    // 旧架构（向后兼容）
    createTask: crudOps.createTask, // 标记为 @deprecated
    completeTask: crudOps.completeTask, // 标记为 @deprecated
  }
})
```

**优势**：

- ✅ 不影响现有代码
- ✅ 新代码使用新架构
- ✅ 逐步迁移旧代码

### 阶段 2：完全迁移（未来）

当所有组件都改用 Command Bus 后：

1. 移除 `crud-operations.ts` 中的 API 调用方法
2. 只保留数据加载方法（如 `fetchAllTasks`）
3. Store 成为纯粹的状态容器

---

## 💡 使用示例

### 在组件中使用

```vue
<script setup lang="ts">
import { commandBus } from '@/commandBus'
import { useTaskStore } from '@/stores/task'

// ✅ 读取数据：直接从 Store
const taskStore = useTaskStore()
const tasks = taskStore.allTasks

// ✅ 写入数据：通过 commandBus
async function handleComplete(taskId: string) {
  await commandBus.emit('task.complete', { id: taskId })
  // Store 会自动更新，UI 响应式刷新
}
</script>
```

### 在 Handler 中使用

```typescript
// src/services/commandBus/handlers/taskHandlers.ts

import { apiPost } from '@/stores/shared'
import { useTaskStore } from '@/stores/task'

const handleCompleteTask = async (payload) => {
  // 1. 调用 API
  const result = await apiPost(`/tasks/${payload.id}/complete`)

  // 2. 调用 Store mutation 更新数据
  const taskStore = useTaskStore()
  taskStore.addOrUpdateTask_mut(result.task)
}
```

### 在测试中使用

```typescript
// 测试 Mutation（不需要 mock API）
describe('TaskStore Mutations', () => {
  it('should add task to store', () => {
    const store = useTaskStore()
    const task = { id: '123', title: 'Test' }

    store.addOrUpdateTask_mut(task)

    expect(store.getTaskById('123')).toEqual(task)
  })
})

// 测试 Handler（需要 mock API）
describe('TaskHandlers', () => {
  it('should handle task.complete', async () => {
    const apiMock = vi.fn().mockResolvedValue({ task: mockTask })

    await handleCompleteTask({ id: '123' })

    expect(apiMock).toHaveBeenCalledWith('/tasks/123/complete')
  })
})
```

---

## 📈 Benefits（收益）

### 1. 职责更清晰

| 层级        | 旧架构            | 新架构                       |
| ----------- | ----------------- | ---------------------------- |
| **组件**    | UI + 调用 Store   | UI + 发送命令                |
| **Handler** | 分发命令          | 分发命令 + API + 业务逻辑 ✨ |
| **Store**   | 数据 + API + 逻辑 | 只管数据 ✨                  |

### 2. 更易测试

```typescript
// 旧架构：需要 mock API
test('completeTask', async () => {
  mockApi('/tasks/123/complete') // 复杂
  await taskStore.completeTask('123')
})

// 新架构：测试 mutation 不需要 mock
test('addOrUpdateTask_mut', () => {
  taskStore.addOrUpdateTask_mut(task) // 简单
  expect(taskStore.getTaskById('123')).toEqual(task)
})
```

### 3. 更好的扩展性

```typescript
// 轻松添加额外逻辑（如缓存、重试）
const handleCompleteTask = async (payload) => {
  // 1. 乐观更新
  taskStore.patchTask_mut(payload.id, { is_completed: true })

  try {
    // 2. 调用API
    const result = await apiPost(`/tasks/${payload.id}/complete`)

    // 3. 确认更新
    taskStore.addOrUpdateTask_mut(result.task)
  } catch (error) {
    // 4. 回滚
    taskStore.patchTask_mut(payload.id, { is_completed: false })
    throw error
  }
}
```

---

## 🎯 待办事项

### 短期（1周）

- [ ] 更新所有 Handler 使用新架构
- [ ] 添加 `@deprecated` 标记到旧的 CRUD 方法
- [ ] 更新文档和示例

### 中期（1月）

- [ ] 迁移所有组件使用 Command Bus
- [ ] 移除旧的 CRUD 方法（在 Store 中）
- [ ] 添加单元测试覆盖 Mutations

### 长期（可选）

- [ ] 支持乐观更新（在 Handler 层实现）
- [ ] 支持离线队列（在 Handler 层实现）
- [ ] 实现时间旅行调试（基于纯 Mutations）

---

## 📚 相关文档

- [COMMAND_BUS_IMPLEMENTATION.md](./COMMAND_BUS_IMPLEMENTATION.md) - 命令总线实现
- [src/services/commandBus/README.md](./src/services/commandBus/README.md) - 命令总线使用指南
- [src/stores/task/mutations.ts](./src/stores/task/mutations.ts) - Mutations 实现

---

**版本**: 3.0  
**最后更新**: 2024-10-14  
**作者**: AI Assistant

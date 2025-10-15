# 🎯 Awaitable Dispatch - 等待指令结果

## 📖 概述

现在 `pipeline.dispatch()` 返回一个 **Promise**，你可以 `await` 它来等待指令执行完成并获取结果！

## ✨ 功能特性

- ✅ **返回 Promise**：可以 `await` 等待结果
- ✅ **类型安全**：支持泛型指定返回类型
- ✅ **错误处理**：失败时 Promise 会 reject
- ✅ **向后兼容**：不 await 也能正常工作（fire-and-forget）
- ✅ **自动清理**：指令完成后自动清理 Promise

## 🚀 使用示例

### 1. 基础用法：等待指令完成

```typescript
import { pipeline } from '@/cpu'

async function createTask() {
  try {
    // 🎯 await 等待指令完成
    const result = await pipeline.dispatch('task.create', {
      title: '新任务',
    })

    console.log('任务创建成功！', result)
    // result: { task: { id: 'xxx', title: '新任务', ... }, side_effects: [...] }
  } catch (error) {
    console.error('任务创建失败', error)
  }
}
```

### 2. 获取返回数据

```typescript
// 创建任务后立即获取任务 ID
async function createAndGetId() {
  const result = await pipeline.dispatch('task.create', {
    title: '我的任务',
  })

  const taskId = result.task.id
  console.log('新任务 ID:', taskId)

  return taskId
}
```

### 3. 链式操作

```typescript
// 创建任务 → 添加标签 → 设置日期
async function createTaskWithDetails() {
  try {
    // 步骤 1: 创建任务
    const createResult = await pipeline.dispatch('task.create', {
      title: '重要任务',
    })

    const taskId = createResult.task.id

    // 步骤 2: 添加标签（假设有这个指令）
    await pipeline.dispatch('task.addTag', {
      id: taskId,
      tag: '紧急',
    })

    // 步骤 3: 设置到今天
    await pipeline.dispatch('schedule.create', {
      task_id: taskId,
      scheduled_day: new Date().toISOString().split('T')[0],
    })

    console.log('✅ 任务创建并配置完成！')
  } catch (error) {
    console.error('❌ 操作失败', error)
  }
}
```

### 4. 在 Vue 组件中使用

```vue
<template>
  <div>
    <input v-model="newTaskTitle" @keyup.enter="createTask" />
    <button @click="createTask" :disabled="isCreating">
      {{ isCreating ? '创建中...' : '创建任务' }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { pipeline } from '@/cpu'

const newTaskTitle = ref('')
const isCreating = ref(false)

async function createTask() {
  if (!newTaskTitle.value.trim()) return

  isCreating.value = true

  try {
    // 🎯 等待指令完成
    const result = await pipeline.dispatch('task.create', {
      title: newTaskTitle.value.trim(),
    })

    // ✅ 成功：显示通知
    console.log('✅ 任务创建成功！', result.task)
    newTaskTitle.value = '' // 清空输入
  } catch (error) {
    // ❌ 失败：显示错误
    console.error('❌ 任务创建失败', error)
    alert(`创建失败：${error.message}`)
  } finally {
    isCreating.value = false
  }
}
</script>
```

### 5. 错误处理

```typescript
async function handleTaskOperation() {
  try {
    await pipeline.dispatch('task.complete', {
      id: 'task-123',
    })

    // ✅ 成功后的操作
    console.log('任务已完成')
    showSuccessNotification('任务完成！')
  } catch (error) {
    // ❌ 错误处理
    if (error.message.includes('database is locked')) {
      console.error('数据库锁定，请稍后重试')
    } else if (error.message.includes('Not found')) {
      console.error('任务不存在')
    } else {
      console.error('未知错误', error)
    }

    showErrorNotification(error.message)
  }
}
```

### 6. 类型安全的返回值

```typescript
import { pipeline } from '@/cpu'
import type { TaskTransactionResult } from '@/types'

async function createTaskTypeSafe() {
  // 🎯 指定返回类型
  const result = await pipeline.dispatch<
    { title: string }, // Payload 类型
    TaskTransactionResult // Result 类型
  >('task.create', {
    title: '类型安全的任务',
  })

  // TypeScript 知道 result 的类型
  const task = result.task // ✅ 有类型提示
  const sideEffects = result.side_effects // ✅ 有类型提示
}
```

### 7. 并发操作

```typescript
// 同时创建多个任务
async function createMultipleTasks() {
  const titles = ['任务 1', '任务 2', '任务 3']

  try {
    // 🚀 并发发射所有指令
    const results = await Promise.all(
      titles.map((title) => pipeline.dispatch('task.create', { title }))
    )

    console.log(`✅ 成功创建 ${results.length} 个任务`)
    results.forEach((result) => {
      console.log(`  - ${result.task.title} (${result.task.id})`)
    })
  } catch (error) {
    console.error('❌ 某个任务创建失败', error)
  }
}
```

### 8. Fire-and-Forget（向后兼容）

```typescript
// 不需要等待结果时，直接调用（和之前一样）
function quickCreate() {
  // 🔥 不 await，立即返回
  pipeline.dispatch('task.create', {
    title: '快速任务',
  })

  // 代码继续执行，不等待
  console.log('指令已发射')
}

// 但你可以选择处理错误
function quickCreateWithErrorHandling() {
  pipeline
    .dispatch('task.create', {
      title: '快速任务',
    })
    .catch((error) => {
      console.error('创建失败', error)
    })
}
```

## 🎨 高级用法

### 1. 带超时的等待

```typescript
async function createTaskWithTimeout() {
  const timeoutPromise = new Promise((_, reject) => {
    setTimeout(() => reject(new Error('操作超时')), 5000)
  })

  try {
    const result = await Promise.race([
      pipeline.dispatch('task.create', { title: '任务' }),
      timeoutPromise,
    ])

    console.log('创建成功', result)
  } catch (error) {
    if (error.message === '操作超时') {
      console.error('操作超时，请检查网络')
    } else {
      console.error('创建失败', error)
    }
  }
}
```

### 2. 重试机制

```typescript
async function createTaskWithRetry(maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const result = await pipeline.dispatch('task.create', {
        title: '重要任务',
      })

      console.log('✅ 创建成功')
      return result
    } catch (error) {
      console.warn(`尝试 ${i + 1}/${maxRetries} 失败`, error)

      if (i === maxRetries - 1) {
        throw new Error(`重试 ${maxRetries} 次后仍然失败`)
      }

      // 等待一段时间后重试
      await new Promise((resolve) => setTimeout(resolve, 1000 * (i + 1)))
    }
  }
}
```

### 3. 条件执行

```typescript
async function updateTaskConditional(taskId: string) {
  try {
    // 先完成任务
    const completeResult = await pipeline.dispatch('task.complete', {
      id: taskId,
    })

    // 根据结果决定是否归档
    if (completeResult.task.is_completed) {
      await pipeline.dispatch('task.archive', {
        id: taskId,
      })
      console.log('✅ 任务已完成并归档')
    }
  } catch (error) {
    console.error('操作失败', error)
  }
}
```

### 4. 进度追踪

```typescript
async function batchOperation(taskIds: string[]) {
  let completed = 0
  const total = taskIds.length

  for (const taskId of taskIds) {
    try {
      await pipeline.dispatch('task.complete', { id: taskId })
      completed++

      console.log(`进度: ${completed}/${total} (${((completed / total) * 100).toFixed(0)}%)`)

      // 更新 UI 进度条
      updateProgressBar(completed / total)
    } catch (error) {
      console.error(`任务 ${taskId} 处理失败`, error)
    }
  }

  console.log(`✅ 完成 ${completed}/${total} 个任务`)
}
```

## 🔧 与 Composables 集成

```typescript
// composables/useTaskOperations.ts
import { pipeline } from '@/cpu'
import { ref } from 'vue'

export function useTaskOperations() {
  const isLoading = ref(false)
  const error = ref<Error | null>(null)

  async function createTask(title: string) {
    isLoading.value = true
    error.value = null

    try {
      const result = await pipeline.dispatch('task.create', { title })
      return result.task
    } catch (err) {
      error.value = err as Error
      throw err
    } finally {
      isLoading.value = false
    }
  }

  async function completeTask(id: string) {
    isLoading.value = true
    error.value = null

    try {
      await pipeline.dispatch('task.complete', { id })
    } catch (err) {
      error.value = err as Error
      throw err
    } finally {
      isLoading.value = false
    }
  }

  return {
    isLoading,
    error,
    createTask,
    completeTask,
  }
}
```

使用：

```vue
<script setup lang="ts">
import { useTaskOperations } from '@/composables/useTaskOperations'

const { isLoading, error, createTask } = useTaskOperations()

async function handleCreate() {
  try {
    const task = await createTask('新任务')
    console.log('创建成功', task)
  } catch (error) {
    console.error('创建失败', error)
  }
}
</script>

<template>
  <div>
    <button @click="handleCreate" :disabled="isLoading">
      {{ isLoading ? '创建中...' : '创建任务' }}
    </button>
    <div v-if="error" class="error">{{ error.message }}</div>
  </div>
</template>
```

## ⚠️ 注意事项

### 1. 不要忘记错误处理

```typescript
// ❌ 不好：忽略错误
await pipeline.dispatch('task.create', { title: '任务' })

// ✅ 好：处理错误
try {
  await pipeline.dispatch('task.create', { title: '任务' })
} catch (error) {
  console.error('创建失败', error)
}
```

### 2. 避免阻塞 UI

```typescript
// ❌ 不好：同步等待多个指令（阻塞 UI）
for (const id of taskIds) {
  await pipeline.dispatch('task.complete', { id })
}

// ✅ 好：并发执行
await Promise.all(taskIds.map((id) => pipeline.dispatch('task.complete', { id })))
```

### 3. 流水线必须运行

```typescript
// 如果流水线未启动，Promise 会立即 reject
try {
  await pipeline.dispatch('task.create', { title: '任务' })
} catch (error) {
  if (error.message === 'Pipeline is not running') {
    console.error('流水线未启动，请先调用 pipeline.start()')
  }
}
```

## 🎉 总结

- ✅ **可等待**：`await pipeline.dispatch()` 等待结果
- ✅ **可获取**：获取指令返回的数据
- ✅ **可处理**：捕获错误并处理
- ✅ **可链式**：一个指令完成后执行下一个
- ✅ **向后兼容**：不 await 也能用

这让你的代码更加：

- 🎯 **可预测**：知道指令何时完成
- 🔒 **可靠**：正确处理错误情况
- 📦 **可组合**：轻松构建复杂的业务逻辑
- 💪 **类型安全**：TypeScript 支持完整

享受新的 CPU Pipeline 带来的强大功能吧！🚀

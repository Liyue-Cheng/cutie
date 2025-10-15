# View Preference 乐观更新使用指南

## 快速开始

### 基础用法

```typescript
import { commandBus } from '@/commandBus'
import { useViewStore } from '@/stores/view'

const viewStore = useViewStore()

// 获取当前任务列表
const currentTasks = viewStore.applySorting(tasks.value, viewKey)

// 获取原始顺序（用于回滚）
const originalOrder = viewStore.getSortedTaskIds(viewKey, currentTasks)

// 新的顺序
const newOrder = ['task-1', 'task-2', 'task-3']

// 🔥 发送命令（乐观更新）
await commandBus.emit('view.update_sorting', {
  view_key: viewKey,
  sorted_task_ids: newOrder,
  original_sorted_task_ids: originalOrder, // 用于失败回滚
})
```

## 实际场景示例

### 场景 1: 拖拽排序

```typescript
function handleDrop(session: DragSession) {
  // 计算新顺序
  const newOrder = calculateNewOrder(session)

  // 获取原始顺序
  const originalOrder = viewStore.getSortedTaskIds(props.viewKey, tasks.value)

  // 🔥 乐观更新
  commandBus
    .emit('view.update_sorting', {
      view_key: props.viewKey,
      sorted_task_ids: newOrder,
      original_sorted_task_ids: originalOrder,
    })
    .catch((error) => {
      // 失败时显示友好提示（状态已自动回滚）
      showErrorMessage('排序保存失败，已恢复原状态')
    })
}
```

### 场景 2: 完成任务重排

```typescript
function moveCompletedTaskToBottom(completedTaskId: string) {
  const currentOrder = displayTasks.value.map((t) => t.id)

  // 移除完成的任务
  const newOrder = currentOrder.filter((id) => id !== completedTaskId)

  // 添加到最后
  newOrder.push(completedTaskId)

  // 获取原始顺序
  const originalOrder = viewStore.getSortedTaskIds(props.viewKey, tasks.value)

  // 🔥 乐观更新
  commandBus
    .emit('view.update_sorting', {
      view_key: props.viewKey,
      sorted_task_ids: newOrder,
      original_sorted_task_ids: originalOrder,
    })
    .catch((error) => {
      logger.error('Failed to persist completed task reorder', error)
    })
}
```

### 场景 3: 自动持久化

```typescript
watch(
  () => tasks.value,
  (newTasks) => {
    const currentOrder = newTasks.map((t) => t.id)
    const originalOrder = viewStore.getSortedTaskIds(props.viewKey, previousTasks.value)

    // 🔥 自动持久化（乐观更新）
    commandBus
      .emit('view.update_sorting', {
        view_key: props.viewKey,
        sorted_task_ids: currentOrder,
        original_sorted_task_ids: originalOrder,
      })
      .catch((error) => {
        logger.error('Failed to auto-persist view tasks', error)
      })
  },
  { deep: false }
)
```

## 迁移指南

### 从旧 API 迁移

**旧方式** (ViewStore V4.0):

```typescript
// ❌ 直接调用 Store 的 API 方法
await viewStore.updateSorting(viewKey, newOrder)
```

**新方式** (ViewStore V5.0):

```typescript
// ✅ 使用 Command Bus（乐观更新）
const originalOrder = viewStore.getSortedTaskIds(viewKey, tasks.value)
await commandBus.emit('view.update_sorting', {
  view_key: viewKey,
  sorted_task_ids: newOrder,
  original_sorted_task_ids: originalOrder,
})
```

### 为什么要迁移？

| 特性         | 旧方式    | 新方式           |
| ------------ | --------- | ---------------- |
| 用户感知延迟 | 300ms     | **0ms** ✅       |
| 网络失败处理 | ❌ 无反馈 | ✅ 自动回滚      |
| 操作追踪     | ❌ 无     | ✅ 完整日志      |
| 架构一致性   | ❌ 不符合 | ✅ 符合 CPU 架构 |

## 注意事项

### ⚠️ 必须提供 `original_sorted_task_ids`

```typescript
// ✅ 推荐：提供原始顺序
commandBus.emit('view.update_sorting', {
  view_key: viewKey,
  sorted_task_ids: newOrder,
  original_sorted_task_ids: originalOrder, // 🔥 用于回滚
})

// ⚠️ 不推荐：未提供原始顺序
commandBus.emit('view.update_sorting', {
  view_key: viewKey,
  sorted_task_ids: newOrder,
  // 失败时会清除该视图的排序（而不是回滚）
})
```

### 💡 如何获取原始顺序

```typescript
// 方法 1：从 Store 获取当前排序
const originalOrder = viewStore.getSortedTaskIds(viewKey, tasks.value)

// 方法 2：从当前显示列表获取
const originalOrder = displayTasks.value.map((t) => t.id)

// 方法 3：在操作前缓存
const originalOrder = tasks.value.map((t) => t.id)
// ... 进行操作
const newOrder = [...originalOrder]
newOrder.splice(fromIndex, 1)
newOrder.splice(toIndex, 0, movedTaskId)
```

## 调试技巧

### 查看乐观更新日志

开发环境下，打开控制台即可看到完整的指令流水线：

```
[00:12:34] [DEBUG] 🎯 [IF] Instruction: command.view.update_sorting
[00:12:34] [DEBUG] ⚡ [WB-Optimistic] Optimistic update to ViewStore
[00:12:34] [DEBUG] 🔧 [EX] Execute: view.update_sorting
[00:12:34] [INFO] ✅ Pipeline Complete: command.view.update_sorting
```

### 模拟网络失败

```typescript
// 在 API 层模拟失败
if (Math.random() < 0.5) {
  throw new Error('Network timeout')
}
```

观察控制台：

```
[00:12:40] [ERROR] ❌ [ROLLBACK] Failed to update view sorting, rolling back
[00:12:40] [DEBUG] 🔄 Rollback complete
```

## 常见问题

### Q: 为什么我的排序没有保存？

**A**: 检查以下几点：

1. 是否正确传递了 `view_key`？
2. 是否使用了正确的 viewKey 格式（如 `daily::2025-10-15`）？
3. 控制台是否有错误日志？

### Q: 乐观更新后为什么又变回去了？

**A**: 可能的原因：

1. 网络请求失败，触发了自动回滚
2. 查看控制台的 `[ROLLBACK]` 日志
3. 检查网络连接和后端服务

### Q: 如何禁用乐观更新？

**A**: 不推荐禁用乐观更新，但如果确实需要：

```typescript
// 不使用 Command Bus，直接调用 API（不推荐）
// 注意：这会失去所有乐观更新的好处
await apiPut(`/view-preferences/${encodeURIComponent(viewKey)}`, {
  sorted_task_ids: newOrder,
})
```

### Q: 乐观更新会影响性能吗？

**A**: 不会，反而提升了性能：

- 用户感知延迟：300ms → 0ms
- 状态更新开销：<10ms（本地操作）
- 回滚开销：<10ms（仅失败时）

## 相关文档

- [VIEW_PREFERENCE_CPU_ARCHITECTURE.md](../VIEW_PREFERENCE_CPU_ARCHITECTURE.md) - 完整架构设计
- [FRONTEND_CPU_ARCHITECTURE.md](../FRONTEND_CPU_ARCHITECTURE.md) - 类 CPU 架构说明
- [VIEW_CONTEXT_KEY_SPEC.md](../VIEW_CONTEXT_KEY_SPEC.md) - viewKey 规范

---

**版本**: 1.0  
**最后更新**: 2025-10-15

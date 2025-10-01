# 前端事件处理幂等性改进建议

**日期**: 2025-10-01  
**状态**: 建议 (未实施)

---

## 📊 当前状态分析

### 1. 隐式幂等性（已有）

**优点**:

- `Map.delete(id)` 天然幂等
- 简单直接，不需要额外逻辑

**不足**:

- 未使用 `aggregate_version` 进行版本控制
- 无法防止事件乱序问题
- 没有重复事件检测

### 2. 潜在问题场景

#### 场景 1: 事件乱序

```
时序：
1. 后端发送: task.completed (version: 100)
2. 后端发送: task.deleted (version: 101)
3. 网络延迟，前端收到顺序：
   - task.deleted (version: 101) ← 先到
   - task.completed (version: 100) ← 后到

结果：任务被标记为完成而不是删除 ❌
```

#### 场景 2: 重复事件

```
1. 网络抖动导致 SSE 重连
2. 事件分发器重新推送未确认的事件
3. 前端收到重复的 time_blocks.deleted 事件
4. 对同一个时间块进行多次删除操作

当前实现：多次调用 Map.delete(id) ✅ 安全但低效
```

#### 场景 3: API 请求风暴

```
1. 短时间内收到 5 次 task.completed 事件（重复推送）
2. 每次都触发 fetch(`/tasks/${id}`)
3. 产生 5 次相同的 HTTP 请求

当前实现：没有防抖/去重 ❌
```

---

## 🎯 改进方案

### 方案 1: 基于版本的幂等性检查（推荐）

#### 实现原理

在 Store 中维护已处理事件的版本号：

```typescript
// TaskStore 增强
const taskEventVersions = ref<Map<string, number>>(new Map())

async function handleTaskCompletedEvent(event: DomainEvent) {
  const taskId = event.payload.task_id
  const eventVersion = event.aggregate_version

  // 1. 检查是否已处理过更新版本的事件
  const lastVersion = taskEventVersions.value.get(taskId)
  if (lastVersion !== undefined && eventVersion <= lastVersion) {
    console.log(`[TaskStore] Ignoring stale event for task ${taskId}`)
    return
  }

  // 2. 处理事件
  try {
    const response = await fetch(`${apiBaseUrl}/tasks/${taskId}`)
    const result = await response.json()
    addOrUpdateTask(result.data.card)

    // 3. 更新版本号
    taskEventVersions.value.set(taskId, eventVersion)
  } catch (e) {
    console.error('[TaskStore] Failed to handle event:', e)
  }
}
```

**优点**:

- ✅ 防止旧事件覆盖新状态
- ✅ 自动忽略重复事件
- ✅ 无需额外存储

**缺点**:

- ⚠️ 内存中的版本号在刷新后会丢失（可接受）

### 方案 2: 基于事件 ID 的去重

```typescript
// 全局已处理事件缓存（可选：持久化到 localStorage）
const processedEventIds = ref<Set<string>>(new Set())
const MAX_CACHE_SIZE = 1000

async function handleTaskDeletedEvent(event: DomainEvent) {
  // 1. 检查是否已处理
  if (processedEventIds.value.has(event.event_id)) {
    console.log(`[TaskStore] Event ${event.event_id} already processed`)
    return
  }

  // 2. 处理事件
  const taskId = event.payload.task_id
  removeTask(taskId)

  // 3. 记录已处理事件
  processedEventIds.value.add(event.event_id)

  // 4. 清理旧缓存（FIFO）
  if (processedEventIds.value.size > MAX_CACHE_SIZE) {
    const firstId = processedEventIds.value.values().next().value
    processedEventIds.value.delete(firstId)
  }
}
```

**优点**:

- ✅ 精确去重（基于唯一 event_id）
- ✅ 可持久化到 localStorage

**缺点**:

- ⚠️ 需要额外存储
- ⚠️ 需要缓存淘汰策略

### 方案 3: 请求防抖（针对 API 请求）

```typescript
// 防抖 map
const pendingRefreshes = new Map<string, Promise<void>>()

async function handleTaskCompletedEvent(event: DomainEvent) {
  const taskId = event.payload.task_id

  // 1. 检查是否已有进行中的请求
  if (pendingRefreshes.has(taskId)) {
    console.log(`[TaskStore] Refresh for ${taskId} already pending`)
    return pendingRefreshes.get(taskId)
  }

  // 2. 创建新的请求
  const refreshPromise = (async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/tasks/${taskId}`)
      const result = await response.json()
      addOrUpdateTask(result.data.card)
    } finally {
      pendingRefreshes.delete(taskId)
    }
  })()

  pendingRefreshes.set(taskId, refreshPromise)
  return refreshPromise
}
```

**优点**:

- ✅ 防止并发重复请求
- ✅ 节省网络带宽

**缺点**:

- ⚠️ 稍微增加代码复杂度

### 方案 4: 综合方案（最健壮）

```typescript
// 综合使用版本检查 + 事件去重 + 请求防抖

const taskEventVersions = ref<Map<string, number>>(new Map())
const processedEventIds = ref<Set<string>>(new Set())
const pendingRefreshes = new Map<string, Promise<void>>()

async function handleTaskCompletedEvent(event: DomainEvent) {
  // 第一道防线：事件ID去重
  if (processedEventIds.value.has(event.event_id)) {
    return
  }

  const taskId = event.payload.task_id
  const eventVersion = event.aggregate_version

  // 第二道防线：版本检查
  const lastVersion = taskEventVersions.value.get(taskId)
  if (lastVersion !== undefined && eventVersion <= lastVersion) {
    processedEventIds.value.add(event.event_id)
    return
  }

  // 第三道防线：请求防抖
  if (pendingRefreshes.has(taskId)) {
    return pendingRefreshes.get(taskId)
  }

  // 处理事件
  const refreshPromise = (async () => {
    try {
      const response = await fetch(`${apiBaseUrl}/tasks/${taskId}`)
      const result = await response.json()
      addOrUpdateTask(result.data.card)

      taskEventVersions.value.set(taskId, eventVersion)
      processedEventIds.value.add(event.event_id)
    } finally {
      pendingRefreshes.delete(taskId)
    }
  })()

  pendingRefreshes.set(taskId, refreshPromise)
  return refreshPromise
}
```

---

## 🚦 实施建议

### 阶段 1: 最小改进（立即实施）

**针对删除操作**（天然幂等，无需改进）:

```typescript
// ✅ 当前实现已足够
function removeTask(id: string) {
  tasks.value.delete(id)
}
```

**针对API请求**（添加防抖）:

```typescript
// ✅ 防止短时间内重复请求
const pendingRefreshes = new Map<string, Promise<void>>()
```

### 阶段 2: 版本控制（可选）

如果观察到事件乱序问题，再引入版本检查：

```typescript
const taskEventVersions = ref<Map<string, number>>(new Map())
```

### 阶段 3: 事件ID去重（可选）

如果需要更严格的去重保证：

```typescript
const processedEventIds = ref<Set<string>>(new Set())
// 可选：持久化到 localStorage
```

---

## 📝 结论

**当前实现评估**:

- ✅ 删除操作：天然幂等，无需改进
- ⚠️ API请求：可能产生重复请求，建议添加防抖
- ⚠️ 版本控制：暂无，但目前未发现乱序问题

**建议优先级**:

1. **高优先级**: 添加 API 请求防抖（方案 3）
2. **中优先级**: 添加版本检查（方案 1），如果观察到乱序
3. **低优先级**: 事件ID去重（方案 2），如果需要更严格保证

**权衡**:

- 简单场景：当前实现已足够（Map.delete 天然幂等）
- 高并发/不稳定网络：建议实施方案 3 或 4

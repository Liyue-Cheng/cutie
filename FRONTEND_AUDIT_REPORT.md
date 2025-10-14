# 前端代码全面审计报告

**项目**: Cutie Task Management
**审计日期**: 2025-10-12
**审计范围**: `src/` 目录下所有 Vue 3 + TypeScript 代码
**审计者**: Claude Code

---

## 📊 执行摘要

| 指标                        | 数量                       |
| --------------------------- | -------------------------- |
| **分析文件数**              | 107 个（TypeScript + Vue） |
| **关键问题 (CRITICAL)**     | 8 个 🔴                    |
| **高优先级 (HIGH)**         | 24 个 🟠                   |
| **中低优先级 (MEDIUM/LOW)** | 31 个 🟡                   |
| **总问题数**                | 63 个                      |

### 总体评估

代码库显示出 V2.0 重构的努力痕迹，架构基础良好，但存在**严重的类型安全、API 一致性和内存管理问题**。建议在继续开发新功能前，先花 3-4 周解决所有关键问题和部分高优先级问题。

**核心优势**：

- ✅ 模块化 Pinia store 架构
- ✅ 关注点分离设计
- ✅ 完善的 Logger 系统
- ✅ 复杂的跨视图拖放系统
- ✅ 基于 SSE 的实时更新

**核心问题**：

- ❌ 类型安全缺失（30+ `any` 类型）
- ❌ API 客户端使用混乱
- ❌ 错误处理模式不统一
- ❌ 存在内存泄漏风险
- ❌ 完全缺少测试

---

## 🔴 关键问题（CRITICAL - 必须立即修复）

### 1. API 客户端使用严重不一致

**严重程度**: 🔴 CRITICAL
**受影响文件**:

- `src/stores/area.ts` (lines 98, 121, 151, 184)
- `src/stores/timeblock.ts` (lines 258, 296, 347, 398, 445)
- `src/composables/useRecurrenceOperations.ts` (line 161)
- `src/composables/calendar/useCalendarDrag.ts` (line 357)

**问题描述**:

Task store 正确使用了统一的 `apiGet/apiPost/apiPatch/apiDelete` 辅助函数，但 TimeBlock 和 Area store 仍在使用原始的 `fetch()` 调用。

**代码示例**:

```typescript
// ❌ 错误模式（area.ts:98）
const response = await fetch(`${apiBaseUrl}/areas`)
if (!response.ok) {
  throw new Error(`HTTP ${response.status}`)
}
const result = await response.json()

// ✅ 正确模式（应该这样写）
import { apiGet } from '@/stores/shared'
const areaList: Area[] = await apiGet('/areas')
```

**影响**:

- 错误处理不一致
- 部分请求缺少 correlation ID 支持
- 无法统一日志记录
- 难以维护和测试
- 无法集中管理请求拦截器

**修复方案**:

1. **立即修复** - 将所有 `fetch()` 调用替换为 `apiGet/apiPost/apiPatch/apiDelete`
2. **添加 ESLint 规则** - 禁止直接使用 `fetch()`
3. **代码审查** - 确保新代码使用统一 API 客户端

**修复检查清单**:

- [ ] `src/stores/area.ts` - 4 处 fetch 调用
- [ ] `src/stores/timeblock.ts` - 5 处 fetch 调用
- [ ] `src/composables/useRecurrenceOperations.ts` - 1 处
- [ ] `src/composables/calendar/useCalendarDrag.ts` - 1 处
- [ ] 添加 ESLint 规则: `no-restricted-globals` 禁用 `fetch`
- [ ] 更新 `CLAUDE.md` 强调必须使用统一 API 客户端

---

### 2. 类型安全灾难 - 30+ 处 `any` 类型

**严重程度**: 🔴 CRITICAL
**受影响文件**:

- `src/stores/task/event-handlers.ts` (8 个函数)
- `src/stores/timeblock.ts` (4 个函数)
- `src/stores/trash/event-handlers.ts` (3 个函数)
- `src/stores/template/event-handlers.ts` (2 个函数)
- `src/stores/schedule/event-handlers.ts` (若干函数)

**问题描述**:

所有 SSE 事件处理器都使用 `event: any` 参数类型，完全失去了 TypeScript 的类型保护。

**代码示例**:

```typescript
// ❌ 当前代码（task/event-handlers.ts:73）
async function handleTaskCompletedEvent(event: any) {
  const task = event.payload.task // ⚠️ 无类型检查
  const sideEffects = event.payload.side_effects // ⚠️ 可能 undefined
  // ... 200+ 行代码都没有类型保护
}
```

**影响**:

- 运行时错误风险极高
- IDE 无法提供智能提示
- 重构时容易遗漏
- 新成员无法理解事件结构
- 后端修改 event payload 结构时前端无感知

**修复方案**:

**第 1 步**: 创建事件类型定义文件

```typescript
// src/types/events.ts

import type { TaskCard, TaskDetail, TimeBlockView } from './dtos'

// 基础事件接口
export interface DomainEvent<T = unknown> {
  event_id: string
  event_type: string
  aggregate_type: string
  aggregate_id: string
  payload: T
  occurred_at: string
}

// Task 相关事件载荷
export interface TaskCreatedPayload {
  task: TaskCard
}

export interface TaskUpdatedPayload {
  task: TaskCard
}

export interface TaskCompletedPayload {
  task: TaskCard
  side_effects?: {
    deleted_time_blocks?: TimeBlockView[]
    truncated_time_blocks?: TimeBlockView[]
    completed_subtasks?: TaskCard[]
  }
}

export interface TaskReopenedPayload {
  task: TaskCard
  side_effects?: {
    schedule_outcome_reset?: boolean
  }
}

export interface TaskDeletedPayload {
  task_id: string
}

export interface TaskMovedToTrashPayload {
  task: TaskCard
}

// 类型化事件
export type TaskCreatedEvent = DomainEvent<TaskCreatedPayload>
export type TaskUpdatedEvent = DomainEvent<TaskUpdatedPayload>
export type TaskCompletedEvent = DomainEvent<TaskCompletedPayload>
export type TaskReopenedEvent = DomainEvent<TaskReopenedPayload>
export type TaskDeletedEvent = DomainEvent<TaskDeletedPayload>
export type TaskMovedToTrashEvent = DomainEvent<TaskMovedToTrashPayload>

// TimeBlock 相关事件
export interface TimeBlockCreatedPayload {
  time_block: TimeBlockView
}

export interface TimeBlockUpdatedPayload {
  time_block: TimeBlockView
}

export interface TimeBlockDeletedPayload {
  time_block_id: string
}

export type TimeBlockCreatedEvent = DomainEvent<TimeBlockCreatedPayload>
export type TimeBlockUpdatedEvent = DomainEvent<TimeBlockUpdatedPayload>
export type TimeBlockDeletedEvent = DomainEvent<TimeBlockDeletedPayload>

// ... 其他事件类型
```

**第 2 步**: 更新事件处理器

```typescript
// src/stores/task/event-handlers.ts

import type {
  TaskCreatedEvent,
  TaskUpdatedEvent,
  TaskCompletedEvent,
  TaskReopenedEvent,
  TaskDeletedEvent,
} from '@/types/events'

// ✅ 修复后 - 完全类型安全
async function handleTaskCompletedEvent(event: TaskCompletedEvent) {
  const { task, side_effects } = event.payload // ✅ 类型自动推导

  // ✅ IDE 提供智能提示
  if (side_effects?.deleted_time_blocks) {
    for (const block of side_effects.deleted_time_blocks) {
      timeBlockStore.removeTimeBlock(block.id) // ✅ 类型安全
    }
  }

  updateMapItem(tasks, task.id, task)
}
```

**第 3 步**: 更新事件注册

```typescript
// src/composables/useApiConfig.ts

import type {
  TaskCreatedEvent,
  TaskCompletedEvent,
  // ...
} from '@/types/events'

function setupTaskEventHandlers(eventService: EventService) {
  eventService.on<TaskCreatedEvent>('task.created', async (event) => {
    await taskStore.handleTaskCreatedEvent(event) // ✅ 类型检查
  })

  eventService.on<TaskCompletedEvent>('task.completed', async (event) => {
    await taskStore.handleTaskCompletedEvent(event) // ✅ 类型检查
  })
}
```

**修复检查清单**:

- [ ] 创建 `src/types/events.ts` 并定义所有事件类型
- [ ] 更新 `src/stores/task/event-handlers.ts` (8 个函数)
- [ ] 更新 `src/stores/timeblock.ts` 事件处理器 (4 个函数)
- [ ] 更新 `src/stores/trash/event-handlers.ts` (3 个函数)
- [ ] 更新 `src/stores/template/event-handlers.ts` (2 个函数)
- [ ] 更新 `src/stores/schedule/event-handlers.ts`
- [ ] 更新事件注册代码（`useApiConfig.ts`）
- [ ] 添加 ESLint 规则禁止 `any` 类型
- [ ] 运行全量测试确保无回归

**预估工作量**: 6-8 小时

---

### 3. 状态变更模式严重不一致

**严重程度**: 🔴 CRITICAL
**受影响文件**:

- `src/stores/area.ts` (lines 74, 82, 88)
- `src/stores/timeblock.ts` (lines 182-186, 192-195, 201-204)
- `src/stores/template/core.ts` (lines 22, 28, 34)
- `src/stores/recurrence/core.ts` (lines 26, 32, 39)

**问题描述**:

Task store 正确使用共享的 `updateMapItem/updateMapItems/removeMapItem` 工具函数，但其他 store 手动创建新 Map 对象。这导致：

1. 代码重复
2. 可能的 Vue 响应式 bug
3. 维护困难
4. 新成员困惑

**代码示例**:

```typescript
// ✅ 正确模式（task/core.ts:191）
import { updateMapItem } from '@/stores/shared/map-helpers'
updateMapItem(tasks, task.id, task)

// ❌ 错误模式 1（area.ts:74）
const newMap = new Map(areas.value)
newMap.set(area.id, area)
areas.value = newMap

// ❌ 错误模式 2（timeblock.ts:182-186）
const newMap = new Map(timeBlocks.value)
for (const block of blocks) {
  newMap.set(block.id, block)
}
timeBlocks.value = newMap

// ❌ 错误模式 3（template/core.ts:22）
templates.value = new Map([...templates.value, [template.id, template]])
```

**影响**:

- Vue 响应式可能失效（取决于具体场景）
- 代码审查困难
- 性能不一致（创建新 Map vs 原地修改）
- 维护成本高

**修复方案**:

**第 1 步**: 确保共享工具函数完整

```typescript
// src/stores/shared/map-helpers.ts（确认存在这些函数）

import type { Ref } from 'vue'

/**
 * 更新 Map 中的单个项目（响应式安全）
 */
export function updateMapItem<K, V>(mapRef: Ref<Map<K, V>>, key: K, value: V): void {
  const newMap = new Map(mapRef.value)
  newMap.set(key, value)
  mapRef.value = newMap
}

/**
 * 批量更新 Map 中的多个项目
 */
export function updateMapItems<K, V>(
  mapRef: Ref<Map<K, V>>,
  items: V[],
  getKey: (item: V) => K
): void {
  const newMap = new Map(mapRef.value)
  for (const item of items) {
    newMap.set(getKey(item), item)
  }
  mapRef.value = newMap
}

/**
 * 从 Map 中删除项目
 */
export function removeMapItem<K, V>(mapRef: Ref<Map<K, V>>, key: K): void {
  const newMap = new Map(mapRef.value)
  newMap.delete(key)
  mapRef.value = newMap
}

/**
 * 批量删除 Map 中的多个项目
 */
export function removeMapItems<K, V>(mapRef: Ref<Map<K, V>>, keys: K[]): void {
  const newMap = new Map(mapRef.value)
  for (const key of keys) {
    newMap.delete(key)
  }
  mapRef.value = newMap
}

/**
 * 清空 Map
 */
export function clearMap<K, V>(mapRef: Ref<Map<K, V>>): void {
  mapRef.value = new Map()
}
```

**第 2 步**: 重构所有 store

```typescript
// src/stores/area.ts

import { updateMapItem, updateMapItems, removeMapItem } from '@/stores/shared/map-helpers'

// ✅ 修复 line 74
function addArea(area: Area) {
  updateMapItem(areas, area.id, area)
}

// ✅ 修复 line 82
function updateArea(area: Area) {
  updateMapItem(areas, area.id, area)
}

// ✅ 修复 line 88
function deleteArea(areaId: string) {
  removeMapItem(areas, areaId)
}
```

```typescript
// src/stores/timeblock.ts

// ✅ 修复 lines 182-186
function addOrUpdateTimeBlocks(blocks: TimeBlockView[]) {
  updateMapItems(timeBlocks, blocks, (block) => block.id)
}

// ✅ 修复 lines 192-195
function removeTimeBlocks(blockIds: string[]) {
  removeMapItems(timeBlocks, blockIds)
}
```

**修复检查清单**:

- [ ] 确认 `src/stores/shared/map-helpers.ts` 完整
- [ ] 重构 `src/stores/area.ts` (3 处)
- [ ] 重构 `src/stores/timeblock.ts` (6 处)
- [ ] 重构 `src/stores/template/core.ts` (3 处)
- [ ] 重构 `src/stores/recurrence/core.ts` (3 处)
- [ ] 搜索整个代码库确保无遗漏: `grep -rn "new Map(" src/stores/`
- [ ] 添加 ESLint 规则（可选，防止直接操作 Map）
- [ ] 更新 `CLAUDE.md` 添加 Map 操作规范

**预估工作量**: 3-4 小时

全部修了

---

### 4. SSE 重连逻辑存在严重缺陷

**严重程度**: 🔴 CRITICAL
**受影响文件**:

- `src/services/events.ts` (lines 108-127)

**问题描述**:

EventSource 重连逻辑使用指数退避，但没有最大延迟上限，可能导致等待时间增长到数小时。同时缺少：

- UI 连接状态指示
- 手动重连功能
- 重连成功通知
- 连接健康检查

**代码示例**:

```typescript
// ❌ 当前代码（events.ts:108-127）
this.eventSource.onerror = (error) => {
  logger.error(TAG, 'EventSource error', error)

  if (!this.isManualClose && this.reconnectAttempts < this.maxReconnectAttempts) {
    this.reconnectAttempts++

    // 💀 问题：delay 无上限！
    // 第 10 次重连：1000ms * 2^9 = 512 秒 = 8.5 分钟
    // 第 20 次重连：1000ms * 2^19 = 524,288 秒 = 145 小时！
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)

    logger.info(TAG, `Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`)

    setTimeout(() => {
      this.connect()
    }, delay)
  } else {
    logger.error(TAG, 'Max reconnection attempts reached or manual close')
  }
}
```

**影响**:

- 网络闪断后应用可能 8+ 分钟才恢复
- 用户不知道连接断开
- 用户无法手动触发重连
- 长时间运行后用户体验极差

**修复方案**:

**第 1 步**: 添加连接状态枚举

```typescript
// src/types/connection-status.ts

export enum ConnectionStatus {
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  DISCONNECTED = 'disconnected',
  RECONNECTING = 'reconnecting',
  FAILED = 'failed',
}
```

**第 2 步**: 重构 EventService

```typescript
// src/services/events.ts

import { ref, type Ref } from 'vue'
import type { ConnectionStatus } from '@/types/connection-status'

const MAX_RECONNECT_DELAY = 30000 // 最大 30 秒
const INITIAL_RECONNECT_DELAY = 1000 // 初始 1 秒
const MAX_RECONNECT_ATTEMPTS = 10 // 最多尝试 10 次
const HEALTH_CHECK_INTERVAL = 30000 // 每 30 秒心跳检查

export class EventService {
  private eventSource: EventSource | null = null
  private reconnectAttempts = 0
  private reconnectDelay = INITIAL_RECONNECT_DELAY
  private maxReconnectAttempts = MAX_RECONNECT_ATTEMPTS
  private isManualClose = false
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private healthCheckTimer: ReturnType<typeof setInterval> | null = null
  private lastEventTime = 0

  // ✅ 新增：暴露连接状态
  public connectionStatus: Ref<ConnectionStatus> = ref(ConnectionStatus.DISCONNECTED)

  async connect(): Promise<void> {
    // 清理旧连接
    this.cleanup()

    this.connectionStatus.value = ConnectionStatus.CONNECTING
    this.isManualClose = false

    try {
      const port = await this.getPort()
      const url = `http://localhost:${port}/api/events/stream`

      this.eventSource = new EventSource(url)

      this.eventSource.onopen = () => {
        logger.info(TAG, 'EventSource connected')
        this.connectionStatus.value = ConnectionStatus.CONNECTED
        this.reconnectAttempts = 0 // ✅ 重置重连计数
        this.reconnectDelay = INITIAL_RECONNECT_DELAY // ✅ 重置延迟
        this.lastEventTime = Date.now()

        // ✅ 启动健康检查
        this.startHealthCheck()
      }

      this.eventSource.onerror = (error) => {
        logger.error(TAG, 'EventSource error', error)

        // ✅ 更新状态
        if (this.connectionStatus.value === ConnectionStatus.CONNECTED) {
          this.connectionStatus.value = ConnectionStatus.RECONNECTING
        }

        if (!this.isManualClose) {
          this.scheduleReconnect()
        } else {
          this.connectionStatus.value = ConnectionStatus.DISCONNECTED
        }
      }

      this.eventSource.onmessage = (event) => {
        this.lastEventTime = Date.now() // ✅ 更新心跳时间
        // ... 处理消息
      }
    } catch (error) {
      logger.error(TAG, 'Failed to connect', error)
      this.connectionStatus.value = ConnectionStatus.FAILED
      this.scheduleReconnect()
    }
  }

  // ✅ 新增：重连调度（带最大延迟限制）
  private scheduleReconnect(): void {
    // 清理旧的重连计时器
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      logger.error(TAG, 'Max reconnection attempts reached')
      this.connectionStatus.value = ConnectionStatus.FAILED
      return
    }

    this.reconnectAttempts++

    // ✅ 计算延迟，带上限
    const calculatedDelay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)
    const delay = Math.min(calculatedDelay, MAX_RECONNECT_DELAY)

    logger.info(
      TAG,
      `Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`
    )

    this.connectionStatus.value = ConnectionStatus.RECONNECTING

    this.reconnectTimer = setTimeout(() => {
      this.connect()
    }, delay)
  }

  // ✅ 新增：手动重连
  public async reconnect(): Promise<void> {
    logger.info(TAG, 'Manual reconnect requested')
    this.reconnectAttempts = 0 // 重置计数
    this.reconnectDelay = INITIAL_RECONNECT_DELAY
    await this.connect()
  }

  // ✅ 新增：健康检查（防止僵尸连接）
  private startHealthCheck(): void {
    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer)
    }

    this.healthCheckTimer = setInterval(() => {
      const timeSinceLastEvent = Date.now() - this.lastEventTime

      // 如果 60 秒没收到任何事件，认为连接可能已断开
      if (
        timeSinceLastEvent > 60000 &&
        this.connectionStatus.value === ConnectionStatus.CONNECTED
      ) {
        logger.warn(TAG, `No events received for ${timeSinceLastEvent}ms, reconnecting...`)
        this.reconnect()
      }
    }, HEALTH_CHECK_INTERVAL)
  }

  // ✅ 改进：清理函数
  private cleanup(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.healthCheckTimer) {
      clearInterval(this.healthCheckTimer)
      this.healthCheckTimer = null
    }

    if (this.eventSource) {
      this.eventSource.close()
      this.eventSource = null
    }
  }

  public disconnect(): void {
    logger.info(TAG, 'Manual disconnect')
    this.isManualClose = true
    this.connectionStatus.value = ConnectionStatus.DISCONNECTED
    this.cleanup()
  }
}
```

**第 3 步**: 添加 UI 连接状态指示器

```vue
<!-- src/components/parts/ConnectionStatusIndicator.vue -->
<template>
  <div class="connection-status" :class="statusClass">
    <div class="status-dot" />
    <span class="status-text">{{ statusText }}</span>
    <button v-if="canReconnect" @click="handleReconnect" class="reconnect-button">重新连接</button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useEventService } from '@/composables/useEventService'
import { ConnectionStatus } from '@/types/connection-status'

const eventService = useEventService()

const statusClass = computed(() => {
  switch (eventService.connectionStatus.value) {
    case ConnectionStatus.CONNECTED:
      return 'status-connected'
    case ConnectionStatus.CONNECTING:
    case ConnectionStatus.RECONNECTING:
      return 'status-connecting'
    case ConnectionStatus.FAILED:
    case ConnectionStatus.DISCONNECTED:
      return 'status-disconnected'
    default:
      return ''
  }
})

const statusText = computed(() => {
  switch (eventService.connectionStatus.value) {
    case ConnectionStatus.CONNECTED:
      return '已连接'
    case ConnectionStatus.CONNECTING:
      return '连接中...'
    case ConnectionStatus.RECONNECTING:
      return '重新连接中...'
    case ConnectionStatus.FAILED:
      return '连接失败'
    case ConnectionStatus.DISCONNECTED:
      return '未连接'
    default:
      return '未知状态'
  }
})

const canReconnect = computed(() => {
  return [ConnectionStatus.FAILED, ConnectionStatus.DISCONNECTED].includes(
    eventService.connectionStatus.value
  )
})

function handleReconnect() {
  eventService.reconnect()
}
</script>

<style scoped>
.connection-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 12px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-connected .status-dot {
  background: #52c41a;
}

.status-connecting .status-dot {
  background: #faad14;
  animation: pulse 1.5s ease-in-out infinite;
}

.status-disconnected .status-dot {
  background: #ff4d4f;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.reconnect-button {
  padding: 2px 8px;
  font-size: 11px;
  cursor: pointer;
}
</style>
```

**修复检查清单**:

- [ ] 创建 `src/types/connection-status.ts`
- [ ] 重构 `src/services/events.ts`
  - [ ] 添加 `connectionStatus` ref
  - [ ] 实现带上限的指数退避
  - [ ] 添加 `reconnect()` 方法
  - [ ] 添加健康检查机制
  - [ ] 改进 `cleanup()` 方法
- [ ] 创建 `src/components/parts/ConnectionStatusIndicator.vue`
- [ ] 在主布局中添加连接状态指示器
- [ ] 添加单元测试（重连逻辑）
- [ ] 手动测试：断网、恢复网络、长时间运行

**预估工作量**: 4-6 小时

---

全部修了

### 5. Correlation Tracker 存在内存泄漏

**严重程度**: 🔴 CRITICAL
**受影响文件**:

- `src/stores/shared/correlation-tracker.ts` (lines 173-181, 147)

**问题描述**:

性能计时器在 10 秒后通过 `setTimeout` 清理，但如果：

1. 操作耗时超过 10 秒
2. 操作失败导致 `finishTracking` 未调用
3. setTimeout 回调因某些原因未执行

则 `performanceTimers` Map 会无限增长。

**代码示例**:

```typescript
// ❌ 当前代码（correlation-tracker.ts:173-181）
function finishTracking(correlationId: string, delayMs = 10000): void {
  pendingCorrelations.value.delete(correlationId)

  // 💀 问题 1：如果这个 setTimeout 从未执行？
  // 💀 问题 2：如果操作耗时 > 10 秒？
  // 💀 问题 3：如果页面在这期间导航？
  setTimeout(() => {
    performanceTimers.value.delete(correlationId)
  }, delayMs)
}

// ❌ 当前代码（correlation-tracker.ts:147）
function startTracking(correlationId: string, operation: string): void {
  performanceTimers.value.set(correlationId, {
    operation,
    startTime: performance.now(),
  })
  // 💀 没有任何清理机制
}
```

**影响**:

- 长时间运行的应用会累积大量计时器
- 内存占用持续增长
- 性能逐渐下降
- 潜在的浏览器崩溃

**修复方案**:

```typescript
// src/stores/shared/correlation-tracker.ts

import { ref, type Ref } from 'vue'
import { logger } from '@/utils/logger'

const TAG = 'CorrelationTracker'

// ✅ 添加最大年龄常量
const MAX_TIMER_AGE_MS = 60000 // 1 分钟
const CLEANUP_INTERVAL_MS = 30000 // 每 30 秒清理一次

interface PerformanceTimer {
  operation: string
  startTime: number
  createdAt: number // ✅ 新增：创建时间戳
}

interface PendingCorrelation {
  operation: string
  params?: unknown
}

const performanceTimers: Ref<Map<string, PerformanceTimer>> = ref(new Map())
const pendingCorrelations: Ref<Map<string, PendingCorrelation>> = ref(new Map())

// ✅ 新增：清理计时器引用
let cleanupInterval: ReturnType<typeof setInterval> | null = null

// ✅ 新增：启动定期清理
export function initializeCorrelationTracker(): void {
  if (cleanupInterval) {
    clearInterval(cleanupInterval)
  }

  cleanupInterval = setInterval(() => {
    cleanupStaleTimers()
  }, CLEANUP_INTERVAL_MS)

  logger.debug(TAG, 'Correlation tracker initialized with periodic cleanup')
}

// ✅ 新增：清理过期计时器
function cleanupStaleTimers(): void {
  const now = Date.now()
  const staleIds: string[] = []

  for (const [id, timer] of performanceTimers.value) {
    const age = now - timer.createdAt

    if (age > MAX_TIMER_AGE_MS) {
      staleIds.push(id)
      logger.warn(TAG, `Cleaning up stale timer: ${timer.operation} (age: ${age}ms)`, {
        correlationId: id,
        operation: timer.operation,
        age,
      })
    }
  }

  if (staleIds.length > 0) {
    const newMap = new Map(performanceTimers.value)
    for (const id of staleIds) {
      newMap.delete(id)
    }
    performanceTimers.value = newMap

    logger.info(TAG, `Cleaned up ${staleIds.length} stale timers`)
  }
}

// ✅ 改进：添加创建时间戳
function startTracking(correlationId: string, operation: string): void {
  performanceTimers.value.set(correlationId, {
    operation,
    startTime: performance.now(),
    createdAt: Date.now(), // ✅ 记录创建时间
  })

  logger.debug(TAG, `Started tracking: ${operation}`, { correlationId })
}

// ✅ 改进：立即清理，不使用 setTimeout
function finishTracking(correlationId: string): void {
  const timer = performanceTimers.value.get(correlationId)

  if (timer) {
    const duration = performance.now() - timer.startTime
    logger.debug(TAG, `Finished tracking: ${timer.operation} (${duration.toFixed(2)}ms)`, {
      correlationId,
      duration,
    })

    // ✅ 立即删除，不延迟
    const newMap = new Map(performanceTimers.value)
    newMap.delete(correlationId)
    performanceTimers.value = newMap
  }

  // 清理 pending correlation
  pendingCorrelations.value.delete(correlationId)
}

// ✅ 新增：获取当前状态（用于调试）
export function getTrackerStats() {
  return {
    activeTimers: performanceTimers.value.size,
    pendingCorrelations: pendingCorrelations.value.size,
    timers: Array.from(performanceTimers.value.entries()).map(([id, timer]) => ({
      id,
      operation: timer.operation,
      age: Date.now() - timer.createdAt,
      duration: performance.now() - timer.startTime,
    })),
  }
}

// ✅ 新增：停止清理（用于测试或应用卸载）
export function shutdownCorrelationTracker(): void {
  if (cleanupInterval) {
    clearInterval(cleanupInterval)
    cleanupInterval = null
  }

  performanceTimers.value.clear()
  pendingCorrelations.value.clear()

  logger.debug(TAG, 'Correlation tracker shut down')
}

// 导出
export const correlationTracker = {
  startTracking,
  finishTracking,
  getTrackerStats,
}
```

**第 2 步**: 在应用启动时初始化

```typescript
// src/main.ts

import {
  initializeCorrelationTracker,
  shutdownCorrelationTracker,
} from '@/stores/shared/correlation-tracker'

// ✅ 在应用启动时初始化
initializeCorrelationTracker()

// ✅ 在应用卸载时清理（如果需要）
window.addEventListener('beforeunload', () => {
  shutdownCorrelationTracker()
})
```

**第 3 步**: 添加开发工具（可选）

```typescript
// 仅在开发环境暴露调试工具
if (import.meta.env.DEV) {
  ;(window as any).__correlationTracker__ = {
    getStats: getTrackerStats,
    cleanup: cleanupStaleTimers,
  }
}
```

**修复检查清单**:

- [ ] 重构 `src/stores/shared/correlation-tracker.ts`
  - [ ] 添加 `createdAt` 字段
  - [ ] 实现 `cleanupStaleTimers()`
  - [ ] 实现 `initializeCorrelationTracker()`
  - [ ] 移除 `finishTracking` 中的 setTimeout
  - [ ] 添加 `getTrackerStats()` 调试工具
- [ ] 在 `src/main.ts` 中初始化
- [ ] 添加单元测试验证清理逻辑
- [ ] 手动测试：长时间运行，检查内存占用
- [ ] 添加性能监控（可选）

**预估工作量**: 3-4 小时

---

### 6. View Store 刷新方法存在竞态条件

**严重程度**: 🔴 CRITICAL
**受影响文件**:

- `src/stores/view.ts` (lines 318-334)

**问题描述**:

`refreshAllMountedDailyViews()` 使用防抖，但 `isRefreshing` 标志在 setTimeout 回调内部设置，导致多次快速调用会创建多个排队的刷新操作。

**代码示例**:

```typescript
// ❌ 当前代码（view.ts:318-334）
async function refreshAllMountedDailyViews(): Promise<void> {
  if (refreshDebounceTimer) {
    clearTimeout(refreshDebounceTimer)
  }

  return new Promise<void>((resolve) => {
    refreshDebounceTimer = setTimeout(async () => {
      try {
        isRefreshing.value = true // 💀 太晚了！已经有多个 Promise 在队列中

        const dates = Array.from(mountedDailyViews.value)
        await performConcurrentRefresh(dates)
      } finally {
        isRefreshing.value = false
        refreshDebounceTimer = null
        resolve()
      }
    }, REFRESH_DEBOUNCE_DELAY)
  })
}
```

**竞态场景**:

```typescript
// 用户快速拖动任务
await refreshAllMountedDailyViews() // 创建 Promise 1
await refreshAllMountedDailyViews() // 取消 timer，创建 Promise 2
await refreshAllMountedDailyViews() // 取消 timer，创建 Promise 3

// 300ms 后，三个 Promise 的 callback 都会执行！
// 因为 isRefreshing 检查在 callback 内部
```

**影响**:

- 可能同时发起多个 API 请求
- 浪费带宽
- 数据不一致风险
- 性能问题

**修复方案**:

```typescript
// src/stores/view.ts

const REFRESH_DEBOUNCE_DELAY = 300
const isRefreshing = ref(false)
let refreshDebounceTimer: ReturnType<typeof setTimeout> | null = null
let pendingRefreshResolvers: Array<() => void> = [] // ✅ 新增：待解决的 Promise

async function refreshAllMountedDailyViews(): Promise<void> {
  // ✅ 第一步：如果正在刷新，等待当前刷新完成
  if (isRefreshing.value) {
    logger.debug(TAG, 'Refresh already in progress, waiting...')
    return new Promise<void>((resolve) => {
      pendingRefreshResolvers.push(resolve)
    })
  }

  // ✅ 第二步：清除之前的防抖计时器
  if (refreshDebounceTimer) {
    clearTimeout(refreshDebounceTimer)
    refreshDebounceTimer = null
  }

  // ✅ 第三步：创建新的防抖 Promise
  return new Promise<void>((resolve) => {
    // 将当前 resolver 加入队列
    pendingRefreshResolvers.push(resolve)

    refreshDebounceTimer = setTimeout(async () => {
      // ✅ 双重检查
      if (isRefreshing.value) {
        logger.warn(TAG, 'Race condition detected, skipping duplicate refresh')
        return
      }

      isRefreshing.value = true
      refreshDebounceTimer = null

      try {
        const dates = Array.from(mountedDailyViews.value)
        logger.info(TAG, `Refreshing ${dates.length} daily views`, { dates })

        await performConcurrentRefresh(dates)

        logger.info(TAG, 'All daily views refreshed successfully')
      } catch (error) {
        logger.error(TAG, 'Failed to refresh daily views', error)
        throw error
      } finally {
        isRefreshing.value = false

        // ✅ 解决所有等待的 Promise
        const resolvers = [...pendingRefreshResolvers]
        pendingRefreshResolvers = []

        for (const resolver of resolvers) {
          resolver()
        }
      }
    }, REFRESH_DEBOUNCE_DELAY)
  })
}

// ✅ 新增：取消正在进行的刷新（如果需要）
export function cancelRefresh(): void {
  if (refreshDebounceTimer) {
    clearTimeout(refreshDebounceTimer)
    refreshDebounceTimer = null
  }

  // 拒绝所有等待的 Promise
  pendingRefreshResolvers = []
  isRefreshing.value = false
}
```

**更好的替代方案（使用 VueUse）**:

```typescript
import { useDebounceFn } from '@vueuse/core'

// ✅ 使用 VueUse 的防抖函数（自动处理竞态）
const debouncedRefresh = useDebounceFn(async () => {
  if (isRefreshing.value) {
    return
  }

  isRefreshing.value = true

  try {
    const dates = Array.from(mountedDailyViews.value)
    await performConcurrentRefresh(dates)
  } finally {
    isRefreshing.value = false
  }
}, REFRESH_DEBOUNCE_DELAY)

async function refreshAllMountedDailyViews(): Promise<void> {
  await debouncedRefresh()
}
```

**修复检查清单**:

- [ ] 选择修复方案（手动实现 vs VueUse）
- [ ] 重构 `refreshAllMountedDailyViews()`
- [ ] 添加 `pendingRefreshResolvers` 管理
- [ ] 添加双重检查锁
- [ ] 添加单元测试（模拟快速连续调用）
- [ ] 手动测试：快速拖放任务，监控网络请求
- [ ] 添加开发环境日志

**预估工作量**: 2-3 小时

---

### 7. 注释掉的调试代码遍布全局

**严重程度**: 🔴 CRITICAL（可维护性）
**受影响文件**: 20+ 个文件，100+ 行注释代码

**主要问题文件**:

- `src/components/templates/InfiniteDailyKanban.vue` (20+ 行注释 console.log)
- `src/stores/task/core.ts` (lines 127-150)
- `src/components/parts/kanban/SimpleKanbanColumn.vue` (多处)
- `src/stores/view.ts` (多处)
- `src/composables/drag/*` (多处)

**问题示例**:

```typescript
// ❌ InfiniteDailyKanban.vue:50-70（大量注释日志）
function calculateVisibleLeftmostDate(): string | null {
  // logger.debug(TAG, 'calculateVisibleLeftmostDate called')

  const container = kanbanContainerRef.value
  if (!container) {
    // logger.debug(TAG, 'No container ref')
    return null
  }

  const scrollLeft = container.scrollLeft
  // logger.debug(TAG, 'scrollLeft:', scrollLeft)

  // logger.debug(TAG, 'visibleDate:', visibleDate, 'offsetLeft:', offsetLeft)
  // logger.debug(TAG, 'Calculated visible leftmost date:', visibleDate)

  return visibleDate
}
```

**影响**:

1. **代码膨胀** - 100+ 行无用代码
2. **审查困难** - PR diff 中难以分辨有效代码
3. **维护困惑** - 新成员不知道该保留还是删除
4. **信任问题** - 说明开发者不信任 logger 系统
5. **合并冲突** - 注释行增加冲突概率

**修复方案**:

**第 1 步**: 删除所有注释的 console.log 和 logger 调用

```bash
# 搜索所有注释的日志
grep -rn "// console\." src/
grep -rn "// logger\." src/
grep -rn "//.*(console|logger)" src/

# 手动审查并删除
```

**第 2 步**: 信任并改进 Logger 系统

```typescript
// 如果需要调试，使用 logger 而不是 console.log
import { logger } from '@/utils/logger'

const TAG = 'InfiniteDailyKanban'

function calculateVisibleLeftmostDate(): string | null {
  // ✅ 使用 logger.debug，可以通过配置开关
  logger.debug(TAG, 'calculateVisibleLeftmostDate called')

  const container = kanbanContainerRef.value
  if (!container) {
    logger.debug(TAG, 'No container ref')
    return null
  }

  const scrollLeft = container.scrollLeft
  logger.debug(TAG, 'Scroll position', { scrollLeft })

  // ... 业务逻辑

  logger.debug(TAG, 'Calculated visible date', { visibleDate })
  return visibleDate
}
```

**第 3 步**: 添加 ESLint 规则防止注释日志

```json
// .eslintrc.json
{
  "rules": {
    "no-console": "warn",
    "no-commented-out-code": "warn" // 需要插件
  }
}
```

**第 4 步**: 添加 VS Code 设置高亮 console

```json
// .vscode/settings.json
{
  "todohighlight.keywords": [
    {
      "text": "console.",
      "color": "#ff0000",
      "backgroundColor": "#ffff00",
      "overviewRulerColor": "red"
    }
  ]
}
```

**修复检查清单**:

- [ ] 搜索所有注释的日志调用
- [ ] 删除 `InfiniteDailyKanban.vue` 中的注释日志 (20+ 行)
- [ ] 删除 `SimpleKanbanColumn.vue` 中的注释日志
- [ ] 删除 `task/core.ts` 中的注释日志
- [ ] 删除 `view.ts` 中的注释日志
- [ ] 删除 `drag/*` composables 中的注释日志
- [ ] 全局搜索确保无遗漏: `grep -rn "// .*console\." src/`
- [ ] 添加 ESLint 规则
- [ ] 更新 `CLAUDE.md` 添加日志规范
- [ ] Code review 时检查此项

**预估工作量**: 1-2 小时

---

### 8. 废弃代码未移除或未添加警告

**严重程度**: 🔴 CRITICAL
**受影响文件**:

- `src/stores/task/core.ts` (lines 106-110)
- 其他 store 中可能存在的废弃 getter/action

**问题示例**:

```typescript
// ❌ task/core.ts:106-110
/**
 * @deprecated 使用 plannedTasks（只含未完成）
 */
const scheduledTasks = computed(() => {
  return allTasksArray.value.filter((task) => task.schedule_status === 'scheduled')
})

// 💀 问题：
// 1. 仍然被导出和使用
// 2. 没有运行时警告
// 3. 新成员可能误用
```

**影响**:

- 开发者可能使用错误的 API
- 技术债务累积
- 代码库混乱
- 重构困难

**修复方案**:

**选项 1**: 直接删除（推荐）

```typescript
// ✅ 直接删除废弃代码
// 删除 scheduledTasks getter
// 搜索所有引用并替换为 plannedTasks
```

**选项 2**: 添加运行时警告（如果需要兼容性过渡期）

```typescript
import { logger } from '@/utils/logger'

const TAG = 'TaskStore'

/**
 * @deprecated 使用 plannedTasks（只含未完成）
 * 此 getter 将在 v3.0 移除
 */
const scheduledTasks = computed(() => {
  // ✅ 添加运行时警告
  if (import.meta.env.DEV) {
    logger.warn(
      TAG,
      'scheduledTasks is DEPRECATED. Use plannedTasks instead. This will be removed in v3.0'
    )
    console.trace('Deprecated API usage trace:') // 显示调用栈
  }

  return allTasksArray.value.filter((task) => task.schedule_status === 'scheduled')
})
```

**选项 3**: 使用 TypeScript `@deprecated` 注解（编译时警告）

```typescript
/**
 * @deprecated Use `plannedTasks` instead - will be removed in v3.0
 * @see plannedTasks
 */
const scheduledTasks = computed(() => {
  return allTasksArray.value.filter((task) => task.schedule_status === 'scheduled')
})

// TypeScript 会在使用时显示删除线
// IDE 会显示警告
```

**修复步骤**:

1. **查找所有废弃代码**

```bash
grep -rn "@deprecated" src/
```

2. **评估每个废弃项**
   - 是否有使用？
   - 能否直接删除？
   - 是否需要过渡期？

3. **执行修复**
   - 直接删除：搜索引用并替换
   - 添加警告：运行时 + TypeScript 注解
   - 文档：在 CHANGELOG 中记录

**修复检查清单**:

- [ ] 搜索所有 `@deprecated` 标记
- [ ] 对于 `scheduledTasks`:
  - [ ] 搜索所有使用: `grep -rn "scheduledTasks" src/`
  - [ ] 如果无使用，直接删除
  - [ ] 如果有使用，替换为 `plannedTasks` 后删除
  - [ ] 或添加运行时警告
- [ ] 检查其他 store 是否有废弃代码
- [ ] 更新 CHANGELOG 记录破坏性变更（如果删除公共 API）
- [ ] 添加 ESLint 规则检测废弃 API 使用（可选）

**预估工作量**: 1-2 小时

---

## 🟠 高优先级问题（HIGH - 应尽快修复）

### 9. Loading 状态管理模式不一致

**严重程度**: 🟠 HIGH
**受影响文件**:

- `src/stores/task/*` - 使用 `createLoadingState()` + `withLoading()`
- `src/stores/area.ts` - 手动管理 `isLoading` ref
- `src/stores/timeblock.ts` - 手动管理 `isLoading` ref

**问题对比**:

```typescript
// ✅ Task store - 统一模式
const { isLoading, error, withLoading } = createLoadingState()

async function fetchAllTasks() {
  return withLoading(async () => {
    const tasks = await apiGet<TaskCard[]>('/tasks')
    // ... 处理数据
    return tasks
  }, 'fetch all tasks')
}

// ❌ Area store - 手动模式
const isLoading = ref(false)

async function fetchAllAreas() {
  isLoading.value = true
  try {
    const response = await fetch(`${apiBaseUrl}/areas`)
    const result = await response.json()
    // ... 处理数据
  } catch (error) {
    console.error('Failed to fetch areas:', error)
    throw error
  } finally {
    isLoading.value = false
  }
}
```

**修复**:

```typescript
// src/stores/area.ts

import { createLoadingState } from '@/stores/shared/loading-state'
import { apiGet, apiPost, apiPatch, apiDelete } from '@/stores/shared/api-client'

const { isLoading, error, withLoading } = createLoadingState()

// ✅ 统一使用 withLoading
async function fetchAllAreas() {
  return withLoading(async () => {
    const areas = await apiGet<Area[]>('/areas')

    const newMap = new Map<string, Area>()
    for (const area of areas) {
      newMap.set(area.id, area)
    }
    areas.value = newMap

    return areas
  }, 'fetch all areas')
}
```

**修复检查清单**:

- [ ] 重构 `area.ts` 使用 `createLoadingState()`
- [ ] 重构 `timeblock.ts` 使用 `createLoadingState()`
- [ ] 确保所有 store 使用统一模式
- [ ] 更新 `CLAUDE.md` 添加 loading 状态规范

---

### 10. 缺少客户端输入验证

**严重程度**: 🟠 HIGH
**受影响**: 所有 CRUD 操作

**问题示例**:

```typescript
// ❌ task/crud-operations.ts:34
async function createTask(payload: CreateTaskPayload): Promise<TaskCard | null> {
  // 💀 没有验证 payload.title 是否为空！
  // 💀 没有验证 estimated_duration 是否为负数！
  const newTask: TaskCard = await apiPost('/tasks', payload)
  return newTask
}
```

**修复方案**:

使用 Zod 进行运行时验证：

```typescript
// src/schemas/task-schemas.ts

import { z } from 'zod'

export const CreateTaskSchema = z.object({
  title: z.string().min(1, '任务标题不能为空').max(500, '任务标题不能超过 500 字符').trim(),

  glance_note: z.string().max(1000, '备注不能超过 1000 字符').optional().nullable(),

  estimated_duration: z
    .number()
    .int('时长必须是整数')
    .positive('时长必须大于 0')
    .max(1440, '时长不能超过 24 小时')
    .optional()
    .nullable(),

  area_id: z.string().uuid('无效的 Area ID').optional().nullable(),

  parent_id: z.string().uuid('无效的 Parent ID').optional().nullable(),
})

export type ValidatedCreateTaskPayload = z.infer<typeof CreateTaskSchema>
```

```typescript
// src/stores/task/crud-operations.ts

import { CreateTaskSchema } from '@/schemas/task-schemas'
import type { ValidatedCreateTaskPayload } from '@/schemas/task-schemas'

async function createTask(payload: CreateTaskPayload): Promise<TaskCard | null> {
  try {
    // ✅ 验证输入
    const validated = CreateTaskSchema.parse(payload)

    // ✅ 使用验证后的数据
    const newTask: TaskCard = await apiPost('/tasks', validated)

    updateMapItem(tasks, newTask.id, newTask)
    logger.info(TAG, 'Task created', { taskId: newTask.id })

    return newTask
  } catch (error) {
    if (error instanceof z.ZodError) {
      // 处理验证错误
      logger.error(TAG, 'Validation failed', error.errors)

      // 可以展示用户友好的错误消息
      const firstError = error.errors[0]
      throw new Error(firstError.message)
    }

    throw error
  }
}
```

**修复检查清单**:

- [ ] 安装 Zod: `pnpm add zod`
- [ ] 创建 `src/schemas/task-schemas.ts`
- [ ] 为所有 CRUD payload 添加 schema
- [ ] 在 store actions 中使用验证
- [ ] 添加用户友好的错误提示
- [ ] 考虑在组件层也使用相同 schema（表单验证）

---

### 11. 日期工具函数重复实现

**严重程度**: 🟠 HIGH
**受影响文件**:

- `src/components/templates/InfiniteDailyKanban.vue` (lines 50-63)
- `src/utils/dateUtils.ts`

**问题**:

```typescript
// ❌ InfiniteDailyKanban.vue:50-63（重复实现）
function formatDate(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function addDays(date: Date, days: number): Date {
  const result = new Date(date)
  result.setDate(result.getDate() + days)
  return result
}

// ✅ dateUtils.ts 已经有类似功能
export const toDateString = (date: Date | string): string => {
  /* ... */
}
```

**修复**:

```typescript
// ✅ InfiniteDailyKanban.vue
import { toDateString, addDays } from '@/infra/utils/dateUtils'

// 删除重复函数，直接使用 import 的版本
```

**修复检查清单**:

- [ ] 删除 `InfiniteDailyKanban.vue` 中的重复函数
- [ ] 确认 `dateUtils.ts` 有所需的所有功能
- [ ] 如果缺少，补充到 `dateUtils.ts`
- [ ] 搜索其他可能的重复: `grep -rn "function.*Date" src/components/`

---

### 12. 硬编码魔法数字

**严重程度**: 🟠 HIGH

**问题列表**:

```typescript
// ❌ 多处重复的魔法数字
estimated_duration: (60, // SimpleKanbanColumn.vue:103, 119
  (delayMs = 10000)) // correlation-tracker.ts:147

const maxAttempts = 100 // useApiConfig.ts:53

REFRESH_DEBOUNCE_DELAY = 300 // view.ts:311
```

**修复**:

```typescript
// src/constants/defaults.ts

// Task 相关
export const DEFAULT_TASK_DURATION_MINUTES = 60

// Correlation Tracker
export const CORRELATION_CLEANUP_DELAY_MS = 10000
export const CORRELATION_MAX_TIMER_AGE_MS = 60000

// API 配置
export const MAX_PORT_DISCOVERY_ATTEMPTS = 100
export const PORT_DISCOVERY_INTERVAL_MS = 100

// View 刷新
export const VIEW_REFRESH_DEBOUNCE_MS = 300

// SSE 重连
export const SSE_INITIAL_RECONNECT_DELAY_MS = 1000
export const SSE_MAX_RECONNECT_DELAY_MS = 30000
export const SSE_MAX_RECONNECT_ATTEMPTS = 10

// 时间块
export const TIMEBLOCK_MIN_DURATION_MINUTES = 1
export const TIMEBLOCK_MAX_DURATION_MINUTES = 1440 // 24小时
```

**使用**:

```typescript
// SimpleKanbanColumn.vue
import { DEFAULT_TASK_DURATION_MINUTES } from '@/constants/defaults'

const newTask = {
  title: inputText.value.trim(),
  estimated_duration: DEFAULT_TASK_DURATION_MINUTES,
}
```

**修复检查清单**:

- [ ] 创建 `src/constants/defaults.ts`
- [ ] 提取所有魔法数字
- [ ] 更新所有使用处
- [ ] 添加 ESLint 规则检测魔法数字（可选）

---

### 13-31. 其他高优先级问题（简略）

由于篇幅限制，以下问题简要列举：

**13. 组件中潜在 N+1 查询模式**

- 每个任务卡片单独查询 area
- 修复：父组件预计算 lookup map

**14. Composables 缺少清理**

- `useCrossViewDrag` 无 `onBeforeUnmount`
- 修复：添加清理逻辑

**15. 错误处理模式不一致**

- 有的返回 `null`，有的抛异常，有的返回 `boolean`
- 修复：统一为抛异常或 Result 类型

**16. TypeScript 严格检查未启用**

- 大量 `?.` 可选链
- 修复：启用 `strict: true`

**17. 未完成的 TODO 注释**

- 6 个未实现的 API 调用
- 修复：实现或删除

**18. Props 传递层级过深**

- ViewMetadata 传递 3 层
- 修复：使用 provide/inject

**19. 无请求去重**

- 快速切换导致重复请求
- 修复：实现请求缓存

**20. 组件命名不一致**

- 混合多种命名风格
- 修复：统一为 PascalCase + 类型后缀

**21. 缺少运行时 Props 验证**

- 仅 TypeScript 类型
- 修复：使用对象语法 + validator

**22. Window 对象污染**

- `(window as any).appLogger`
- 修复：使用 `app.config.globalProperties`

**23. 组件文件过大**

- 500+ 行组件
- 修复：拆分子组件

**24. 无加载骨架屏**

- 空白屏闪烁
- 修复：添加 skeleton loader

**25. InfiniteDailyKanban 内存泄漏**

- View 注册可能累积
- 修复：改进生命周期管理

**26. 缺少无障碍属性**

- 缺少 ARIA
- 修复：添加 `aria-*` 属性

**27. 未使用的导入**

- 注释的 import
- 修复：运行 ESLint 清理

**28. 事件命名不一致**

- 混用 kebab-case 和 camelCase
- 修复：统一 kebab-case

**29. 无节流/防抖**

- 滚动事件每像素触发
- 修复：使用 `useDebounceFn`

**30. 潜在 XSS 风险**

- 需检查 `v-html`
- 修复：使用 DOMPurify

**31. 无性能监控**

- 缺少 performance marks
- 修复：添加性能标记

---

## 🟡 中低优先级问题（MEDIUM/LOW）

### 32. Store 初始化顺序无保证

**问题**: Area store 在 `main.ts` 加载，但事件订阅在 `useApiConfig.ts` 初始化，可能出现竞态。

**修复**: 使用明确的初始化函数，保证顺序。

---

### 33. 大量注释代码块

**问题**: 除了 console.log，还有大段注释的业务逻辑。

**修复**: 删除或移到 Git 分支。

---

### 34. 函数命名不一致

**问题**: 混用 `handle*`, `on*`, `do*` 前缀。

**修复**: 统一规范（`handle*` 用于事件处理器）。

---

### 35. 缺少 JSDoc 注释

**问题**: Composables 和复杂函数缺少文档。

**修复**: 添加 JSDoc。

---

### 36. 硬编码中文文本

**问题**: 模板中大量中文文本，无 i18n。

**修复**: 使用 `vue-i18n`。

---

### 37-63. 其他中低优先级问题（列表）

37. 无单元测试
38. 无 E2E 测试
39. 文件组织不一致
40. 大量 barrel exports
41. 无 Git pre-commit hooks
42. 无 bundle size 监控
43. 重复 CSS 颜色定义
44. 混合缩进（2 和 4 空格）
45. 缺少 error boundaries
46. 无离线支持
47. 无 Service Worker
48. localStorage 使用未加密
49. 无 CSRF 保护
50. 混合 HTTP 状态码处理
51. 无 API 响应缓存
52. SSE fallback 未实现
53. 无请求重试逻辑
54. 缺少乐观更新
55. 无撤销/重做
56. 键盘快捷键不完整
57. 无焦点管理
58. 缺少打印样式
59. 无暗黑模式
60. Z-index 值不一致
61. 无响应式图片
62. 无路由懒加载
63. 生产环境 console.log 未剥离

---

## 🎯 快速胜利（Quick Wins）

以下是投入产出比最高的修复任务：

| #   | 任务                        | 时间 | 收益                 | 优先级     |
| --- | --------------------------- | ---- | -------------------- | ---------- |
| 1   | 删除所有注释 console.log    | 1h   | LOC -100+，可读性 ↑↑ | ⭐⭐⭐⭐⭐ |
| 2   | 统一 API 客户端使用         | 4h   | 一致性 ↑↑，维护性 ↑↑ | ⭐⭐⭐⭐⭐ |
| 3   | 提取魔法数字到常量          | 1h   | 可维护性 ↑           | ⭐⭐⭐⭐   |
| 4   | 启用 TypeScript strict 模式 | 10h  | 类型安全 ↑↑↑         | ⭐⭐⭐⭐⭐ |
| 5   | 为事件添加类型定义          | 6h   | 类型安全 ↑↑↑         | ⭐⭐⭐⭐⭐ |
| 6   | 统一状态变更模式            | 4h   | 一致性 ↑↑            | ⭐⭐⭐⭐   |
| 7   | 修复 SSE 重连逻辑           | 6h   | 可靠性 ↑↑↑           | ⭐⭐⭐⭐⭐ |
| 8   | 添加请求去重                | 2h   | 性能 ↑↑              | ⭐⭐⭐⭐   |
| 9   | 清理未使用导入              | 1h   | 包大小 ↓             | ⭐⭐⭐     |
| 10  | 添加 loading 状态           | 4h   | UX ↑↑                | ⭐⭐⭐⭐   |

**总计**: ~39 小时可解决最关键的 10 个问题

---

## 📅 重构路线图

### 第 1-2 周：基础设施（Foundation）

**目标**: 建立代码质量基准线

- [ ] **删除所有注释代码**（console.log、业务逻辑）
- [ ] **统一 API 客户端使用**（area.ts, timeblock.ts 等）
- [ ] **统一状态变更模式**（使用 map-helpers）
- [ ] **为所有 SSE 事件添加类型定义**
- [ ] **提取所有魔法数字到常量**
- [ ] **启用 TypeScript strict 模式**
- [ ] **修复所有 strict 模式错误**
- [ ] **添加 ESLint 规则**（no-console, no-any 等）
- [ ] **设置 Git pre-commit hooks**

**产出**:

- 类型安全的代码库
- 统一的编码规范
- 自动化质量检查

---

### 第 3-4 周：可靠性（Reliability）

**目标**: 消除已知 bug 和内存泄漏

- [ ] **修复 SSE 重连逻辑**（指数退避上限、状态管理）
- [ ] **修复 Correlation Tracker 内存泄漏**
- [ ] **修复 View Store 竞态条件**
- [ ] **修复 InfiniteDailyKanban 内存泄漏**
- [ ] **添加 Vue Error Boundaries**
- [ ] **实现请求去重**
- [ ] **添加请求重试逻辑**（除 SSE 外）
- [ ] **改进错误处理**（统一模式）
- [ ] **添加连接状态 UI 指示器**

**产出**:

- 稳定的应用运行时
- 用户可见的连接状态
- 更好的错误恢复

---

### 第 5-6 周：性能（Performance）

**目标**: 优化用户体验

- [ ] **实现请求缓存层**
- [ ] **添加加载骨架屏**
- [ ] **实现虚拟滚动**（长列表）
- [ ] **添加节流/防抖**（滚动、拖放事件）
- [ ] **优化组件重渲染**（useMemo, v-memo）
- [ ] **添加性能监控**（performance.mark/measure）
- [ ] **实现路由懒加载**
- [ ] **Bundle size 分析和优化**
- [ ] **添加 Web Vitals 监控**

**产出**:

- 更快的加载速度
- 更流畅的交互
- 性能指标可视化

---

### 第 7-8 周：开发体验（Developer Experience）

**目标**: 提升开发效率

- [ ] **为所有 composables 添加 JSDoc**
- [ ] **为复杂函数添加 JSDoc**
- [ ] **创建组件库文档**（Storybook 或 VitePress）
- [ ] **添加单元测试**（utils, composables）
- [ ] **添加 E2E 测试**（关键流程）
- [ ] **创建开发指南文档**
- [ ] **设置 VS Code 推荐扩展**
- [ ] **添加调试配置**
- [ ] **改进日志系统**（分级、过滤）

**产出**:

- 完善的文档
- 自动化测试覆盖
- 新成员快速上手

---

### 第 9-10 周：打磨（Polish）

**目标**: 提升产品质量

- [ ] **添加无障碍支持**（ARIA 属性、键盘导航）
- [ ] **实现完整 i18n**
- [ ] **添加键盘快捷键系统**
- [ ] **改进错误消息**（用户友好）
- [ ] **实现撤销/重做**
- [ ] **添加暗黑模式**（如需要）
- [ ] **添加打印样式**
- [ ] **实现离线支持**（Service Worker）
- [ ] **添加更新通知**

**产出**:

- 专业级产品体验
- 无障碍友好
- 国际化支持

---

## 🏗️ 架构建议

### 1. 建立统一 API 层

**当前问题**: API 调用分散在各个 store，模式不统一。

**建议方案**:

```typescript
// src/api/index.ts

import { apiGet, apiPost, apiPatch, apiDelete } from '@/stores/shared/api-client'
import type {
  TaskCard,
  TaskDetail,
  CreateTaskPayload,
  UpdateTaskPayload,
  Area,
  TimeBlockView,
  // ...
} from '@/types/dtos'

// ✅ 统一的 API 定义
export const api = {
  // Task APIs
  tasks: {
    getAll: () => apiGet<TaskCard[]>('/tasks'),
    getById: (id: string) => apiGet<TaskDetail>(`/tasks/${id}`),
    getForStaging: () => apiGet<TaskCard[]>('/views/staging'),
    getForDaily: (date: string) => apiGet<TaskCard[]>(`/views/daily/${date}`),

    create: (payload: CreateTaskPayload) => apiPost<TaskCard>('/tasks', payload),

    update: (id: string, payload: UpdateTaskPayload) => apiPatch<TaskCard>(`/tasks/${id}`, payload),

    delete: (id: string) => apiDelete(`/tasks/${id}`),

    complete: (id: string) => apiPost<TaskCard>(`/tasks/${id}/complete`, {}),

    reopen: (id: string) => apiPost<TaskCard>(`/tasks/${id}/reopen`, {}),
  },

  // Area APIs
  areas: {
    getAll: () => apiGet<Area[]>('/areas'),
    getById: (id: string) => apiGet<Area>(`/areas/${id}`),
    create: (payload: CreateAreaPayload) => apiPost<Area>('/areas', payload),
    update: (id: string, payload: UpdateAreaPayload) => apiPatch<Area>(`/areas/${id}`, payload),
    delete: (id: string) => apiDelete(`/areas/${id}`),
  },

  // TimeBlock APIs
  timeblocks: {
    getForDate: (date: string) => apiGet<TimeBlockView[]>(`/time-blocks/date/${date}`),

    create: (payload: CreateTimeBlockPayload) => apiPost<TimeBlockView>('/time-blocks', payload),

    update: (id: string, payload: UpdateTimeBlockPayload) =>
      apiPatch<TimeBlockView>(`/time-blocks/${id}`, payload),

    delete: (id: string) => apiDelete(`/time-blocks/${id}`),
  },
}

// Store 中使用
import { api } from '@/api'

async function fetchAllTasks() {
  const tasks = await api.tasks.getAll() // ✅ 类型安全，统一管理
  // ...
}
```

**好处**:

- 所有 API 端点集中管理
- 类型安全
- 易于 mock（测试）
- 易于添加拦截器
- 易于版本控制

---

### 2. 实现 Error Boundary 组件

**当前问题**: 组件错误导致整个应用崩溃。

**建议方案**:

```vue
<!-- src/components/functional/ErrorBoundary.vue -->
<template>
  <div v-if="error" class="error-boundary">
    <div class="error-content">
      <h3>出错了</h3>
      <p>{{ error.message }}</p>
      <button @click="reset">重试</button>
      <button @click="reload">刷新页面</button>
    </div>
  </div>
  <slot v-else />
</template>

<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'
import { logger } from '@/utils/logger'

const TAG = 'ErrorBoundary'

const error = ref<Error | null>(null)

onErrorCaptured((err, instance, info) => {
  logger.error(TAG, 'Component error caught', err, {
    component: instance?.$options.name,
    info,
  })

  error.value = err as Error

  // 阻止错误继续传播
  return false
})

function reset() {
  error.value = null
}

function reload() {
  window.location.reload()
}
</script>
```

**使用**:

```vue
<!-- App.vue -->
<template>
  <ErrorBoundary>
    <RouterView />
  </ErrorBoundary>
</template>
```

---

### 3. 添加请求/响应拦截器

**建议方案**:

```typescript
// src/api/interceptors.ts

import { logger } from '@/utils/logger'
import { correlationTracker } from '@/stores/shared/correlation-tracker'
import { router } from '@/router'

const TAG = 'ApiInterceptor'

// 请求拦截器
export function requestInterceptor(endpoint: string, init: RequestInit): RequestInit {
  const correlationId = crypto.randomUUID()

  // 添加 correlation ID
  const headers = new Headers(init.headers)
  headers.set('X-Correlation-ID', correlationId)

  // 添加认证头（如需要）
  const token = localStorage.getItem('auth_token')
  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  // 开始追踪
  correlationTracker.startTracking(correlationId, `${init.method} ${endpoint}`)

  logger.debug(TAG, `Request: ${init.method} ${endpoint}`, {
    correlationId,
    body: init.body,
  })

  return {
    ...init,
    headers,
  }
}

// 响应拦截器
export function responseInterceptor(response: Response, correlationId: string): Response {
  correlationTracker.finishTracking(correlationId)

  logger.debug(TAG, `Response: ${response.status} ${response.url}`, {
    correlationId,
    status: response.status,
  })

  // 处理 401 未授权
  if (response.status === 401) {
    logger.warn(TAG, 'Unauthorized, redirecting to login')
    router.push('/login')
  }

  // 处理 403 禁止
  if (response.status === 403) {
    logger.error(TAG, 'Forbidden')
    // 显示错误提示
  }

  return response
}

// 错误拦截器
export function errorInterceptor(error: Error, correlationId: string): never {
  correlationTracker.finishTracking(correlationId)

  logger.error(TAG, 'Request failed', error, { correlationId })

  throw error
}
```

---

### 4. 实现 Feature Flags 系统

**用于渐进式发布新功能**:

```typescript
// src/services/feature-flags.ts

import { ref, computed, type Ref } from 'vue'

interface FeatureFlags {
  newDragSystem: boolean
  virtualScrolling: boolean
  darkMode: boolean
  offlineSupport: boolean
  // ...
}

const flags: Ref<FeatureFlags> = ref({
  newDragSystem: false,
  virtualScrolling: false,
  darkMode: false,
  offlineSupport: false,
})

// 从服务器或 localStorage 加载
export async function loadFeatureFlags(): Promise<void> {
  try {
    const response = await fetch('/api/feature-flags')
    const serverFlags = await response.json()
    flags.value = { ...flags.value, ...serverFlags }
  } catch {
    // 使用默认值
  }
}

export function useFeatureFlags() {
  return {
    flags: computed(() => flags.value),
    isEnabled: (feature: keyof FeatureFlags) => flags.value[feature],
    enable: (feature: keyof FeatureFlags) => {
      flags.value[feature] = true
    },
    disable: (feature: keyof FeatureFlags) => {
      flags.value[feature] = false
    },
  }
}
```

**使用**:

```vue
<template>
  <div>
    <!-- 根据 feature flag 切换实现 -->
    <NewDragSystem v-if="features.isEnabled('newDragSystem')" />
    <OldDragSystem v-else />
  </div>
</template>

<script setup lang="ts">
import { useFeatureFlags } from '@/services/feature-flags'

const features = useFeatureFlags()
</script>
```

---

### 5. 改进日志系统

**当前问题**: Logger 存在但使用不一致。

**建议方案**:

```typescript
// src/utils/logger.ts（改进版）

import { ref } from 'vue'

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
  NONE = 4,
}

interface LogEntry {
  timestamp: string
  level: LogLevel
  tag: string
  message: string
  data?: unknown
}

const logLevel = ref(import.meta.env.DEV ? LogLevel.DEBUG : LogLevel.WARN)

const logHistory = ref<LogEntry[]>([])
const MAX_LOG_HISTORY = 1000

// ✅ 添加日志过滤
const enabledTags = ref<Set<string> | null>(null) // null = 所有启用

function shouldLog(level: LogLevel, tag: string): boolean {
  if (level < logLevel.value) return false
  if (enabledTags.value && !enabledTags.value.has(tag)) return false
  return true
}

function log(level: LogLevel, tag: string, message: string, data?: unknown) {
  if (!shouldLog(level, tag)) return

  const entry: LogEntry = {
    timestamp: new Date().toISOString(),
    level,
    tag,
    message,
    data,
  }

  // 保存到历史（用于调试）
  logHistory.value.push(entry)
  if (logHistory.value.length > MAX_LOG_HISTORY) {
    logHistory.value.shift()
  }

  // 控制台输出
  const prefix = `[${entry.timestamp}] [${tag}]`

  switch (level) {
    case LogLevel.DEBUG:
      console.debug(prefix, message, data ?? '')
      break
    case LogLevel.INFO:
      console.info(prefix, message, data ?? '')
      break
    case LogLevel.WARN:
      console.warn(prefix, message, data ?? '')
      break
    case LogLevel.ERROR:
      console.error(prefix, message, data ?? '')
      break
  }
}

export const logger = {
  debug: (tag: string, message: string, data?: unknown) => log(LogLevel.DEBUG, tag, message, data),

  info: (tag: string, message: string, data?: unknown) => log(LogLevel.INFO, tag, message, data),

  warn: (tag: string, message: string, data?: unknown) => log(LogLevel.WARN, tag, message, data),

  error: (tag: string, message: string, error?: unknown, data?: unknown) =>
    log(LogLevel.ERROR, tag, message, { error, ...data }),

  // ✅ 配置方法
  setLevel: (level: LogLevel) => {
    logLevel.value = level
  },

  enableTags: (tags: string[]) => {
    enabledTags.value = new Set(tags)
  },

  enableAllTags: () => {
    enabledTags.value = null
  },

  getHistory: () => logHistory.value,

  clearHistory: () => {
    logHistory.value = []
  },
}

// 开发工具
if (import.meta.env.DEV) {
  ;(window as any).__logger__ = logger
}
```

**使用示例**:

```typescript
// 开发环境只看特定 tag 的日志
logger.enableTags(['TaskStore', 'DragSystem'])

// 临时调整日志级别
logger.setLevel(LogLevel.DEBUG)

// 查看日志历史
console.table(logger.getHistory())
```

---

### 6. 考虑状态机管理复杂流程

**对于拖放这种有复杂状态转换的功能，考虑使用 XState**:

```typescript
// src/composables/drag/drag-machine.ts

import { createMachine, interpret } from 'xstate'

export const dragMachine = createMachine({
  id: 'drag',
  initial: 'idle',
  states: {
    idle: {
      on: {
        DRAG_START: {
          target: 'dragging',
          actions: 'setupDragContext',
        },
      },
    },

    dragging: {
      on: {
        DRAG_OVER: {
          actions: 'updateDropTarget',
        },
        DROP: {
          target: 'processing',
        },
        CANCEL: {
          target: 'idle',
          actions: 'cleanup',
        },
      },
    },

    processing: {
      invoke: {
        src: 'processDrop',
        onDone: {
          target: 'idle',
          actions: 'cleanup',
        },
        onError: {
          target: 'error',
        },
      },
    },

    error: {
      on: {
        RETRY: 'processing',
        DISMISS: {
          target: 'idle',
          actions: 'cleanup',
        },
      },
    },
  },
})
```

**好处**:

- 状态转换可视化
- 不可能进入非法状态
- 易于测试
- 易于理解复杂逻辑

---

## 📈 成功指标

修复完成后，应达到以下指标：

### 代码质量

- [ ] TypeScript strict 模式无错误
- [ ] ESLint 无警告
- [ ] 零 `any` 类型（关键路径）
- [ ] 零注释代码块
- [ ] 测试覆盖率 > 60%

### 性能

- [ ] FCP < 1.5s
- [ ] LCP < 2.5s
- [ ] TTI < 3.5s
- [ ] Bundle size < 500KB (gzipped)

### 可靠性

- [ ] SSE 重连成功率 > 95%
- [ ] 零内存泄漏（24 小时运行）
- [ ] 错误率 < 1%

### 开发体验

- [ ] 所有公共 API 有 JSDoc
- [ ] 关键流程有 E2E 测试
- [ ] 新成员上手时间 < 1 天

---

## 🎬 下一步行动

### 立即开始（本周）

1. **创建 GitHub Issues** - 为每个 CRITICAL 问题创建 issue
2. **删除注释代码** - 1 小时快速胜利（PR #1）
3. **统一 API 客户端** - 开始最重要的重构（PR #2）
4. **启用 TypeScript strict** - 发现隐藏问题（PR #3）

### 本月目标

- 完成所有 8 个 CRITICAL 问题
- 完成至少 10 个 HIGH 优先级问题
- 建立 CI/CD pipeline（ESLint, TypeScript check）
- 代码质量提升 50%

### 季度目标

- 完成完整重构路线图（10 周）
- 测试覆盖率达到 60%+
- 性能指标达标
- 技术债务减少 80%

---

## 📚 参考资源

### 推荐阅读

- [Vue 3 Best Practices](https://vuejs.org/guide/best-practices/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/)
- [Pinia Best Practices](https://pinia.vuejs.org/cookbook/)
- [Clean Code JavaScript](https://github.com/ryanmcdermott/clean-code-javascript)

### 推荐工具

- [Vite Plugin Inspect](https://github.com/antfu/vite-plugin-inspect) - 分析包大小
- [Vue DevTools](https://devtools.vuejs.org/) - 调试
- [Vitest](https://vitest.dev/) - 单元测试
- [Playwright](https://playwright.dev/) - E2E 测试
- [XState](https://xstate.js.org/) - 状态机

---

## ✅ 总结

你的代码库有**良好的架构基础**，但存在**严重的技术债务**，主要集中在：

1. **类型安全**（30+ `any` 类型）
2. **API 一致性**（混用 fetch 和 apiClient）
3. **内存管理**（多处泄漏风险）
4. **可靠性**（SSE 重连、竞态条件）

**建议优先级**：

1. ⭐⭐⭐⭐⭐ **立即修复 8 个 CRITICAL 问题**（约 30-40 小时）
2. ⭐⭐⭐⭐ **修复高频使用的 HIGH 问题**（约 40-50 小时）
3. ⭐⭐⭐ **渐进式处理 MEDIUM 问题**（约 30-40 小时）

**总预估工作量**：一名高级开发者 **10-12 周**完成所有高优先级+问题。

**投资回报**：

- 大幅减少未来 bug
- 提升开发速度 30%+
- 改善用户体验
- 降低维护成本

---

**报告生成日期**: 2025-10-12
**下次审计建议**: 重构完成后（约 3 个月）

# INT 中断管理器架构重构完成报告

## 🎯 重构目标

将 INT 从简单的"去重过滤器"升级为真正的**中断管理器（Interrupt Controller）**，统一管理所有外部事件。

---

## 🏗️ 新架构设计

### 核心原则

**所有入口点（SSE、WebSocket、轮询等）都将事件转发给 INT，由 INT 统一处理和分发。**

```
┌─────────────────────────────────────────────────────────────┐
│                     外部事件源                                │
├─────────────────────────────────────────────────────────────┤
│  [SSE]          [WebSocket]         [Polling]               │
│    ↓                 ↓                   ↓                   │
│    └─────────────────┴───────────────────┘                   │
│                      ↓                                       │
│              ┌───────────────┐                               │
│              │  INT 中断管理器 │                              │
│              │  (Controller)  │                              │
│              └───────┬───────┘                               │
│                      ↓                                       │
│          ┌───────────┴───────────┐                          │
│          ↓                       ↓                          │
│    [去重检查]               [事件分发]                       │
│  (基于中断表)              (按 eventType)                    │
│          ↓                       ↓                          │
│    [丢弃/继续]       ┌───────────┴──────────┐               │
│                      ↓                      ↓               │
│              [TaskStore Handler]    [TrashStore Handler]    │
│              [TimeBlockStore Handler] ...                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 📝 关键修改

### 1. InterruptHandler 升级为中断管理器

**文件**: `src/cpu/interrupt/InterruptHandler.ts`

#### 1.1 新增功能

1. **事件处理器注册系统**：
   ```typescript
   // 注册 handler（类似 EventEmitter）
   interruptHandler.on('task.completed', handler)
   interruptHandler.off('task.completed', handler)
   ```

2. **统一分发入口**：
   ```typescript
   // 所有外部事件通过此方法进入系统
   interruptHandler.dispatch(event: InterruptEvent)
   ```

3. **自动去重检查**：
   - 检查中断表（本机操作）
   - 如果是本机操作 → 丢弃
   - 如果是远程操作 → 分发给所有注册的 handlers

#### 1.2 核心代码

```typescript
export class InterruptHandler {
  // 事件处理器映射：eventType → handlers[]
  private handlers = new Map<string, InterruptEventHandler[]>()
  
  // 中断表：记录本机发起的指令
  private interruptTable = new Map<string, InterruptEntry>()

  /**
   * 注册事件处理器
   */
  on(eventType: string, handler: InterruptEventHandler): void {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, [])
    }
    this.handlers.get(eventType)!.push(handler)
  }

  /**
   * 分发中断事件（统一入口）
   */
  dispatch(event: InterruptEvent): void {
    // 1. 去重检查
    if (event.correlationId) {
      const entry = this.interruptTable.get(event.correlationId)
      if (entry) {
        // 本机已处理，丢弃
        return
      }
    }

    // 2. 分发给对应的 handlers
    const handlers = this.handlers.get(event.eventType) || []
    for (const handler of handlers) {
      handler(event)
    }
  }
}
```

---

### 2. SSE 事件处理器简化

**文件**: `src/infra/events/events.ts`

#### 2.1 修改内容

**之前**：
```typescript
// ❌ 复杂：自己检查去重，自己分发给 handlers
private handleEvent(eventType: string, data: string): void {
  // 解析事件
  const event = JSON.parse(data)
  
  // 检查 INT 去重
  if (interruptHandler.shouldApply(event)) {
    // 分发给所有 handlers
    this.dispatchToHandlers(eventType, event)
  }
}
```

**现在**：
```typescript
// ✅ 简洁：直接转发给 INT
private handleEvent(eventType: string, data: string): void {
  const event = JSON.parse(data)
  
  // 🔥 转发给 INT，由 INT 负责去重和分发
  interruptHandler.dispatch({
    type: InterruptType.SSE,
    eventType: eventType,
    correlationId: event.correlation_id,
    eventId: event.event_id,
    payload: event.payload,
    timestamp: Date.now(),
  })
}
```

---

### 3. Store 事件处理器迁移

#### 3.1 TaskStore (`src/stores/task/event-handlers.ts`)

**之前**：
```typescript
// ❌ 直接注册到 SSE Subscriber
function initEventSubscriptions() {
  const subscriber = getEventSubscriber()
  subscriber.on('task.completed', handleTaskTransactionEvent)
  // ...
}
```

**现在**：
```typescript
// ✅ 注册到 INT（中断管理器）
function initEventSubscriptions() {
  import('@/cpu/interrupt/InterruptHandler').then(({ interruptHandler }) => {
    interruptHandler.on('task.completed', handleTaskTransactionEvent)
    interruptHandler.on('task.updated', handleTaskTransactionEvent)
    // ...
    logger.info(LogTags.STORE_TASKS, 'Task event subscriptions initialized (v4.0 - via INT)')
  })
}
```

**Handler 签名修改**：
```typescript
// 之前：DomainEvent
async function handleTaskTransactionEvent(event: DomainEvent) {
  await transactionProcessor.applyTaskTransaction(event.payload, {
    correlation_id: event.correlation_id,  // ❌
    event_id: event.event_id,              // ❌
    source: 'sse',
  })
}

// 现在：InterruptEvent
async function handleTaskTransactionEvent(event: InterruptEvent) {
  await transactionProcessor.applyTaskTransaction(event.payload, {
    correlation_id: event.correlationId,  // ✅ (驼峰命名)
    event_id: event.eventId,              // ✅
    source: 'sse',
  })
}
```

#### 3.2 TrashStore (`src/stores/trash/event-handlers.ts`)

同样的重构模式：
- 从 `getEventSubscriber()` 改为 `interruptHandler`
- 从 `DomainEvent` 改为 `InterruptEvent`
- v1.0 → v2.0 (via INT)

---

### 4. 日志标签扩展

**文件**: `src/infra/logging/logger.ts`

新增 `SYSTEM_PIPELINE` 标签：
```typescript
export const LogTags = {
  // ...
  SYSTEM_SSE: 'System:SSE',
  SYSTEM_API: 'System:API',
  SYSTEM_COMMAND: 'System:CommandBus',
  SYSTEM_PIPELINE: 'System:Pipeline',  // 🔥 新增
  // ...
}
```

---

## 🎯 架构优势

### 1. **职责清晰**
- **SSE Subscriber**: 只负责接收原始事件，立即转发给 INT
- **INT**: 统一的中断管理器，负责去重、分发
- **Store Handlers**: 只关心业务逻辑，不关心去重和事件来源

### 2. **零冗余**
- ✅ 去重逻辑只在 INT 中实现一次
- ✅ 所有 Store 通过 INT 统一订阅
- ✅ 未来添加 WebSocket/轮询，不需要修改 Store 代码

### 3. **易于扩展**
```typescript
// 新增 WebSocket 事件源（未来）
webSocket.onmessage = (msg) => {
  interruptHandler.dispatch({
    type: InterruptType.WEBSOCKET,
    eventType: msg.type,
    correlationId: msg.correlationId,
    payload: msg.data,
    timestamp: Date.now(),
  })
}
// ✅ Store 无需修改，自动支持！
```

### 4. **类型安全**
```typescript
export interface InterruptEvent {
  type: InterruptType           // SSE | WEBSOCKET | POLLING
  eventType: string             // task.completed, task.updated...
  correlationId?: string        // 用于去重
  eventId?: string             // 事件唯一标识
  payload: any                 // 业务数据
  timestamp: number            // 时间戳
}
```

---

## 📊 完整流程示例

### 场景：用户在机器 A 完成任务

```
[机器 A - 用户操作]
  ↓
pipeline.dispatch('task.complete', { task_id: '123' })
  ↓
[IF] → [SCH] → [EX] → [RES] → [WB]
  ↓
WB: interruptHandler.register(correlationId, {...})
  ↓
[后端] 推送 SSE 事件（correlation_id = 'corr-123'）
  ↓
─────────────────────────────────────────────

[机器 A - SSE 到达]
  ↓
EventSubscriber.handleEvent('task.completed', data)
  ↓
interruptHandler.dispatch({
  type: 'sse',
  eventType: 'task.completed',
  correlationId: 'corr-123',  // 🔥 关键
  payload: { task: {...} }
})
  ↓
INT: 检查中断表 → 找到 'corr-123' → 丢弃 ✅

─────────────────────────────────────────────

[机器 B - SSE 到达]
  ↓
EventSubscriber.handleEvent('task.completed', data)
  ↓
interruptHandler.dispatch({
  type: 'sse',
  eventType: 'task.completed',
  correlationId: 'corr-123',
  payload: { task: {...} }
})
  ↓
INT: 检查中断表 → 未找到 'corr-123' → 分发 ✅
  ↓
TaskStore.handleTaskTransactionEvent(event)
  ↓
transactionProcessor.applyTaskTransaction(...)
  ↓
TaskStore 更新 ✅
```

---

## ✅ 修改文件清单

1. **src/cpu/interrupt/InterruptHandler.ts** - 升级为中断管理器
2. **src/infra/events/events.ts** - 简化为转发器
3. **src/stores/task/event-handlers.ts** - 注册到 INT (v4.0)
4. **src/stores/trash/event-handlers.ts** - 注册到 INT (v2.0)
5. **src/infra/logging/logger.ts** - 新增 SYSTEM_PIPELINE 标签

---

## 🚀 下一步扩展

### 支持 WebSocket

```typescript
// src/infra/websocket/client.ts
export function setupWebSocket() {
  const ws = new WebSocket('ws://...')
  
  ws.onmessage = (msg) => {
    const data = JSON.parse(msg.data)
    
    // 🔥 直接转发给 INT
    interruptHandler.dispatch({
      type: InterruptType.WEBSOCKET,
      eventType: data.type,
      correlationId: data.correlation_id,
      payload: data.payload,
      timestamp: Date.now(),
    })
  }
}
```

### 支持长轮询

```typescript
// src/infra/polling/client.ts
async function pollEvents() {
  const events = await fetch('/api/events/poll')
  
  for (const event of events) {
    // 🔥 直接转发给 INT
    interruptHandler.dispatch({
      type: InterruptType.POLLING,
      eventType: event.type,
      correlationId: event.correlation_id,
      payload: event.payload,
      timestamp: Date.now(),
    })
  }
}
```

---

## 📅 完成时间

**日期**: 2025-10-15

**状态**: ✅ 架构重构完成，所有语法错误已修复

**核心原则**: **入口点只负责转发，INT 统一处理，Store 只关心业务**


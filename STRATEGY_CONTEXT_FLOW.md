# StrategyContext 传入流程详解

**问题**: `StrategyContext` 是在哪里被创建和传入的？

---

## 📊 完整调用链

```
用户拖放操作
  ↓
┌─────────────────────────────────────────────────────────────┐
│ 1️⃣ InteractKanbanColumn.vue (组件层)                        │
│                                                             │
│  onDrop: async (session: DragSession) => {                 │
│    const result = await dragStrategy.executeDrop(          │
│      session,                                              │
│      props.viewKey,                                        │
│      {                                                     │
│        dropIndex: dragPreviewState.value?.computed.dropIndex,│
│        sourceTaskIds: [...],  // 🔥 组件传入              │
│        targetTaskIds: [...],  // 🔥 组件传入              │
│      }                                                     │
│    )                                                       │
│  }                                                         │
└─────────────────────────────────────────────────────────────┘
  ↓
┌─────────────────────────────────────────────────────────────┐
│ 2️⃣ useDragStrategy.ts (Composable 层)                      │
│                                                             │
│  async function executeDrop(                               │
│    session: DragSession,                                   │
│    targetZone: string,                                     │
│    extraContext?: { ... }  // 🔥 接收组件传入的数据        │
│  ) {                                                       │
│    const result = await strategyExecutor.execute(          │
│      session,                                              │
│      targetZone,                                           │
│      extraContext  // 🔥 转发给执行器                      │
│    )                                                       │
│    return result                                           │
│  }                                                         │
└─────────────────────────────────────────────────────────────┘
  ↓
┌─────────────────────────────────────────────────────────────┐
│ 3️⃣ strategy-executor.ts (执行器层)                         │
│                                                             │
│  async execute(                                            │
│    session: DragSession,                                   │
│    targetZone: string,                                     │
│    extraContext?: {                                        │
│      dropIndex?: number,                                   │
│      sourceTaskIds?: string[],                             │
│      targetTaskIds?: string[]                              │
│    }                                                       │
│  ) {                                                       │
│    // 查找策略                                              │
│    const strategy = strategyRegistry.findMatch(...)        │
│                                                             │
│    // 🔥 构建 StrategyContext                              │
│    const context = this.buildContext(                      │
│      session,                                              │
│      targetZone,                                           │
│      strategy,                                             │
│      extraContext  // 传入组件的数据                        │
│    )                                                       │
│                                                             │
│    // 🔥 执行策略，传入 context                             │
│    const result = await strategy.action.execute(context)   │
│                                                             │
│    return result                                           │
│  }                                                         │
└─────────────────────────────────────────────────────────────┘
  ↓
┌─────────────────────────────────────────────────────────────┐
│ 4️⃣ buildContext() 方法 (内部方法)                          │
│                                                             │
│  private buildContext(                                     │
│    session: DragSession,                                   │
│    targetZone: string,                                     │
│    strategy: Strategy,                                     │
│    extraContext?: { ... }                                  │
│  ): StrategyContext {                                      │
│                                                             │
│    // 🔥 从 extraContext 提取任务顺序                       │
│    const sourceTaskIds = extraContext?.sourceTaskIds ?? [] │
│    const targetTaskIds = extraContext?.targetTaskIds ?? [] │
│                                                             │
│    // 🔥 构建完整的 StrategyContext 对象                    │
│    return {                                                │
│      session,              // 从参数                       │
│      targetZone,           // 从参数                       │
│      sourceViewId: session.source.viewId,                  │
│      sourceViewType: session.source.viewType,              │
│      targetViewId: targetZone,                             │
│      targetViewType: this.inferViewType(targetZone),       │
│      task: session.object.data,                            │
│      dropIndex: extraContext?.dropIndex,                   │
│      sourceTaskIds,        // 🔥 从 extraContext           │
│      targetTaskIds,        // 🔥 从 extraContext           │
│      timestamp: Date.now(),                                │
│    }                                                       │
│  }                                                         │
└─────────────────────────────────────────────────────────────┘
  ↓
┌─────────────────────────────────────────────────────────────┐
│ 5️⃣ Strategy.action.execute(context) (策略层)               │
│                                                             │
│  async execute(ctx: StrategyContext) {                     │
│    // 🔥 策略直接使用 ctx 中的数据                          │
│    const sourceSorting = ctx.sourceTaskIds                 │
│    const targetSorting = ctx.targetTaskIds                 │
│    const task = ctx.task                                   │
│    const dropIndex = ctx.dropIndex                         │
│                                                             │
│    // 纯计算                                                │
│    const newOrder = moveTaskWithin(sourceSorting, ...)     │
│                                                             │
│    // 发送命令                                              │
│    await commandBus.emit('view.update_sorting', {...})     │
│                                                             │
│    return { success: true, ... }                           │
│  }                                                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔑 关键点

### 1. **StrategyContext 创建位置**

**文件**: `src/infra/drag/strategy-executor.ts`  
**方法**: `private buildContext()`  
**时机**: 在 `execute()` 方法中，找到匹配策略后立即创建

```typescript
// Line 67
const context = this.buildContext(session, targetZone, strategy, extraContext)
```

### 2. **数据来源**

`StrategyContext` 的数据来自两个地方：

#### A. `DragSession` (会话数据)

```typescript
{
  session,                          // 完整的 DragSession 对象
  sourceViewId: session.source.viewId,
  sourceViewType: session.source.viewType,
  task: session.object.data,
  // ...
}
```

#### B. `extraContext` (组件传入)

```typescript
{
  dropIndex: extraContext?.dropIndex,
  sourceTaskIds: extraContext?.sourceTaskIds ?? [],  // 🔥 关键
  targetTaskIds: extraContext?.targetTaskIds ?? [],  // 🔥 关键
}
```

### 3. **传入策略的时机**

**文件**: `src/infra/drag/strategy-executor.ts`  
**方法**: `execute()`  
**代码**:

```typescript
// Line 89
const result = await strategy.action.execute(context)
```

此时，`context` (StrategyContext) 作为参数传给策略的 `execute` 方法。

---

## 🔄 数据流向图

```
组件的 displayTasks (响应式)
  ↓
  map(t => t.id)
  ↓
sourceTaskIds: string[]  ┐
targetTaskIds: string[]  ├─→ extraContext
dropIndex: number        ┘
  ↓
传给 executeDrop()
  ↓
传给 strategyExecutor.execute()
  ↓
buildContext() 提取数据
  ↓
创建 StrategyContext 对象
  ↓
传给 strategy.action.execute(context)
  ↓
策略使用 ctx.sourceTaskIds, ctx.targetTaskIds
```

---

## 📝 代码追踪

### Step 1: 组件传入数据

**文件**: `src/components/test/InteractKanbanColumn.vue:69`

```typescript
onDrop: async (session) => {
  const result = await dragStrategy.executeDrop(session, props.viewKey, {
    dropIndex: dragPreviewState.value?.computed.dropIndex,
    sourceTaskIds: (session.metadata?.sourceTaskIds as string[]) || [],
    targetTaskIds: displayTasks.value.map((t) => t.id), // 🔥 组件的响应式数据
  })
}
```

### Step 2: Composable 转发

**文件**: `src/composables/drag/useDragStrategy.ts:57`

```typescript
const result = await strategyExecutor.execute(session, targetZone, extraContext)
```

### Step 3: 执行器构建 Context

**文件**: `src/infra/drag/strategy-executor.ts:67`

```typescript
const context = this.buildContext(session, targetZone, strategy, extraContext)
```

### Step 4: buildContext 实现

**文件**: `src/infra/drag/strategy-executor.ts:163-202`

```typescript
private buildContext(
  session: DragSession,
  targetZone: string,
  strategy: Strategy,
  extraContext?: { ... }
): StrategyContext {
  const sourceTaskIds = extraContext?.sourceTaskIds ?? []
  const targetTaskIds = extraContext?.targetTaskIds ?? []

  return {
    session,
    targetZone,
    sourceViewId: session.source.viewId,
    sourceViewType: session.source.viewType,
    targetViewId: targetZone,
    targetViewType: this.inferViewType(targetZone),
    task: session.object.data,
    dropIndex: extraContext?.dropIndex ?? session.target?.dropIndex,
    sourceTaskIds,  // 🔥 从 extraContext
    targetTaskIds,  // 🔥 从 extraContext
    timestamp: Date.now(),
  }
}
```

### Step 5: 策略接收 Context

**文件**: `src/infra/drag/strategies/task-scheduling.ts` (任意策略)

```typescript
async execute(ctx: StrategyContext) {
  const sourceSorting = ctx.sourceTaskIds  // 🔥 直接使用
  const targetSorting = ctx.targetTaskIds  // 🔥 直接使用
  // ...
}
```

---

## ✅ 总结

### 问题：`StrategyContext` 是在哪里被传入的？

**答案**：

1. **创建位置**: `strategy-executor.ts` 的 `buildContext()` 方法
2. **创建时机**: 在 `execute()` 方法中，找到匹配策略后
3. **传入时机**: 调用 `strategy.action.execute(context)` 时
4. **数据来源**:
   - `DragSession` (会话信息)
   - `extraContext` (组件传入的 `sourceTaskIds`, `targetTaskIds`, `dropIndex`)

### 关键设计

- **组件负责**：提供当前的任务顺序（响应式数据快照）
- **执行器负责**：将组件数据和会话数据组合成 `StrategyContext`
- **策略负责**：纯计算，不查询 Store，只使用 `context` 中的数据

这就是为什么重构后策略变成了**纯函数**：所有输入都通过 `StrategyContext` 显式传入！

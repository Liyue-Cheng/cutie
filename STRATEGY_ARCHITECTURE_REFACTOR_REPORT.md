# 策略系统架构重构报告

**日期**: 2025-10-15  
**状态**: ✅ 完成  
**类型**: 架构重构（单向数据流）

---

## 🎯 重构目标

**核心问题**：策略主动查询 Store，打破单向数据流

```typescript
// ❌ 旧设计：策略查询全局 Store
export function getSortedTaskIds(viewKey: string): string[] {
  const viewStore = useViewStore()
  const taskStore = useTaskStore()
  // ... 查询 Store
}
```

**正确设计**：策略是纯函数，所有数据由组件传入

```typescript
// ✅ 新设计：组件传入，策略计算
const result = await strategyExecutor.execute(session, targetZone, {
  sourceTaskIds: displayTasks.value.map((t) => t.id),
  targetTaskIds: targetDisplayTasks.value.map((t) => t.id),
})
```

---

## 📊 重构内容

### 1. 删除 `getSortedTaskIds()` ✅

**文件**: `src/infra/drag/strategies/strategy-utils.ts`

**删除的函数**:

- `getBaseTasksByViewKey()` - 查询 Store 获取任务列表
- `getSortedTaskIds()` - 查询 Store 获取排序后的任务ID

**保留的纯函数**:

- `removeTaskFrom()`
- `insertTaskAt()`
- `moveTaskWithin()`
- `extractDate()`
- `isSameDay()`
- `createOperationRecord()`

---

### 2. 扩展 `StrategyContext` ✅

**文件**: `src/infra/drag/types.ts`

**新增字段**:

```typescript
export interface StrategyContext {
  // ... 原有字段

  // 🔥 新增：当前任务顺序（响应式数据快照，由组件传入）
  sourceTaskIds: string[] // 源视图的任务ID列表
  targetTaskIds: string[] // 目标视图的任务ID列表

  timestamp: number
}
```

---

### 3. 修改策略执行器 ✅

**文件**: `src/infra/drag/strategy-executor.ts`

**修改的方法**:

```typescript
// execute() 方法签名
async execute(
  session: DragSession,
  targetZone: string,
  extraContext?: {
    dropIndex?: number
    sourceTaskIds?: string[]  // 🔥 新增
    targetTaskIds?: string[]  // 🔥 新增
  }
): Promise<StrategyResult>

// buildContext() 方法
private buildContext(..., extraContext?) {
  const sourceTaskIds = extraContext?.sourceTaskIds ?? []
  const targetTaskIds = extraContext?.targetTaskIds ?? []

  // 如果没有传入任务顺序，记录警告
  if (sourceTaskIds.length === 0 || targetTaskIds.length === 0) {
    logger.warn(...)
  }

  return { ..., sourceTaskIds, targetTaskIds }
}
```

---

### 4. 重写所有策略 ✅

**文件**: `src/infra/drag/strategies/task-scheduling.ts`

**修改的策略**:

1. `stagingToDailyStrategy`
2. `dailyToDailyStrategy`
3. `dailyToStagingStrategy`
4. `dailyReorderStrategy`
5. `stagingReorderStrategy`

**修改内容**:

```typescript
// ❌ 旧代码
const sourceSorting = getSortedTaskIds(ctx.sourceViewId)
const targetSorting = getSortedTaskIds(ctx.targetViewId)

// ✅ 新代码
const sourceSorting = ctx.sourceTaskIds
const targetSorting = ctx.targetTaskIds
```

---

### 5. 修改拖放 Composable ✅

**文件**: `src/composables/drag/useDragStrategy.ts`

**修改的方法**:

```typescript
async function executeDrop(
  session: DragSession,
  targetZone: string,
  extraContext?: {
    // 🔥 新增参数
    dropIndex?: number
    sourceTaskIds?: string[]
    targetTaskIds?: string[]
  }
): Promise<StrategyResult>
```

---

### 6. 数据传递链路 ✅

#### Step 1: 组件收集任务顺序

**文件**: `src/composables/drag/useInteractDrag.ts`

```typescript
const getDragData = (element: HTMLElement): DragData => {
  // ...
  return {
    type: 'task',
    task,
    sourceView: viewMetadata.value,
    index,
    // 🔥 传递任务顺序给策略系统
    taskIds: displayTasks.value.map((t) => t.id),
  }
}
```

#### Step 2: 保存到 DragSession

**文件**: `src/infra/drag-interact/drag-controller.ts`

```typescript
const session: DragSession = {
  // ...
  metadata: {
    date: (dragData.sourceView.config as any).date,
    areaId: dragData.task.area_id || undefined,
    // 🔥 保存源视图的任务顺序
    sourceTaskIds: dragData.taskIds,
  },
}
```

#### Step 3: 组件传入策略执行器

**文件**: `src/components/test/InteractKanbanColumn.vue`

```typescript
onDrop: async (session) => {
  const result = await dragStrategy.executeDrop(session, props.viewKey, {
    dropIndex: dragPreviewState.value?.computed.dropIndex,
    // 源视图的任务顺序（从 session.metadata 中获取）
    sourceTaskIds: (session.metadata?.sourceTaskIds as string[]) || [],
    // 目标视图的任务顺序（当前组件的 displayTasks）
    targetTaskIds: displayTasks.value.map((t) => t.id),
  })
}
```

---

## 🔄 完整数据流

```
用户拖动任务
  ↓
1️⃣ useInteractDrag.getDragData()
   → 读取 displayTasks (响应式)
   → 返回 { task, taskIds: [...] }
  ↓
2️⃣ drag-controller.startPreparing()
   → 创建 DragSession
   → metadata.sourceTaskIds = dragData.taskIds
  ↓
3️⃣ 用户松开鼠标 (drop)
  ↓
4️⃣ InteractKanbanColumn.onDrop()
   → 获取 sourceTaskIds (从 session.metadata)
   → 获取 targetTaskIds (从当前 displayTasks)
   → 调用 dragStrategy.executeDrop(..., extraContext)
  ↓
5️⃣ strategyExecutor.execute()
   → buildContext(extraContext)
   → 传入策略: ctx.sourceTaskIds, ctx.targetTaskIds
  ↓
6️⃣ 策略执行 (纯计算)
   → const newOrder = moveTaskWithin(ctx.sourceTaskIds, ...)
   → emit 'view.update_sorting' 命令
  ↓
7️⃣ CommandBus → Handler → API → Store
   → Store 更新
   → 组件响应式更新
   → displayTasks 自动刷新 ✅
```

---

## ✅ 架构优势

### 1. 单向数据流

```
组件 (数据源) → 策略 (纯计算) → Command → Store → 组件 (响应式更新)
```

- 策略不查询 Store
- 数据流向清晰
- 易于追踪和调试

### 2. 可测试性

```typescript
// ❌ 难以测试：依赖全局 Store
getSortedTaskIds('daily::2025-10-16')

// ✅ 容易测试：纯函数
moveTaskWithin(['task-1', 'task-2', 'task-3'], 'task-2', 0)
// => ['task-2', 'task-1', 'task-3']
```

### 3. 数据一致性

- **组件**：始终显示最新的响应式数据
- **策略**：接收执行时刻的快照，避免时序问题
- **Store**：只负责存储，不被策略直接查询

### 4. 解耦合

- 策略层不依赖 Pinia Store
- 可以独立测试策略逻辑
- 未来可以迁移到其他状态管理方案

---

## 📝 类型定义更新

### DragData

```typescript
export interface DragData {
  type: 'task'
  task: TaskCard
  sourceView: ViewMetadata
  index: number
  taskIds: string[] // 🔥 新增
}
```

### StrategyContext

```typescript
export interface StrategyContext {
  session: DragSession
  targetZone: string
  sourceViewId: string
  targetViewId: string
  task: TaskCard
  dropIndex?: number
  sourceTaskIds: string[] // 🔥 新增
  targetTaskIds: string[] // 🔥 新增
  timestamp: number
}
```

---

## 🧪 测试验证

### 测试场景 1: Staging → Daily

```typescript
// 组件传入
{
  sourceTaskIds: ['task-1', 'task-2', 'task-3'],
  targetTaskIds: ['task-4', 'task-5'],
  dropIndex: 1
}

// 策略计算
newSourceOrder = removeTaskFrom(['task-1', 'task-2', 'task-3'], 'task-2')
// => ['task-1', 'task-3']

newTargetOrder = insertTaskAt(['task-4', 'task-5'], 'task-2', 1)
// => ['task-4', 'task-2', 'task-5']

// 结果
✅ 命令发送：view.update_sorting (staging)
✅ 命令发送：view.update_sorting (daily)
```

### 测试场景 2: Daily 内部排序

```typescript
// 组件传入
{
  sourceTaskIds: ['task-1', 'task-2', 'task-3'],
  targetTaskIds: ['task-1', 'task-2', 'task-3'],  // 同一视图
  dropIndex: 0
}

// 策略计算
newOrder = moveTaskWithin(['task-1', 'task-2', 'task-3'], 'task-2', 0)
// => ['task-2', 'task-1', 'task-3']

// 结果
✅ 命令发送：view.update_sorting (daily)
```

---

## ✅ 验收检查

- [x] 删除 `getSortedTaskIds()` 和 `getBaseTasksByViewKey()`
- [x] 修复 `extractDate()` 的类型错误
- [x] 扩展 `StrategyContext` 添加 `sourceTaskIds` 和 `targetTaskIds`
- [x] 修改 `strategy-executor.ts` 传入任务顺序
- [x] 重写所有 5 个策略使用 `ctx.sourceTaskIds` 和 `ctx.targetTaskIds`
- [x] 修改 `useDragStrategy.ts` 接受额外上下文
- [x] 修改 `useInteractDrag.ts` 在 `getDragData` 中传递 `taskIds`
- [x] 修改 `DragData` 类型添加 `taskIds` 字段
- [x] 修改 `drag-controller.ts` 保存 `sourceTaskIds` 到 `session.metadata`
- [x] 修改 `InteractKanbanColumn.vue` 传入任务顺序
- [x] 所有文件通过 Linter 检查

---

## 📚 相关文档

1. [策略链实现完成报告](STRATEGY_IMPLEMENTATION_COMPLETE.md)
2. [策略链设计](src/infra/drag/STRATEGY_CHAIN_DESIGN.md)
3. [策略匹配流程](src/infra/drag/STRATEGY_MATCHING_FLOW.md)
4. [策略系统总览](src/infra/drag/README.md)

---

## 🔜 下一步

1. **退出 PRINT MODE**：将 `console.log` 替换为真实的 `commandBus.emit`
2. **集成 InstructionTracker**：自动追踪策略执行的每一步
3. **实现事务回滚**：如果某步失败，自动回滚所有操作
4. **完整测试**：在浏览器中测试所有拖放场景

---

**状态**: ✅ 架构重构完成  
**破坏性变更**: 是（策略系统 API 完全改变）  
**向后兼容**: 否（旧的 `useCrossViewDrag` 等 composable 仍可用，但新策略系统独立运行）  
**Linter 状态**: ✅ 无错误

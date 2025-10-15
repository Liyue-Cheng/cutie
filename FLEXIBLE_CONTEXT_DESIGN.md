# 灵活上下文设计（V2）

**日期**: 2025-10-15  
**版本**: V2.0  
**状态**: ✅ 完成

---

## 🎯 设计理念

**核心思想**：让组件自由传递任意数据，策略自行解包，最大化灵活性。

### 为什么需要 V2？

#### V1 的问题：字段固定，扩展性差

```typescript
// ❌ V1: 固定字段
interface StrategyContext {
  sourceTaskIds: string[]
  targetTaskIds: string[]
  dropIndex?: number
  // 未来需要新数据？必须修改接口！
}
```

**痛点**：

- 每次需要新数据都要修改 `StrategyContext` 接口
- 策略无法获取特定场景的自定义数据
- 组件被限制只能传递预定义的字段

#### V2 的解决方案：灵活的 JSON 上下文

```typescript
// ✅ V2: 自由扩展
interface StrategyContext {
  sourceContext: Record<string, any> // 起始组件传入任意数据
  targetContext: Record<string, any> // 结束组件传入任意数据
}
```

**优势**：

- ✅ 组件可以传递任意数据
- ✅ 策略自行决定需要什么数据
- ✅ 无需修改接口即可扩展
- ✅ 类型安全由策略保证

---

## 📊 新架构设计

### 类型定义

```typescript
/**
 * 策略执行上下文（V2）
 */
export interface StrategyContext {
  // 拖放会话（不变）
  session: DragSession
  targetZone: string

  // 便捷访问（不变）
  sourceViewId: string
  sourceViewType: ViewType
  targetViewId: string
  targetViewType: ViewType
  task: TaskCard
  dropIndex?: number

  // 🔥 灵活的上下文数据（V2 新增）
  sourceContext: Record<string, any> // 起始组件传入的所有数据
  targetContext: Record<string, any> // 结束组件传入的所有数据

  timestamp: number
}

/**
 * 常见的上下文数据结构（供参考，非强制）
 */
export interface CommonSourceContext {
  taskIds?: string[]
  displayTasks?: TaskCard[]
  viewConfig?: Record<string, any>
  [key: string]: any // 允许任意扩展
}

export interface CommonTargetContext {
  taskIds?: string[]
  displayTasks?: TaskCard[]
  dropIndex?: number
  viewConfig?: Record<string, any>
  [key: string]: any // 允许任意扩展
}
```

---

## 🔄 完整数据流

```
起始组件 (InteractKanbanColumn A)
  ↓
getDragData() {
  sourceContext: {
    taskIds: [...],
    displayTasks: [...],
    viewKey: '...',
    customData: ...  // 🔥 可以自由添加
  }
}
  ↓
保存到 DragSession.metadata.sourceContext
  ↓
用户拖动到目标组件
  ↓
结束组件 (InteractKanbanColumn B)
  ↓
onDrop(session) {
  targetContext: {
    taskIds: [...],
    displayTasks: [...],
    dropIndex: ...,
    viewKey: '...',
    customData: ...  // 🔥 可以自由添加
  }
}
  ↓
strategyExecutor.execute(session, targetZone, {
  sourceContext,  // 从 session.metadata 获取
  targetContext   // 当前组件提供
})
  ↓
buildContext() {
  return {
    sourceContext,  // 原样传递
    targetContext   // 原样传递
  }
}
  ↓
strategy.action.execute(ctx) {
  // 🔥 策略自行解包需要的数据
  const sourceTaskIds = extractTaskIds(ctx.sourceContext)
  const targetTaskIds = extractTaskIds(ctx.targetContext)
  const customData = ctx.sourceContext.customData
}
```

---

## 🛠️ 使用示例

### 1. 组件端：传入数据

```typescript
// src/components/test/InteractKanbanColumn.vue

onDrop: async (session) => {
  const result = await dragStrategy.executeDrop(session, props.viewKey, {
    // 起始组件的上下文（从 session 获取）
    sourceContext: session.metadata?.sourceContext || {},

    // 结束组件的上下文（当前组件提供）
    targetContext: {
      // 标准数据
      taskIds: displayTasks.value.map(t => t.id),
      displayTasks: displayTasks.value,
      dropIndex: dragPreviewState.value?.computed.dropIndex,
      viewKey: props.viewKey,

      // 🔥 自定义数据（完全自由）
      isFilterActive: someFilterState.value,
      sortBy: currentSortOrder.value,
      viewMode: 'kanban',
      customSettings: {...},
      // ... 任意数据
    },
  })
}
```

### 2. 策略端：解包数据

```typescript
// src/infra/drag/strategies/task-scheduling.ts

export const myCustomStrategy: Strategy = {
  id: 'my-custom-strategy',
  name: 'My Custom Strategy',

  action: {
    async execute(ctx: StrategyContext) {
      // 使用辅助函数提取标准数据
      const sourceTaskIds = extractTaskIds(ctx.sourceContext)
      const targetTaskIds = extractTaskIds(ctx.targetContext)

      // 🔥 直接访问自定义数据
      const isFiltered = ctx.targetContext.isFilterActive
      const sortBy = ctx.targetContext.sortBy
      const customSettings = ctx.targetContext.customSettings

      // 根据自定义数据做不同的处理
      if (isFiltered) {
        console.log('目标视图正在过滤，特殊处理...')
      }

      if (sortBy === 'priority') {
        console.log('目标视图按优先级排序，调整插入逻辑...')
      }

      // ... 策略逻辑
    },
  },
}
```

### 3. 辅助函数：提取数据

```typescript
// src/infra/drag/strategies/strategy-utils.ts

/**
 * 从上下文中提取任务ID列表
 *
 * 支持多种格式：
 * - taskIds: string[]
 * - displayTasks: TaskCard[]
 * - 自动回退到空数组
 */
export function extractTaskIds(context: Record<string, any>): string[] {
  if (Array.isArray(context.taskIds)) {
    return context.taskIds
  }

  if (Array.isArray(context.displayTasks)) {
    return context.displayTasks.map((t: any) => t.id)
  }

  console.warn('[strategy-utils] No taskIds found in context')
  return []
}
```

---

## 🎨 扩展场景示例

### 场景 1：日历视图 → 看板视图

```typescript
// 日历组件传入
sourceContext: {
  taskIds: [...],
  viewType: 'calendar',
  selectedDate: '2025-10-15',
  timeSlot: { start: '09:00', end: '10:00' },
  isAllDay: false
}

// 看板组件传入
targetContext: {
  taskIds: [...],
  viewType: 'kanban',
  columnStatus: 'in_progress',
  swimlane: 'backend-team'
}

// 策略可以根据 viewType 做不同处理
if (ctx.sourceContext.viewType === 'calendar') {
  console.log('从日历拖入，时间信息:', ctx.sourceContext.timeSlot)
}
```

### 场景 2：筛选视图 → 普通视图

```typescript
// 源组件（筛选视图）
sourceContext: {
  taskIds: [...],
  filter: {
    status: 'incomplete',
    priority: 'high',
    assignee: 'user-123'
  },
  totalCount: 50,
  displayedCount: 10
}

// 策略可以利用筛选信息
const filter = ctx.sourceContext.filter
console.log(`从筛选视图拖出（筛选条件：${JSON.stringify(filter)}）`)
```

### 场景 3：项目视图 → 区域视图

```typescript
// 项目视图传入
sourceContext: {
  taskIds: [...],
  projectId: 'proj-123',
  projectName: 'Web Redesign',
  milestone: 'Phase 2'
}

// 区域视图传入
targetContext: {
  taskIds: [...],
  areaId: 'area-456',
  areaName: 'Development',
  areaColor: '#3b82f6'
}

// 策略可以利用项目和区域信息
console.log(`从项目 ${ctx.sourceContext.projectName} 移动到区域 ${ctx.targetContext.areaName}`)
```

---

## ✅ 优势总结

### 1. 灵活性

- 组件可以传递任意数据
- 策略可以访问任意数据
- 无需修改接口即可扩展

### 2. 可维护性

- 添加新数据不破坏现有代码
- 策略自行决定需要什么数据
- 不用的数据会被忽略

### 3. 类型安全

- `Record<string, any>` 在传递时是灵活的
- 策略内部可以使用类型断言
- 可以定义常见的类型供参考（`CommonSourceContext`, `CommonTargetContext`）

### 4. 向后兼容

- 策略可以同时支持旧字段和新字段
- 使用 `extractTaskIds()` 等辅助函数自动适配

---

## 📝 最佳实践

### 1. 组件端：传入数据

**原则**：传递策略可能需要的所有数据

```typescript
targetContext: {
  // 基础数据（必需）
  taskIds: displayTasks.value.map(t => t.id),
  dropIndex: dragPreviewState.value?.computed.dropIndex,

  // 视图信息
  viewKey: props.viewKey,
  viewType: 'kanban',

  // 完整数据（可选，但推荐）
  displayTasks: displayTasks.value,

  // 自定义数据（按需）
  customData: {...}
}
```

### 2. 策略端：解包数据

**原则**：使用辅助函数 + 直接访问

```typescript
async execute(ctx: StrategyContext) {
  // 使用辅助函数提取标准数据
  const sourceTaskIds = extractTaskIds(ctx.sourceContext)
  const targetTaskIds = extractTaskIds(ctx.targetContext)

  // 直接访问自定义数据（带类型断言）
  const viewType = ctx.targetContext.viewType as string | undefined
  const customData = ctx.targetContext.customData as MyCustomType | undefined

  // 提供默认值
  const dropIndex = ctx.targetContext.dropIndex ?? targetTaskIds.length
}
```

### 3. 辅助函数：通用提取

**原则**：支持多种格式，提供默认值

```typescript
export function extractTaskIds(context: Record<string, any>): string[] {
  if (Array.isArray(context.taskIds)) return context.taskIds
  if (Array.isArray(context.displayTasks)) return context.displayTasks.map((t) => t.id)
  return []
}

export function extractDropIndex(context: Record<string, any>): number | undefined {
  if (typeof context.dropIndex === 'number') return context.dropIndex
  return undefined
}
```

---

## 🔧 技术实现

### 文件修改清单

1. ✅ `src/infra/drag/types.ts` - 修改 `StrategyContext` 接口
2. ✅ `src/infra/drag/strategy-executor.ts` - 修改 `buildContext()` 方法
3. ✅ `src/infra/drag/strategies/strategy-utils.ts` - 添加 `extractTaskIds()` 函数
4. ✅ `src/infra/drag/strategies/task-scheduling.ts` - 所有策略使用 `extractTaskIds()`
5. ✅ `src/composables/drag/useDragStrategy.ts` - 修改 `executeDrop()` 签名
6. ✅ `src/composables/drag/useInteractDrag.ts` - 修改 `getDragData()` 返回 `sourceContext`
7. ✅ `src/infra/drag-interact/types.ts` - 修改 `DragData` 接口
8. ✅ `src/infra/drag-interact/drag-controller.ts` - 保存 `sourceContext` 到 `session.metadata`
9. ✅ `src/components/test/InteractKanbanColumn.vue` - 传入 `sourceContext` 和 `targetContext`

---

## 🧪 测试验证

### 测试用例 1：基础数据

```typescript
// 组件传入
sourceContext: { taskIds: ['task-1', 'task-2'] }
targetContext: { taskIds: ['task-3', 'task-4'], dropIndex: 1 }

// 策略使用
const sourceTaskIds = extractTaskIds(ctx.sourceContext)  // ['task-1', 'task-2']
const targetTaskIds = extractTaskIds(ctx.targetContext)  // ['task-3', 'task-4']
const dropIndex = ctx.targetContext.dropIndex  // 1
```

### 测试用例 2：自定义数据

```typescript
// 组件传入
sourceContext: {
  taskIds: ['task-1'],
  customFlag: true,
  metadata: { source: 'calendar' }
}
targetContext: {
  taskIds: ['task-2'],
  customFlag: false,
  metadata: { target: 'kanban' }
}

// 策略使用
const isCustomSource = ctx.sourceContext.customFlag  // true
const sourceType = ctx.sourceContext.metadata?.source  // 'calendar'
const targetType = ctx.targetContext.metadata?.target  // 'kanban'
```

---

## 📚 相关文档

1. [策略架构重构报告](STRATEGY_ARCHITECTURE_REFACTOR_REPORT.md)
2. [策略上下文流程](STRATEGY_CONTEXT_FLOW.md)
3. [策略链设计](src/infra/drag/STRATEGY_CHAIN_DESIGN.md)

---

**版本**: V2.0  
**状态**: ✅ 生产就绪  
**Linter**: ✅ 无错误  
**向后兼容**: ✅ 使用 `extractTaskIds()` 等辅助函数自动适配

# 策略匹配流程详解

## 🎯 策略系统架构

```
用户拖放操作
  ↓
drag-controller 创建 DragSession
  ├─ source: { viewId, viewType, viewKey, elementId }
  ├─ object: { type, data (TaskCard), originalIndex }
  ├─ dragMode: 'normal' | 'copy' | 'scheduled'
  └─ target?: { viewId, viewType, viewKey, dropIndex }
  ↓
dropzone.drop 事件 → InteractKanbanColumn.onDrop(session)
  ↓
useDragStrategy.executeDrop(session, targetZone)
  ↓
strategyExecutor.execute(session, targetZone)
  ├─ 1️⃣ strategyRegistry.findMatch()
  │    ├─ 遍历所有已注册策略（按优先级降序）
  │    ├─ 对每个策略调用 matchStrategy()
  │    └─ 返回第一个匹配的策略
  │
  ├─ 2️⃣ buildContext() - 构建执行上下文
  │    └─ StrategyContext { session, targetZone, task, dropIndex, ... }
  │
  ├─ 3️⃣ printStrategyInfo() - 打印策略详情（PRINT MODE）
  │
  ├─ 4️⃣ strategy.action.canExecute() - 前置检查（可选）
  │
  └─ 5️⃣ strategy.action.execute(context) - 执行策略
       └─ 返回 StrategyResult
```

---

## 🔍 匹配算法详解

### 1. Registry 查找流程

**文件**: `src/infra/drag/strategy-registry.ts`

```typescript
findMatch(session: DragSession, targetZone: string): Strategy | null {
  // 遍历已排序的策略列表（按优先级降序）
  for (const strategy of this.sortedStrategies) {
    // 1. 跳过禁用的策略
    if (strategy.enabled === false) {
      continue
    }

    // 2. 调用匹配算法
    if (matchStrategy(strategy.conditions, session, targetZone)) {
      // 3. 返回第一个匹配的策略
      return strategy
    }
  }

  // 4. 没有匹配的策略
  return null
}
```

**关键点**:

- ✅ **优先级优先**: 高优先级策略先匹配
- ✅ **第一匹配原则**: 找到第一个匹配就返回，不继续查找
- ✅ **跳过禁用**: `enabled === false` 的策略被忽略

---

### 2. 匹配算法（matchStrategy）

**文件**: `src/infra/drag/strategy-matcher.ts`

```typescript
function matchStrategy(
  condition: StrategyCondition,
  session: DragSession,
  targetZone: string
): boolean {
  // 1️⃣ 匹配源视图
  if (condition.source) {
    if (!matchSource(condition.source, session)) {
      return false // 源不匹配 → 整个策略不匹配
    }
  }

  // 2️⃣ 匹配目标视图
  if (condition.target) {
    if (!matchTarget(condition.target, targetZone, session)) {
      return false // 目标不匹配 → 整个策略不匹配
    }
  }

  // 3️⃣ 匹配拖放模式
  if (condition.dragMode && session.dragMode !== condition.dragMode) {
    return false // 模式不匹配 → 整个策略不匹配
  }

  // ✅ 所有条件都匹配
  return true
}
```

**逻辑**: `source AND target AND dragMode` 必须全部匹配

---

### 3. 源视图匹配（matchSource）

```typescript
function matchSource(condition: SourceCondition, session: DragSession): boolean {
  // 🔹 匹配视图类型 (viewType)
  if (condition.viewType) {
    const types = Array.isArray(condition.viewType) ? condition.viewType : [condition.viewType]
    if (!types.includes(session.source.viewType)) {
      return false // viewType 不匹配
    }
  }

  // 🔹 匹配视图键 (viewKey) - 支持字符串或正则
  if (condition.viewKey) {
    if (condition.viewKey instanceof RegExp) {
      // 正则匹配
      if (!condition.viewKey.test(session.source.viewKey)) {
        return false // 正则不匹配
      }
    } else {
      // 精确匹配
      if (session.source.viewKey !== condition.viewKey) {
        return false // 字符串不匹配
      }
    }
  }

  // 🔹 匹配任务状态 (taskStatus)
  if (condition.taskStatus) {
    const statuses = Array.isArray(condition.taskStatus)
      ? condition.taskStatus
      : [condition.taskStatus]
    if (!statuses.includes(session.object.data.schedule_status)) {
      return false // taskStatus 不匹配
    }
  }

  // 🔹 自定义检查 (customCheck)
  if (condition.customCheck) {
    if (!condition.customCheck(session)) {
      return false // 自定义检查失败
    }
  }

  // ✅ 所有源条件都匹配
  return true
}
```

**检查顺序**: `viewType → viewKey → taskStatus → customCheck`

---

### 4. 目标视图匹配（matchTarget）

```typescript
function matchTarget(
  condition: TargetCondition,
  targetZone: string,
  session: DragSession
): boolean {
  // 🔹 匹配视图类型 (viewType)
  if (condition.viewType) {
    const types = Array.isArray(condition.viewType) ? condition.viewType : [condition.viewType]
    const targetViewType = session.target?.viewType
    if (targetViewType && !types.includes(targetViewType)) {
      return false // viewType 不匹配
    }
  }

  // 🔹 匹配视图键 (viewKey) - 支持字符串或正则
  if (condition.viewKey) {
    if (condition.viewKey instanceof RegExp) {
      // 正则匹配
      if (!condition.viewKey.test(targetZone)) {
        return false // 正则不匹配
      }
    } else {
      // 精确匹配
      if (targetZone !== condition.viewKey) {
        return false // 字符串不匹配
      }
    }
  }

  // 🔹 匹配接受的任务状态 (acceptsStatus)
  if (condition.acceptsStatus) {
    if (!condition.acceptsStatus.includes(session.object.data.schedule_status)) {
      return false // 目标不接受此状态的任务
    }
  }

  // 🔹 自定义检查 (customCheck)
  if (condition.customCheck) {
    if (!condition.customCheck(targetZone, session)) {
      return false // 自定义检查失败
    }
  }

  // ✅ 所有目标条件都匹配
  return true
}
```

**检查顺序**: `viewType → viewKey → acceptsStatus → customCheck`

---

## 📝 实际匹配示例

### 示例 1: Staging 内部拖放

**操作**: 在 `misc::staging` 内部拖动任务

**Session 数据**:

```typescript
{
  source: {
    viewKey: 'misc::staging',
    viewType: 'status',
    // ...
  },
  object: {
    data: {
      schedule_status: 'staging',
      // ...
    }
  },
  dragMode: 'normal'
}
```

**targetZone**: `'misc::staging'`

**匹配的策略**: `staging-reorder`

```typescript
{
  id: 'staging-reorder',
  conditions: {
    source: {
      viewKey: 'misc::staging',  // ✅ 精确匹配
    },
    target: {
      viewKey: 'misc::staging',  // ✅ 精确匹配
    },
    priority: 80,
  }
}
```

**匹配过程**:

1. ✅ `condition.source.viewKey === 'misc::staging'` → `session.source.viewKey === 'misc::staging'` ✓
2. ✅ `condition.target.viewKey === 'misc::staging'` → `targetZone === 'misc::staging'` ✓
3. ✅ 没有 `dragMode` 限制
4. ✅ 没有 `taskStatus` 限制
5. **结果**: 匹配成功 ✓

---

### 示例 2: Staging → Daily

**操作**: 从 `misc::staging` 拖动到 `daily::2025-10-15`

**Session 数据**:

```typescript
{
  source: {
    viewKey: 'misc::staging',
    viewType: 'status',
    // ...
  },
  object: {
    data: {
      schedule_status: 'staging',
      // ...
    }
  },
  dragMode: 'normal'
}
```

**targetZone**: `'daily::2025-10-15'`

**匹配的策略**: `staging-to-daily`

```typescript
{
  id: 'staging-to-daily',
  conditions: {
    source: {
      viewKey: 'misc::staging',     // ✅ 精确匹配
      taskStatus: 'staging',        // ✅ 状态匹配
    },
    target: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,  // ✅ 正则匹配
    },
    priority: 100,
  }
}
```

**匹配过程**:

1. ✅ `condition.source.viewKey === 'misc::staging'` → `session.source.viewKey === 'misc::staging'` ✓
2. ✅ `condition.source.taskStatus === 'staging'` → `session.object.data.schedule_status === 'staging'` ✓
3. ✅ `condition.target.viewKey.test('daily::2025-10-15')` → `/^daily::\d{4}-\d{2}-\d{2}$/.test('daily::2025-10-15')` ✓
4. **结果**: 匹配成功 ✓

---

### 示例 3: Daily → Daily (同日期)

**操作**: 在 `daily::2025-10-15` 内部拖动任务

**Session 数据**:

```typescript
{
  source: {
    viewKey: 'daily::2025-10-15',
    viewType: 'date',
    // ...
  },
  object: {
    data: {
      schedule_status: 'scheduled',
      // ...
    }
  },
  dragMode: 'normal'
}
```

**targetZone**: `'daily::2025-10-15'`

**匹配的策略**: `daily-to-daily`

```typescript
{
  id: 'daily-to-daily',
  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,  // ✅ 正则匹配
      taskStatus: 'scheduled',                // ✅ 状态匹配
    },
    target: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,  // ✅ 正则匹配
    },
    priority: 90,
  }
}
```

**匹配过程**:

1. ✅ `condition.source.viewKey.test('daily::2025-10-15')` ✓
2. ✅ `condition.source.taskStatus === 'scheduled'` → `session.object.data.schedule_status === 'scheduled'` ✓
3. ✅ `condition.target.viewKey.test('daily::2025-10-15')` ✓
4. **结果**: 匹配成功 ✓

**策略内部逻辑**:

```typescript
async execute(ctx) {
  const sourceDate = ctx.sourceViewId.split('::')[1]
  const targetDate = ctx.targetZone.split('::')[1]

  if (sourceDate === targetDate) {
    // 同日期 → 重新排序
    console.log('🔄 [PRINT MODE] Would reorder task in same day')
    return { success: true, reorderOnly: true }
  } else {
    // 不同日期 → 重新安排
    console.log('📆 [PRINT MODE] Would reschedule task')
    return { success: true, affectedViews: [sourceView, targetView] }
  }
}
```

---

## 🎯 优先级系统

### 当前注册的策略及其优先级

| 策略 ID            | 优先级  | 匹配条件                                  |
| ------------------ | ------- | ----------------------------------------- |
| `staging-to-daily` | **100** | `misc::staging` → `daily::YYYY-MM-DD`     |
| `daily-to-staging` | **95**  | `daily::YYYY-MM-DD` → `misc::staging`     |
| `daily-to-daily`   | **90**  | `daily::YYYY-MM-DD` → `daily::YYYY-MM-DD` |
| `staging-reorder`  | **80**  | `misc::staging` → `misc::staging`         |

**为什么这样排序？**

1. **`staging-to-daily` (100)**: 最高优先级
   - 原因：最明确的跨视图操作，避免被其他策略误匹配
2. **`daily-to-staging` (95)**: 第二优先级
   - 原因：退回操作，明确的逆向流程
3. **`daily-to-daily` (90)**: 第三优先级
   - 原因：可能是同日期重排序，也可能是跨日期移动，优先于纯排序
4. **`staging-reorder` (80)**: 最低优先级
   - 原因：最通用的操作，作为兜底

---

## 🔍 匹配失败的常见原因

### 1. viewKey 不匹配

```typescript
// ❌ 错误：session.source.viewKey = 'staging'
// ✓ 正确：session.source.viewKey = 'misc::staging'

// 策略定义
source: {
  viewKey: 'misc::staging' // 必须完全匹配（包括前缀）
}
```

**修复**: 确保 `drag-controller` 创建的 `session.source.viewKey` 格式正确

---

### 2. taskStatus 类型错误

```typescript
// ❌ 错误：使用 'planned' / 'completed'
source: {
  taskStatus: ['planned', 'in_progress', 'completed'] // 这些是 DailyOutcome
}

// ✓ 正确：使用 'scheduled' / 'staging'
source: {
  taskStatus: 'scheduled' // 这是 ScheduleStatus
}
```

**修复**: 使用正确的 `ScheduleStatus` 类型（`'scheduled' | 'staging'`）

---

### 3. 正则表达式错误

```typescript
// ❌ 错误：忘记转义特殊字符
target: {
  viewKey: /^daily::\d{4}-\d{2}-\d{2}$/ // ✓ 正确
}

// ❌ 错误：错误的正则
target: {
  viewKey: /daily::.*/ // 太宽泛，会匹配不该匹配的
}
```

**修复**: 使用精确的正则表达式

---

### 4. 优先级冲突

```typescript
// 两个策略都匹配，但优先级低的永远不会被选中
Strategy A: priority: 100  ← 总是被选中
Strategy B: priority: 50   ← 永远不会被选中（如果 A 也匹配）
```

**修复**: 调整优先级，或者让条件更精确

---

## 🛠️ 调试工具

### 1. 查看所有已注册策略

```typescript
// 在浏览器控制台
window.strategyRegistry.debug()
```

**输出**:

```
🎯 Drag Strategy Registry
Total strategies: 4

ID                  | Name                        | Priority | Enabled | Tags
--------------------|----------------------------|----------|---------|------------------
staging-to-daily    | Staging to Daily Schedule   | 100      | ✓       | scheduling, staging, daily
daily-to-staging    | Daily to Staging Return     | 95       | ✓       | scheduling, staging, daily, return
daily-to-daily      | Daily to Daily Reschedule   | 90       | ✓       | scheduling, daily, reschedule
staging-reorder     | Staging Internal Reorder    | 80       | ✓       | scheduling, staging, reorder
```

---

### 2. 查看当前拖放的匹配情况

```typescript
// 在 InteractKanbanColumn.onDrop 中
const debugInfo = window.strategyExecutor.getDebugInfo(session, targetZone)
console.log('Debug Info:', debugInfo)
```

**输出**:

```typescript
{
  allMatches: [  // 所有匹配的策略
    { id: 'staging-reorder', name: 'Staging Internal Reorder', priority: 80 }
  ],
  bestMatch: {  // 最佳匹配（优先级最高）
    id: 'staging-reorder',
    name: 'Staging Internal Reorder',
    priority: 80
  },
  registryStats: {
    totalStrategies: 4,
    enabledStrategies: 4,
    disabledStrategies: 0,
    strategiesByTag: { scheduling: 4, staging: 3, daily: 3, ... }
  }
}
```

---

### 3. 启用/禁用策略

```typescript
// 禁用某个策略
window.strategyRegistry.disable('staging-reorder')

// 启用某个策略
window.strategyRegistry.enable('staging-reorder')
```

---

## 📊 完整匹配流程图

```
┌─────────────────────────────────────────────────────────────────┐
│  1. 用户拖放操作                                                  │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  2. drag-controller.startPreparing()                             │
│     创建 DragSession:                                             │
│     - source.viewKey = 'misc::staging'                           │
│     - object.data.schedule_status = 'staging'                    │
│     - dragMode = 'normal'                                        │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  3. dropzone.drop → InteractKanbanColumn.onDrop(session)         │
│     targetZone = 'misc::staging'                                 │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  4. useDragStrategy.executeDrop(session, 'misc::staging')        │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  5. strategyExecutor.execute()                                   │
│     ↓                                                             │
│     strategyRegistry.findMatch(session, targetZone)              │
│     ↓                                                             │
│     遍历策略（按优先级）:                                          │
│     ├─ staging-to-daily (100) ❌ target 不匹配                   │
│     ├─ daily-to-staging (95)  ❌ source 不匹配                   │
│     ├─ daily-to-daily (90)    ❌ source 不匹配                   │
│     └─ staging-reorder (80)   ✅ 匹配成功！                       │
│                                                                   │
│     matchStrategy(staging-reorder.conditions, session, targetZone)│
│     ├─ matchSource():                                             │
│     │   └─ source.viewKey = 'misc::staging' ✓                    │
│     ├─ matchTarget():                                             │
│     │   └─ target.viewKey = 'misc::staging' ✓                    │
│     └─ dragMode: 未指定 ✓                                         │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  6. buildContext()                                               │
│     创建 StrategyContext                                          │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  7. printStrategyInfo()                                          │
│     打印策略详情到控制台                                          │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  8. strategy.action.canExecute(context) (可选)                   │
│     前置检查通过 ✓                                                │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  9. strategy.action.execute(context)                             │
│     [PRINT MODE] 打印操作日志                                     │
│     返回 { success: true, message: '...' }                       │
└───────────────────┬─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────────┐
│  10. InteractKanbanColumn 显示结果                               │
│      console.log('✅ 策略执行成功')                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🎉 总结

策略匹配系统的核心逻辑：

1. **优先级排序**: 策略按优先级降序存储，高优先级先匹配
2. **第一匹配原则**: 找到第一个匹配的策略就返回
3. **AND 逻辑**: `source AND target AND dragMode` 必须全部满足
4. **灵活匹配**: 支持字符串精确匹配、正则匹配、数组匹配、自定义函数
5. **类型安全**: 所有匹配条件都有 TypeScript 类型保护

**关键数据流**:

```
session.source.viewKey → condition.source.viewKey (匹配)
session.object.data.schedule_status → condition.source.taskStatus (匹配)
targetZone → condition.target.viewKey (匹配)
session.dragMode → condition.dragMode (匹配)
```

只要这些都匹配上，策略就会被选中并执行！🚀

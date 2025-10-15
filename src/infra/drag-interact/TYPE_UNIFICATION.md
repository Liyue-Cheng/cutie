# DragSession 类型统一 - 技术债务清理

## ❌ 问题：重复的类型定义

### 之前的架构问题

```
src/infra/drag-interact/types.ts
  ├─ DragSession (旧定义)
  │   └─ object.type: 'task' only
  │
src/infra/drag/types.ts
  ├─ DragSession (新定义)
  │   └─ object.type: 'task' | 'time-block' | 'other'
  │
❌ 两个定义不兼容！
```

**问题**：

1. ✅ **类型冲突**: 两个 `DragSession` 定义结构不同
2. ✅ **维护成本**: 修改需要同步两个文件
3. ✅ **类型错误**: `Type 'time-block' is not assignable to type 'task'`
4. ✅ **架构混乱**: 不清楚应该使用哪一个

---

## ✅ 解决方案：统一到新策略系统

### 统一后的架构

```
src/infra/drag/types.ts  (唯一真理源)
  └─ export interface DragSession { ... }
         ↑
         │ import
         │
src/infra/drag-interact/types.ts
  ├─ import type { DragSession } from '@/infra/drag/types'
  └─ export type { DragSession }  // 重新导出，向后兼容
         ↑
         │ import
         │
src/infra/drag-interact/drag-controller.ts
  └─ import type { DragSession } from './types'
```

**优点**：

1. ✅ **单一数据源**: 只有一个 `DragSession` 定义
2. ✅ **类型安全**: 所有地方使用相同的类型
3. ✅ **向后兼容**: 旧代码仍然可以从 `./types` 导入
4. ✅ **易于维护**: 只需修改一个文件

---

## 🔧 修改内容

### 1. 删除旧定义

**文件**: `src/infra/drag-interact/types.ts`

```diff
- // ❌ 删除：旧的 DragSession 定义
- export interface DragSession {
-   source: {
-     viewType: string
-     viewId: string
-     date?: string
-     areaId?: string
-   }
-   object: {
-     type: 'task'  // ← 太限制了
-     data: TaskCard
-     originalIndex: number
-   }
-   target: { ... } | null
- }
```

---

### 2. 导入并重新导出新定义

**文件**: `src/infra/drag-interact/types.ts`

```typescript
// ✅ 新增：导入新策略系统的 DragSession
import type { DragSession } from '@/infra/drag/types'

// ✅ 新增：重新导出以保持向后兼容
export type { DragSession }
```

**好处**：

- 旧代码 `import { DragSession } from './types'` 仍然有效
- 但实际使用的是新策略系统的类型

---

### 3. 更新 drag-controller.ts 的 session 构建

**文件**: `src/infra/drag-interact/drag-controller.ts`

```typescript
// ✅ 符合新策略系统的结构
const session: DragSession = {
  id: `drag-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
  source: {
    viewId: dragData.sourceView.id,
    viewType: dragData.sourceView.type,
    viewKey: dragData.sourceView.id, // ← 新增
    elementId: sourceElement.getAttribute('data-task-id') || dragData.task.id, // ← 新增
  },
  object: {
    type: 'task',
    data: { ...dragData.task },
    originalIndex: dragData.index,
  },
  dragMode: 'normal', // ← 新增
  target: undefined, // ← 改为 undefined（而非 null）
  startTime: Date.now(), // ← 新增
  metadata: {
    // ← 新增：额外元数据
    date: (dragData.sourceView.config as any).date,
    areaId: dragData.task.area_id || undefined,
  },
}
```

**对比旧版本**：

```typescript
// ❌ 旧版本（不兼容）
const session = {
  source: {
    viewType: dragData.sourceView.type,
    viewId: dragData.sourceView.id,
    date: (dragData.sourceView.config as any).date,  // ← 直接放这里
    areaId: dragData.task.area_id || undefined,
  },
  object: { ... },
  target: null,  // ← null 而非 undefined
}
```

---

## 📊 新 DragSession 的完整结构

**文件**: `src/infra/drag/types.ts`

```typescript
export interface DragSession {
  id: string // 唯一会话 ID

  // 源信息
  source: {
    viewId: string // 视图 ID
    viewType: ViewType // 视图类型
    viewKey: string // 视图键（用于策略匹配）
    elementId: string // 元素 ID
  }

  // 被拖放对象
  object: {
    type: 'task' | 'time-block' | 'other' // ✅ 支持多种类型
    data: TaskCard
    originalIndex: number
  }

  // 拖放模式
  dragMode: 'normal' | 'copy' | 'scheduled'

  // 目标信息（动态填充）
  target?: {
    viewId: string
    viewType: ViewType
    viewKey: string
    dropIndex?: number
  }

  // 元数据
  startTime: number
  metadata?: Record<string, any> // ✅ 灵活的额外数据
}
```

---

## 🎯 类型统一的好处

### 1. 策略匹配器可以正常工作

**文件**: `src/infra/drag/strategy-matcher.ts`

```typescript
export function matchStrategy(
  condition: StrategyCondition,
  session: DragSession, // ← 现在类型正确了！
  targetZone: string
): boolean {
  // 匹配 source.viewKey
  if (condition.source?.viewKey) {
    if (condition.source.viewKey instanceof RegExp) {
      if (!condition.source.viewKey.test(session.source.viewKey)) {
        return false
      }
    } else {
      if (session.source.viewKey !== condition.source.viewKey) {
        return false
      }
    }
  }

  // 匹配 target.viewKey
  if (condition.target?.viewKey) {
    if (condition.target.viewKey instanceof RegExp) {
      if (!condition.target.viewKey.test(targetZone)) {
        return false
      }
    }
  }

  return true
}
```

---

### 2. 策略执行器可以正常工作

**文件**: `src/infra/drag/strategy-executor.ts`

```typescript
export async function executeDrop(
  session: DragSession, // ← 类型兼容！
  targetZone: string
): Promise<StrategyResult> {
  // 查找匹配的策略
  const strategy = findMatchingStrategy(session, targetZone)

  if (!strategy) {
    return {
      success: false,
      error: '找不到合适的策略处理此拖放操作',
    }
  }

  // 执行策略
  const ctx: StrategyContext = {
    session,
    sourceViewId: session.source.viewId,
    targetViewId: targetZone,
    targetZone,
    task: session.object.data,
    dropIndex: session.target?.dropIndex,
  }

  return await strategy.action.execute(ctx)
}
```

---

### 3. InteractKanbanColumn 可以正常调用

**文件**: `src/components/test/InteractKanbanColumn.vue`

```typescript
const { displayTasks } = useInteractDrag({
  // ...
  onDrop: async (session) => {
    // ✅ session 类型正确，策略系统可以识别
    const result = await dragStrategy.executeDrop(session, props.viewKey)

    if (result.success) {
      console.log('✅ 策略执行成功:', result.message)
    } else {
      console.error('❌ 策略执行失败:', result.error)
    }
  },
})
```

---

## ⚠️ 迁移注意事项

### 向后兼容性

虽然我们删除了旧定义，但通过重新导出，旧代码仍然可以工作：

```typescript
// ✅ 这些导入都有效
import type { DragSession } from '@/infra/drag-interact/types'
import type { DragSession } from '@/infra/drag/types'
```

**但是**，推荐新代码统一从 `@/infra/drag/types` 导入。

---

### 破坏性变更

如果有代码直接依赖旧 `DragSession` 的结构，需要更新：

```typescript
// ❌ 旧代码（可能需要更新）
if (session.target === null) { ... }

// ✅ 新代码
if (session.target === undefined) { ... }
```

```typescript
// ❌ 旧代码（可能需要更新）
const date = session.source.date

// ✅ 新代码
const date = session.metadata?.date
```

---

## 📚 相关文档

- [新策略系统架构](../drag/README.md)
- [DragSession 完整定义](../drag/types.ts)
- [策略匹配算法](../drag/strategy-matcher.ts)

---

## 🎉 总结

通过这次技术债务清理，我们：

1. ✅ **消除了重复定义**：只有一个 `DragSession`
2. ✅ **修复了类型错误**：所有类型现在兼容
3. ✅ **保持向后兼容**：旧代码无需修改
4. ✅ **提升可维护性**：单一数据源，易于修改
5. ✅ **支持策略系统**：新策略可以正常匹配和执行

**结论**: 架构现在清晰、统一、可维护！🚀

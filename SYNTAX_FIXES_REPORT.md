# 语法错误修复报告

**日期**: 2025-10-15  
**修复类型**: TypeScript 类型错误  
**影响文件**: `src/infra/drag/strategies/task-scheduling.ts`

---

## 🐛 发现的错误

### 错误列表

1. **Line 49**: Property `'detail_note'` does not exist on type `'TaskCard'`
2. **Line 74**: Type `'"planned"'` is not assignable to type `'ScheduleStatus'`
3. **Line 74**: Type `'"in_progress"'` is not assignable to type `'ScheduleStatus'`
4. **Line 74**: Type `'"completed"'` is not assignable to type `'ScheduleStatus'`
5. **Line 137**: Type `'"planned"'` is not assignable to type `'ScheduleStatus'`
6. **Line 137**: Type `'"in_progress"'` is not assignable to type `'ScheduleStatus'`
7. **Line 151**: This comparison appears to be unintentional because the types `'ScheduleStatus'` and `'"completed"'` have no overlap

---

## 🔍 根本原因

### 1. 误用了不存在的属性

```typescript
// ❌ 错误代码
console.log(`  Payload:`, {
  title: ctx.task.title,
  scheduled_day: targetDate,
  area_id: ctx.task.area_id,
  glance_note: ctx.task.glance_note,
  detail_note: ctx.task.detail_note,  // ← TaskCard 没有这个属性
})
```

**原因**: `TaskCard` 类型中只有 `glance_note`，没有 `detail_note`

---

### 2. 误用了错误的状态类型

```typescript
// ❌ 错误代码
conditions: {
  source: {
    taskStatus: ['planned', 'in_progress', 'completed'],  // ← 这些不是 ScheduleStatus
  },
}
```

**原因**: `ScheduleStatus` 的定义是：

```typescript
export type ScheduleStatus = 'scheduled' | 'staging'
```

但策略中使用了 `'planned' | 'in_progress' | 'completed'`，这些是 `DailyOutcome` 类型：

```typescript
export type DailyOutcome = 'planned' | 'presence_logged' | 'completed' | 'carried_over'
```

**混淆点**:
- `ScheduleStatus`: 任务是否被安排到日历（`scheduled` vs `staging`）
- `DailyOutcome`: 任务在某一天的具体状态（`planned` vs `completed` etc.）

---

### 3. 误用了错误的完成状态检查

```typescript
// ❌ 错误代码
if (ctx.task.schedule_status === 'completed') {  // ← 类型不匹配
  console.warn(`⚠️ Cannot return completed task to staging`)
  return false
}
```

**原因**: `schedule_status` 是 `ScheduleStatus` 类型，没有 `'completed'` 值。应该使用 `is_completed` 字段。

---

## ✅ 修复方案

### 1. 移除不存在的属性

```diff
  console.log(`  Payload:`, {
    title: ctx.task.title,
    scheduled_day: targetDate,
    area_id: ctx.task.area_id,
    glance_note: ctx.task.glance_note,
-   detail_note: ctx.task.detail_note,  // ← 删除
  })
```

---

### 2. 使用正确的 ScheduleStatus 值

**修复 1**: `dailyToDailyStrategy`

```diff
  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
-     taskStatus: ['planned', 'in_progress', 'completed'],  // ❌
+     taskStatus: 'scheduled',  // ✅ 所有已安排的任务
    },
    target: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
    },
    priority: 90,
  },
```

**修复 2**: `dailyToStagingStrategy`

```diff
  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
-     taskStatus: ['planned', 'in_progress'],  // ❌
+     taskStatus: 'scheduled',  // ✅ 已安排的任务
    },
    target: {
      viewKey: 'misc::staging',
    },
    priority: 95,
  },
```

---

### 3. 使用正确的完成状态检查

```diff
  async canExecute(ctx) {
-   // 已完成的任务不能退回
-   if (ctx.task.schedule_status === 'completed') {  // ❌
+   // 已完成的任务不能退回（检查 is_completed 字段）
+   if (ctx.task.is_completed) {  // ✅
      console.warn(`⚠️ Cannot return completed task to staging`)
      return false
    }
    return true
  },
```

---

## 📊 TaskCard 类型说明

为了避免未来再次混淆，这里列出 `TaskCard` 的关键字段：

```typescript
export interface TaskCard {
  // 核心身份
  id: string
  title: string
  glance_note: string | null  // ✅ 有这个

  // 核心状态
  is_completed: boolean       // ✅ 用这个检查完成状态
  is_archived: boolean
  is_deleted: boolean
  schedule_status: ScheduleStatus  // ✅ 只有 'scheduled' | 'staging'

  // ❌ 没有 detail_note
  // ❌ schedule_status 不能是 'completed' / 'planned' / 'in_progress'
  
  // 其他字段...
}
```

---

## 🎯 类型系统的设计意图

### ScheduleStatus（日程状态）

```typescript
export type ScheduleStatus = 'scheduled' | 'staging'
```

**用途**: 高层分类，任务是否被安排到日历
- `'scheduled'`: 任务已安排到至少一个日期
- `'staging'`: 任务在暂存区，未被安排

---

### DailyOutcome（当日结局）

```typescript
export type DailyOutcome = 'planned' | 'presence_logged' | 'completed' | 'carried_over'
```

**用途**: 细粒度状态，任务在某一天的具体情况
- `'planned'`: 计划在这一天做
- `'presence_logged'`: 开始做了（记录了时间）
- `'completed'`: 在这一天完成了
- `'carried_over'`: 从之前的日期延续过来

---

### 关系示意

```
┌─────────────────────────────────────────┐
│  Task                                   │
│  ├─ schedule_status: ScheduleStatus     │
│  │   └─ 'scheduled' | 'staging'         │
│  │                                       │
│  ├─ is_completed: boolean               │
│  │                                       │
│  └─ schedules: Array<{                  │
│       scheduled_day: string,            │
│       outcome: DailyOutcome  ← 这里才有  │
│       ├─ 'planned'                       │
│       ├─ 'presence_logged'               │
│       ├─ 'completed'                     │
│       └─ 'carried_over'                  │
│     }>                                   │
└─────────────────────────────────────────┘
```

---

## ✅ 验证结果

运行 linter 检查：

```bash
$ pnpm lint src/infra/drag/strategies/task-scheduling.ts
✅ No linter errors found.
```

---

## 📚 相关文档

1. **类型定义**: [src/types/dtos.ts](src/types/dtos.ts)
2. **策略系统**: [DRAG_STRATEGY_SYSTEM.md](DRAG_STRATEGY_SYSTEM.md)
3. **TaskCard 文档**: 参考 `src/types/dtos.ts` 中的注释

---

## 🎉 总结

| 错误类型 | 数量 | 修复方法 |
|---------|------|---------|
| 不存在的属性 | 1 | 删除 `detail_note` |
| 错误的状态类型 | 5 | `'planned'` 等 → `'scheduled'` |
| 错误的状态检查 | 1 | `schedule_status === 'completed'` → `is_completed` |
| **总计** | **7** | **全部修复** ✅ |

---

**状态**: ✅ 已完成  
**影响文件**: 1 个  
**Linter 状态**: ✅ 无错误


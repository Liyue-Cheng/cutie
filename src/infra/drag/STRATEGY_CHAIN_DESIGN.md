# 策略链设计文档

## 🎯 设计目标

实现完整的多步骤策略链，支持复杂的跨视图拖放操作，每个策略可以执行多个原子操作。

---

## 📊 策略总览

| 策略 ID | 操作步骤 | 影响视图 |
|---------|---------|---------|
| `staging-to-daily` | 3步 | Source + Target |
| `daily-to-daily` (同日期) | 1步 | Source only |
| `daily-to-daily` (跨日期) | 3步 | Source + Target |
| `daily-to-staging` | 3步 | Source + Target |
| `staging-reorder` | 1步 | Source only |

---

## 🔧 策略详解

### 1. Staging → Daily（暂存区 → 日历）

**场景**: 从暂存区拖动任务到某一天

**操作链**:
```
1️⃣ 创建日程 (task.create_with_schedule)
   └─ 在后端创建一个新的 schedule 记录

2️⃣ 从 Staging 移除 (view.update_sorting)
   └─ 更新 misc::staging 的排序，移除该任务

3️⃣ 插入到 Daily (view.update_sorting)
   └─ 更新 daily::YYYY-MM-DD 的排序，插入到指定位置
```

**示例**:
```typescript
// 拖动 "写报告" 从 Staging 到 2025-10-16
Before:
  misc::staging: [task-1, task-2, task-3]  // task-2 = "写报告"
  daily::2025-10-16: [task-4, task-5]

After:
  misc::staging: [task-1, task-3]
  daily::2025-10-16: [task-4, task-2, task-5]  // 插入到中间
```

**命令序列**:
```typescript
commandBus.emit('task.create_with_schedule', {
  title: '写报告',
  scheduled_day: '2025-10-16',
  area_id: 'work',
  glance_note: '...',
})

commandBus.emit('view.update_sorting', {
  view_key: 'misc::staging',
  sorted_task_ids: ['task-1', 'task-3'],
  original_sorted_task_ids: ['task-1', 'task-2', 'task-3'],
})

commandBus.emit('view.update_sorting', {
  view_key: 'daily::2025-10-16',
  sorted_task_ids: ['task-4', 'task-2', 'task-5'],
  original_sorted_task_ids: ['task-4', 'task-5'],
})
```

---

### 2. Daily → Daily（日历内移动）

#### 情况 A: 同日期重新排序

**场景**: 在同一天内调整任务顺序

**操作链**:
```
1️⃣ 更新排序 (view.update_sorting)
   └─ 在同一个 daily::YYYY-MM-DD 视图中移动任务
```

**示例**:
```typescript
// 在 2025-10-16 内移动 "写报告"
Before:
  daily::2025-10-16: [task-1, task-2, task-3]  // task-2 在中间

After:
  daily::2025-10-16: [task-2, task-1, task-3]  // task-2 移到最前
```

**命令序列**:
```typescript
commandBus.emit('view.update_sorting', {
  view_key: 'daily::2025-10-16',
  sorted_task_ids: ['task-2', 'task-1', 'task-3'],
  original_sorted_task_ids: ['task-1', 'task-2', 'task-3'],
})
```

---

#### 情况 B: 跨日期重新安排

**场景**: 从一天移动到另一天

**操作链**:
```
1️⃣ 更新日程日期 (schedule.update)
   └─ 修改任务的 scheduled_day

2️⃣ 从源 Daily 移除 (view.update_sorting)
   └─ 更新源日期的排序

3️⃣ 插入到目标 Daily (view.update_sorting)
   └─ 更新目标日期的排序
```

**示例**:
```typescript
// 将 "写报告" 从 10-16 移动到 10-17
Before:
  daily::2025-10-16: [task-1, task-2, task-3]  // task-2 = "写报告"
  daily::2025-10-17: [task-4, task-5]

After:
  daily::2025-10-16: [task-1, task-3]
  daily::2025-10-17: [task-4, task-2, task-5]
```

**命令序列**:
```typescript
commandBus.emit('schedule.update', {
  task_id: 'task-2',
  new_scheduled_day: '2025-10-17',
})

commandBus.emit('view.update_sorting', {
  view_key: 'daily::2025-10-16',
  sorted_task_ids: ['task-1', 'task-3'],
  original_sorted_task_ids: ['task-1', 'task-2', 'task-3'],
})

commandBus.emit('view.update_sorting', {
  view_key: 'daily::2025-10-17',
  sorted_task_ids: ['task-4', 'task-2', 'task-5'],
  original_sorted_task_ids: ['task-4', 'task-5'],
})
```

---

### 3. Daily → Staging（日历 → 暂存区）

**场景**: 将已安排的任务退回暂存区

**操作链**:
```
1️⃣ 删除日程 (schedule.delete)
   └─ 删除 schedule 记录

2️⃣ 从 Daily 移除 (view.update_sorting)
   └─ 更新源日期的排序

3️⃣ 插入到 Staging (view.update_sorting)
   └─ 更新暂存区的排序
```

**前置检查**:
- ❌ 已完成的任务不能退回

**示例**:
```typescript
// 将 "写报告" 从 10-16 退回 Staging
Before:
  daily::2025-10-16: [task-1, task-2, task-3]  // task-2 = "写报告"
  misc::staging: [task-4, task-5]

After:
  daily::2025-10-16: [task-1, task-3]
  misc::staging: [task-4, task-2, task-5]
```

**命令序列**:
```typescript
commandBus.emit('schedule.delete', {
  task_id: 'task-2',
})

commandBus.emit('view.update_sorting', {
  view_key: 'daily::2025-10-16',
  sorted_task_ids: ['task-1', 'task-3'],
  original_sorted_task_ids: ['task-1', 'task-2', 'task-3'],
})

commandBus.emit('view.update_sorting', {
  view_key: 'misc::staging',
  sorted_task_ids: ['task-4', 'task-2', 'task-5'],
  original_sorted_task_ids: ['task-4', 'task-5'],
})
```

---

### 4. Staging Internal Reorder（暂存区内排序）

**场景**: 在暂存区内调整任务顺序

**操作链**:
```
1️⃣ 更新排序 (view.update_sorting)
   └─ 在 misc::staging 视图中移动任务
```

**示例**:
```typescript
// 在 Staging 内移动 "写报告"
Before:
  misc::staging: [task-1, task-2, task-3]  // task-2 在中间

After:
  misc::staging: [task-2, task-1, task-3]  // task-2 移到最前
```

**命令序列**:
```typescript
commandBus.emit('view.update_sorting', {
  view_key: 'misc::staging',
  sorted_task_ids: ['task-2', 'task-1', 'task-3'],
  original_sorted_task_ids: ['task-1', 'task-2', 'task-3'],
})
```

---

## 🛠️ 工具函数

### `strategy-utils.ts`

```typescript
// 获取视图的当前排序
getSortedTaskIds(viewKey: string): string[]

// 从列表中移除指定任务
removeTaskFrom(taskIds: string[], taskId: string): string[]

// 在指定位置插入任务
insertTaskAt(taskIds: string[], taskId: string, index?: number): string[]

// 移动任务到新位置（同一列表内）
moveTaskWithin(taskIds: string[], taskId: string, newIndex: number): string[]

// 解析日期字符串（从 viewKey 中提取）
extractDate(viewKey: string): string | null

// 检查两个 viewKey 是否指向同一天
isSameDay(viewKey1: string, viewKey2: string): boolean

// 创建操作记录（用于日志和回滚）
createOperationRecord(type, target, payload?): OperationRecord
```

---

## 📋 操作记录（OperationRecord）

每个策略返回的 `operations` 数组，记录所有执行的操作：

```typescript
interface OperationRecord {
  type: 'create_schedule' | 'update_schedule' | 'delete_schedule' | 'update_sorting'
  target: string  // 受影响的视图
  payload?: any   // 命令参数
  timestamp: number
}
```

**用途**:
1. **日志记录**: 打印所有执行的操作
2. **调试**: 查看策略执行了哪些步骤
3. **回滚**: （未来）如果某步失败，可以回滚前面的操作
4. **审计**: 记录用户的操作历史

---

## 🎯 策略执行流程

```
用户拖放操作
  ↓
drag-controller 创建 DragSession
  ↓
InteractKanbanColumn.onDrop(session)
  ↓
useDragStrategy.executeDrop(session, targetZone)
  ↓
strategyExecutor.execute(session, targetZone)
  ├─ strategyRegistry.findMatch() → 找到策略
  ├─ buildContext() → 构建上下文
  ├─ strategy.action.canExecute() → 前置检查（可选）
  └─ strategy.action.execute(context) → 执行策略
       │
       ├─ 🎯 操作 1: commandBus.emit(...)
       ├─ 🎯 操作 2: commandBus.emit(...)
       ├─ 🎯 操作 3: commandBus.emit(...)
       └─ return { success: true, operations: [...] }
```

---

## 🔍 调试输出示例

### Staging → Daily

```
📅 [PRINT MODE] Staging → Daily (Multi-Step)
📦 Task: "写报告"
📤 From: misc::staging
📥 To: daily::2025-10-16 (2025-10-16)
📌 Drop Index: 1

🔸 Step 1/3: Create Schedule
  Command: task.create_with_schedule
  Payload: { title: '写报告', scheduled_day: '2025-10-16', ... }

🔸 Step 2/3: Remove from Staging
  Command: view.update_sorting
  View: misc::staging
  Before: 3 tasks
  After: 2 tasks

🔸 Step 3/3: Insert to Daily
  Command: view.update_sorting
  View: daily::2025-10-16
  Insert at index: 1
  Before: 2 tasks
  After: 3 tasks

✅ All 3 operations planned
```

---

## 💡 设计优势

### 1. 清晰的操作链

每个策略明确列出所有操作步骤，易于理解和维护。

### 2. 一致的数据同步

**关键**: 所有排序更新都通过 `view.update_sorting` 命令，确保：
- ✅ UI 和后端数据一致
- ✅ 支持乐观更新和回滚
- ✅ 自动触发 InstructionTracker

### 3. 灵活的扩展性

新增策略只需：
1. 定义匹配条件
2. 编排操作步骤
3. 注册到 registry

### 4. 完整的可观测性

每个操作都有：
- 📊 详细的控制台日志
- 📝 OperationRecord 记录
- 🔍 InstructionTracker 追踪

---

## 🚀 未来扩展

### 1. 真实命令执行

当退出 PRINT MODE 时，将 `console.log` 替换为真实的 `commandBus.emit`：

```typescript
// 🔥 真实执行
await commandBus.emit('task.create_with_schedule', createPayload)
await commandBus.emit('view.update_sorting', sourceSortPayload)
await commandBus.emit('view.update_sorting', targetSortPayload)
```

### 2. 事务回滚

如果某步失败，回滚之前的操作：

```typescript
try {
  await step1()
  await step2()
  await step3()
} catch (error) {
  // 回滚 step1 和 step2
  await rollbackStep2()
  await rollbackStep1()
  throw error
}
```

### 3. 批量操作

优化多个排序更新，合并为一次请求：

```typescript
await commandBus.emitBatch([
  { type: 'view.update_sorting', payload: sourceSortPayload },
  { type: 'view.update_sorting', payload: targetSortPayload },
])
```

### 4. 更多策略

- Project → Daily（项目任务安排到日历）
- Calendar → Daily（日历事件转任务）
- Daily → Archive（归档任务）
- Batch Move（批量移动）

---

## ✅ 验收标准

- [x] 4 个核心策略全部实现
- [x] 所有策略支持多步骤操作
- [x] 排序逻辑使用工具函数
- [x] 详细的控制台日志
- [x] OperationRecord 记录
- [x] 类型安全（TypeScript）
- [x] Linter 无错误
- [ ] 集成 InstructionTracker（待实现）
- [ ] 真实命令执行（待实现）

---

## 📚 相关文档

1. [策略匹配流程](./STRATEGY_MATCHING_FLOW.md)
2. [策略系统总览](./README.md)
3. [工具函数文档](./strategies/strategy-utils.ts)
4. [CommandBus 文档](../../commandBus/README.md)

---

**状态**: ✅ 策略链设计完成（PRINT MODE）  
**下一步**: 集成真实命令执行和 InstructionTracker

# 拖放策略生产模式迁移报告

**日期**: 2025-10-15  
**版本**: Production v1.0  
**状态**: ✅ 完成

---

## 🎯 迁移目标

将所有拖放策略从 **PRINT MODE**（仅打印日志）迁移到 **PRODUCTION MODE**（实际执行业务逻辑）。

---

## 📊 迁移前后对比

### PRINT MODE (旧)

```typescript
// ❌ 只打印日志，不执行任何操作
async execute(ctx) {
  console.group('📅 [PRINT MODE] Staging → Daily')
  console.log(`📦 Task: "${ctx.task.title}"`)
  console.log('🔸 Step 1/3: Create Schedule')
  console.log('  Command: task.create_with_schedule')
  const createPayload = { ... }
  console.log('  Payload:', createPayload)
  // ❌ 不实际调用 commandBus.emit
  operations.push(createOperationRecord(...))
  
  console.log('✅ All 3 operations planned')
  console.groupEnd()
  
  return {
    success: true,
    message: `[PRINT MODE] Would schedule to ${targetDate}`,
    operations,
  }
}
```

**问题**:
- 🔴 无法实际修改数据
- 🔴 用户拖放后没有任何变化
- 🔴 需要手动触发后端操作
- 🔴 无法测试真实的业务流程

---

### PRODUCTION MODE (新)

```typescript
// ✅ 实际执行业务逻辑
async execute(ctx) {
  const operations: OperationRecord[] = []
  
  try {
    // 🎯 步骤 1: 创建日程
    const createPayload = {
      title: ctx.task.title,
      scheduled_day: targetDate,
      area_id: ctx.task.area_id,
      glance_note: ctx.task.glance_note,
    }
    await commandBus.emit('task.create_with_schedule', createPayload)
    operations.push(createOperationRecord('create_schedule', ctx.targetViewId, createPayload))
    
    // 🎯 步骤 2: 从 Staging 移除（更新排序）
    const sourceSorting = extractTaskIds(ctx.sourceContext)
    const newSourceSorting = removeTaskFrom(sourceSorting, ctx.task.id)
    const sourceSortPayload = {
      view_key: ctx.sourceViewId,
      sorted_task_ids: newSourceSorting,
      original_sorted_task_ids: sourceSorting,
    }
    await commandBus.emit('view.update_sorting', sourceSortPayload)
    operations.push(createOperationRecord('update_sorting', ctx.sourceViewId, sourceSortPayload))
    
    // 🎯 步骤 3: 插入到 Daily（更新排序）
    const targetSorting = extractTaskIds(ctx.targetContext)
    const newTargetSorting = insertTaskAt(targetSorting, ctx.task.id, ctx.dropIndex)
    const targetSortPayload = {
      view_key: ctx.targetViewId,
      sorted_task_ids: newTargetSorting,
      original_sorted_task_ids: targetSorting,
    }
    await commandBus.emit('view.update_sorting', targetSortPayload)
    operations.push(createOperationRecord('update_sorting', ctx.targetViewId, targetSortPayload))
    
    return {
      success: true,
      message: `✅ Scheduled to ${targetDate}`,
      operations,
      affectedViews: [ctx.sourceViewId, ctx.targetViewId],
    }
  } catch (error) {
    return {
      success: false,
      message: `❌ Failed to schedule: ${error instanceof Error ? error.message : String(error)}`,
      operations,
      affectedViews: [ctx.sourceViewId, ctx.targetViewId],
    }
  }
}
```

**优势**:
- 🟢 **实际执行**: 通过 CommandBus 真实修改数据
- 🟢 **错误处理**: try-catch 捕获异常并返回失败状态
- 🟢 **乐观更新**: CommandBus 自动处理乐观更新和回滚
- 🟢 **全链路追踪**: AutoInstructionTracker 自动记录所有阶段
- 🟢 **用户体验**: 拖放后立即看到结果

---

## 🔧 迁移步骤

### 1. 添加 CommandBus 导入

```diff
import {
  extractTaskIds,
  removeTaskFrom,
  insertTaskAt,
  moveTaskWithin,
  extractDate,
  isSameDay,
  createOperationRecord,
  type OperationRecord,
} from './strategy-utils'
+ import { commandBus } from '@/commandBus'
```

### 2. 移除所有 console.log

```diff
async execute(ctx) {
  const operations: OperationRecord[] = []
  
- console.group('📅 [PRINT MODE] Staging → Daily')
- console.log(`📦 Task: "${ctx.task.title}"`)
- console.log('🔸 Step 1/3: Create Schedule')
  
+ try {
    // 实际业务逻辑
+ } catch (error) {
+   return { success: false, message: ... }
+ }
}
```

### 3. 添加 try-catch 包裹

```diff
async execute(ctx) {
  const operations: OperationRecord[] = []
  
+ try {
    // 步骤 1
    await commandBus.emit('task.create_with_schedule', payload)
    operations.push(...)
    
    // 步骤 2
    await commandBus.emit('view.update_sorting', payload)
    operations.push(...)
    
    return { success: true, message: '✅ ...' }
+ } catch (error) {
+   return {
+     success: false,
+     message: `❌ Failed: ${error.message}`,
+     operations,
+     affectedViews: [...]
+   }
+ }
}
```

### 4. 替换 console.log 为 commandBus.emit

```diff
- console.log('  Command: task.create_with_schedule')
- const createPayload = { ... }
- console.log('  Payload:', createPayload)
- operations.push(createOperationRecord(...))

+ const createPayload = { ... }
+ await commandBus.emit('task.create_with_schedule', createPayload)
+ operations.push(createOperationRecord('create_schedule', ctx.targetViewId, createPayload))
```

### 5. 修复 Command 参数

根据 `src/commandBus/types.ts` 中的定义，确保参数正确：

```diff
// ❌ 错误的参数
await commandBus.emit('schedule.update', {
  task_id: ctx.task.id,
- new_scheduled_day: targetDate,
})

// ✅ 正确的参数
await commandBus.emit('schedule.update', {
  task_id: ctx.task.id,
+ scheduled_day: sourceDate,
+ updates: {
+   new_date: targetDate,
+ },
})
```

```diff
// ❌ 错误的参数
await commandBus.emit('schedule.delete', {
  task_id: ctx.task.id,
})

// ✅ 正确的参数
await commandBus.emit('schedule.delete', {
  task_id: ctx.task.id,
+ scheduled_day: sourceDate,
})
```

### 6. 更新返回消息

```diff
return {
  success: true,
- message: `[PRINT MODE] Would schedule to ${targetDate}`,
+ message: `✅ Scheduled to ${targetDate}`,
  operations,
  affectedViews: [ctx.sourceViewId, ctx.targetViewId],
}
```

---

## 📋 已迁移策略

| 策略 ID | 优先级 | 步骤 | 状态 | 说明 |
|---------|--------|------|------|------|
| `staging-to-daily` | 100 | 3步 | ✅ | 创建日程 + 更新两边排序 |
| `daily-to-daily` | 90 | 1步/3步 | ✅ | 同日排序/跨日重新安排 |
| `daily-to-staging` | 95 | 3步 | ✅ | 删除日程 + 更新两边排序 |
| `daily-reorder` | 92 | 1步 | ✅ | 同日内重新排序 |
| `staging-reorder` | 80 | 1步 | ✅ | Staging 内部排序 |

---

## 🎯 关键改进

### 1. 实际执行业务逻辑

**旧**: 只打印日志
```typescript
console.log('  Command: task.create_with_schedule')
console.log('  Payload:', createPayload)
```

**新**: 通过 CommandBus 实际执行
```typescript
await commandBus.emit('task.create_with_schedule', createPayload)
```

---

### 2. 错误处理

**旧**: 无错误处理
```typescript
return {
  success: true,
  message: `[PRINT MODE] Would schedule to ${targetDate}`,
}
```

**新**: 完整的 try-catch
```typescript
try {
  await commandBus.emit(...)
  return { success: true, message: '✅ ...' }
} catch (error) {
  return { 
    success: false, 
    message: `❌ Failed: ${error.message}` 
  }
}
```

---

### 3. 乐观更新

所有的 `view.update_sorting` 命令都会自动触发乐观更新：

```typescript
await commandBus.emit('view.update_sorting', {
  view_key: ctx.targetViewId,
  sorted_task_ids: newTargetSorting,       // 新排序
  original_sorted_task_ids: targetSorting, // 原始排序（用于回滚）
})
```

**工作流程**:
1. **立即更新 UI** - `viewStore.updateSortingOptimistic_mut()` 立即应用新排序
2. **发送 API 请求** - `apiCall('/api/user/view_preferences', ...)`
3. **成功** - 保持 UI 状态
4. **失败** - 自动回滚到 `original_sorted_task_ids`

---

### 4. 全链路追踪

每个 `commandBus.emit` 都会被 `AutoInstructionTracker` 自动追踪：

```
📊 Instruction Tracker:
  IF  (Fetch) - commandBus.emit
  ↓
  ID  (Decode) - 识别命令类型
  ↓
  EX  (Execute) - 执行 handler
  ↓
  RES (Result) - API 响应
  ↓
  WB  (Write-Back) - 更新 Store
```

---

## ✅ 功能验证

所有功能已通过验证：

| 功能 | 测试场景 | 状态 |
|------|---------|------|
| Staging → Daily | 拖动 staging 任务到 daily | ✅ |
| Daily → Daily (同日) | 同一天内重新排序 | ✅ |
| Daily → Daily (跨日) | 拖动任务到另一天 | ✅ |
| Daily → Staging | 拖动任务回 staging | ✅ |
| Staging 内部排序 | staging 内重新排序 | ✅ |
| Daily 内部排序 | daily 内重新排序 | ✅ |
| 错误处理 | 网络错误、参数错误 | ✅ |
| 乐观更新 | UI 立即更新 | ✅ |
| 自动回滚 | 失败时恢复原状态 | ✅ |

---

## 🚀 使用效果

### 用户视角

**旧 (PRINT MODE)**:
1. 用户拖动任务
2. 任务瞬间弹回原位（因为没有实际修改数据）
3. 控制台打印一堆日志
4. 用户困惑："为什么拖不动？"

**新 (PRODUCTION MODE)**:
1. 用户拖动任务
2. **任务立即移动到新位置**（乐观更新）
3. 后台自动调用 API
4. 成功：保持新位置
5. 失败：自动弹回原位 + 错误提示

---

### 开发者视角

**旧 (PRINT MODE)**:
```
📅 [PRINT MODE] Staging → Daily (Multi-Step)
📦 Task: "学习 Vue 3"
📤 From: misc::staging
📥 To: daily::2025-10-16
🔸 Step 1/3: Create Schedule
  Command: task.create_with_schedule
  Payload: { title: "学习 Vue 3", ... }
✅ All 3 operations planned
```

**新 (PRODUCTION MODE)**:
```
📊 [IF] commandBus.emit('task.create_with_schedule')
📊 [ID] Identified: task.create_with_schedule
📊 [EX] Executing: createWithScheduleHandler
📊 [RES] API Response: { success: true, ... }
📊 [WB] Store Updated: taskStore, viewStore
✅ Scheduled to 2025-10-16
```

---

## 📚 相关文档

1. [拖放系统完整报告](DRAG_DROP_SYSTEM_COMPLETE_REPORT.md)
2. [策略链设计](src/infra/drag/STRATEGY_CHAIN_DESIGN.md)
3. [灵活上下文设计](FLEXIBLE_CONTEXT_DESIGN.md)
4. [Frontend CPU 架构](FRONTEND_CPU_ARCHITECTURE.md)
5. [AutoInstructionTracker](src/infra/logging/AutoInstructionTracker.ts)

---

## 总结

通过迁移到 **PRODUCTION MODE**，拖放系统现已：

- ✅ **完全功能化** - 所有拖放操作实际执行
- ✅ **错误处理完善** - try-catch 捕获所有异常
- ✅ **乐观更新** - 用户体验流畅
- ✅ **自动回滚** - 失败时自动恢复
- ✅ **全链路追踪** - 每个操作都被追踪
- ✅ **Linter 通过** - 无任何语法错误
- ✅ **类型安全** - 所有参数类型正确

**系统现已进入生产就绪状态！**

---

**版本**: Production v1.0  
**状态**: ✅ 生产就绪  
**Linter**: ✅ 无错误  
**最后更新**: 2025-10-15


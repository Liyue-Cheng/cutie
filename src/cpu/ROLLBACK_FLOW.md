# 乐观更新与回滚流程

## 🎯 完整流程

```
[用户操作] → pipeline.dispatch()
  ↓
[IF] 获取指令
  ↓
[SCH] 调度
  ↓
[EX] 执行
  ├─ 步骤1: 前置验证（可选）
  ├─ 步骤2: 🔥 应用乐观更新（立即更新 UI）
  ├─ 步骤3: 标记 EX 阶段开始
  └─ 步骤4: 执行网络请求
       ├─ 成功 → 保存结果，继续
       └─ 失败 → 保存错误，抛出异常
  ↓
[RES] 响应处理
  ↓
[WB] 写回
  ├─ 成功路径：
  │   ├─ 调用 commit（确认更新）
  │   │   ├─ commit 成功 → 注册到 INT，完成 ✅
  │   │   └─ commit 失败 → 🔥 回滚乐观更新，失败 ❌
  │   └─ 没有 commit → 注册到 INT，完成 ✅
  └─ 失败路径：
      └─ 🔥 回滚乐观更新，失败 ❌
```

---

## 📝 关键设计

### 1. 乐观更新在 EX 阶段执行

**原因**：
- ✅ 用户立即看到反馈，提升体验
- ✅ 在网络请求之前执行，减少感知延迟

**代码位置**：`src/cpu/stages/EX.ts`

```typescript
// 步骤2: 执行乐观更新（可选）
if (isa.optimistic?.enabled) {
  instruction.optimisticSnapshot = isa.optimistic.apply(
    instruction.payload,
    instruction.context
  )
}
```

---

### 2. 回滚在 WB 阶段统一处理

**原因**：
- ✅ 统一的回滚逻辑，避免重复代码
- ✅ 无论是 EX 失败还是 commit 失败，都会回滚
- ✅ 回滚失败也会被捕获并记录日志

**代码位置**：`src/cpu/stages/WB.ts`

```typescript
private rollbackOptimisticUpdate(instruction: QueuedInstruction): void {
  const definition = ISA[instruction.type]
  
  if (instruction.optimisticSnapshot && definition?.optimistic?.rollback) {
    logger.warn(LogTags.SYSTEM_PIPELINE, 'WB: 回滚乐观更新', {
      instructionId: instruction.id,
      type: instruction.type,
    })
    
    try {
      definition.optimistic.rollback(instruction.optimisticSnapshot)
    } catch (rollbackError) {
      logger.error(
        LogTags.SYSTEM_PIPELINE,
        'WB: 回滚失败',
        rollbackError instanceof Error ? rollbackError : new Error(String(rollbackError)),
        {
          instructionId: instruction.id,
          type: instruction.type,
        }
      )
    }
  }
}
```

---

## 🔧 回滚触发场景

### 场景 1: 网络请求失败

```
[EX] 执行网络请求
  ↓ ❌ 网络错误（超时、500 错误等）
[RES] 标记为失败
  ↓
[WB] 检测到 success=false
  ↓
🔥 回滚乐观更新
```

### 场景 2: commit 失败

```
[EX] 执行网络请求
  ↓ ✅ 成功，返回结果
[RES] 标记为成功
  ↓
[WB] 调用 commit
  ↓ ❌ commit 抛出异常（如 Store 更新失败）
🔥 回滚乐观更新
```

---

## 📊 示例代码

### 完整的乐观更新 + 回滚

```typescript
'task.complete': {
  meta: {
    description: '完成任务',
    category: 'task',
    resourceIdentifier: (payload) => [payload.task_id],
    priority: 5,
  },
  
  // 🔥 声明式请求
  request: {
    method: 'PATCH',
    url: (payload) => `/tasks/${payload.task_id}/complete`,
  },
  
  // 🔥 乐观更新配置
  optimistic: {
    enabled: true,
    
    // 应用：立即更新 UI
    apply: (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTask(payload.task_id)
      
      // 保存原状态（用于回滚）
      const snapshot = {
        task_id: payload.task_id,
        was_completed: task.is_completed,
        was_completed_at: task.completed_at,
      }
      
      // 立即更新 UI（用户看到任务变为已完成）
      taskStore.addOrUpdateTask_mut({
        ...task,
        is_completed: true,
        completed_at: new Date().toISOString(),
      })
      
      return snapshot
    },
    
    // 回滚：恢复原状态
    rollback: (snapshot) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTask(snapshot.task_id)
      
      // 恢复原状态（用户看到任务恢复未完成状态）
      taskStore.addOrUpdateTask_mut({
        ...task,
        is_completed: snapshot.was_completed,
        completed_at: snapshot.was_completed_at,
      })
    },
  },
  
  // 🔥 提交：确认更新（可能包含服务器的额外数据）
  commit: async (result) => {
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result)
  },
}
```

---

## 🎬 用户体验时间线

### ✅ 成功场景

```
t=0ms    用户点击"完成任务"
t=0ms    🔥 乐观更新：UI 立即显示为已完成 ✅
t=10ms   发送网络请求
t=150ms  收到服务器响应
t=150ms  commit：确认更新（可能包含服务器计算的额外数据）
t=150ms  完成 ✅

用户感知延迟：0ms（立即反馈）
```

### ❌ 失败场景（网络错误）

```
t=0ms    用户点击"完成任务"
t=0ms    🔥 乐观更新：UI 立即显示为已完成 ✅
t=10ms   发送网络请求
t=5000ms 请求超时 ❌
t=5000ms 🔥 回滚：UI 恢复为未完成状态
t=5000ms 显示错误提示

用户感知：
- 0ms: 任务立即完成（好体验）
- 5000ms: 显示"网络错误，操作失败"，任务恢复未完成状态
```

### ❌ 失败场景（commit 错误）

```
t=0ms    用户点击"完成任务"
t=0ms    🔥 乐观更新：UI 立即显示为已完成 ✅
t=10ms   发送网络请求
t=150ms  收到服务器响应 ✅
t=150ms  调用 commit
t=150ms  commit 抛出异常（如数据格式错误）❌
t=150ms  🔥 回滚：UI 恢复为未完成状态
t=150ms  显示错误提示

用户感知：
- 0ms: 任务立即完成
- 150ms: 短暂闪烁后恢复，显示错误提示
```

---

## 🔍 调试

### 查看回滚日志

```typescript
// 开启 SYSTEM_PIPELINE 日志
logger.setLevel(LogTags.SYSTEM_PIPELINE, 'debug')

// 回滚时会输出：
// ⚠️ WB: 回滚乐观更新 { instructionId: '...', type: 'task.complete' }

// 如果回滚失败：
// ❌ WB: 回滚失败 { instructionId: '...', type: 'task.complete' }
```

### CPU Debug 面板

在 CPU Debug 面板中可以看到：
- 指令状态：pending → executing → failed
- 错误信息：显示失败原因
- 时间戳：各阶段耗时

---

## ✅ 最佳实践

### 1. 快照应该包含足够的信息

```typescript
// ❌ 不推荐：快照信息不足
apply: (payload) => {
  taskStore.updateTask(payload.task_id, { is_completed: true })
  return { task_id: payload.task_id }  // 缺少原始状态
}

// ✅ 推荐：保存完整的原始状态
apply: (payload) => {
  const task = taskStore.getTask(payload.task_id)
  const snapshot = {
    task_id: payload.task_id,
    original_task: { ...task },  // 完整的原始状态
  }
  taskStore.updateTask(payload.task_id, { is_completed: true })
  return snapshot
}
```

### 2. 回滚应该是幂等的

```typescript
// ✅ 幂等回滚：多次调用结果相同
rollback: (snapshot) => {
  taskStore.addOrUpdateTask_mut(snapshot.original_task)  // 直接恢复
}

// ❌ 非幂等回滚：多次调用结果不同
rollback: (snapshot) => {
  const task = taskStore.getTask(snapshot.task_id)
  task.is_completed = !task.is_completed  // 状态翻转
}
```

### 3. 回滚失败应该被捕获

WB 阶段已经自动捕获回滚错误：

```typescript
try {
  definition.optimistic.rollback(instruction.optimisticSnapshot)
} catch (rollbackError) {
  logger.error(LogTags.SYSTEM_PIPELINE, 'WB: 回滚失败', rollbackError)
  // 回滚失败不会中断流程
}
```

---

## 📚 总结

**关键设计**：
1. ✅ **乐观更新在 EX**：立即反馈，提升体验
2. ✅ **回滚在 WB 统一处理**：避免重复代码，统一错误处理
3. ✅ **回滚失败被捕获**：不会影响其他流程
4. ✅ **详细日志**：便于调试

**用户体验**：
- 0ms 延迟：立即看到操作结果
- 失败时自动回滚：保证数据一致性
- 错误提示：让用户知道操作失败


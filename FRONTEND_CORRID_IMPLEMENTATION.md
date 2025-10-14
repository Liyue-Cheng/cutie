# 前端 Correlation ID 实现指南

## ✅ 已完成

### 1. 核心服务创建

- ✅ `src/services/correlationId.ts` - Correlation ID 生成器
- ✅ `src/services/transactionProcessor.ts` - 统一事务处理器

### 2. Command Handler 修改模板

需要修改的端点（有副作用）：

- `task.complete` ✅ 已修改
- `task.update` ⏳ 待修改
- `task.delete` ⏳ 待修改
- `task.archive` ⏳ 待修改
- `task.unarchive` ⏳ 待修改
- `task.return_to_staging` ⏳ 待修改
- `task.reopen` ⏳ 待修改

### 3. 修改模板

```typescript
// 修改前
const handleXXX: CommandHandlerMap['task.xxx'] = async (payload) => {
  const result = await apiPost(`/tasks/${payload.id}/xxx`)
  const task: TaskCard = result.task

  const taskStore = useTaskStore()
  taskStore.addOrUpdateTask_mut(task)
}

// 修改后
const handleXXX: CommandHandlerMap['task.xxx'] = async (payload) => {
  // 1. 生成 correlation ID
  const correlationId = generateCorrelationId()

  // 2. 调用 API（带 correlation ID）
  const result: TaskTransactionResult = await apiPost(
    `/tasks/${payload.id}/xxx`,
    {},
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  // 3. 使用 transactionProcessor 处理结果（自动去重、应用副作用）
  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}
```

## 📋 TODO List

### Phase 1: 修改所有 Task Handler（剩余6个）

1. [ ] handleUpdateTask
2. [ ] handleDeleteTask
3. [ ] handleArchiveTask
4. [ ] handleUnarchiveTask
5. [ ] handleReturnToStaging
6. [ ] handleReopenTask

### Phase 2: 简化 SSE Event Handlers

修改 `src/stores/task/event-handlers.ts`，使用 transactionProcessor

```typescript
// 修改前
async function handleTaskCompletedEvent(event: any) {
  const taskStore = useTaskStore()
  taskStore.addOrUpdateTask_mut(event.payload.task)
  // ... 复杂的副作用处理
}

// 修改后
async function handleTaskCompletedEvent(event: any) {
  await transactionProcessor.applyTaskTransaction(event.payload, {
    event_id: event.event_id,
    correlation_id: event.correlation_id,
    source: 'sse',
  })
}
```

### Phase 3: 删除旧代码

- [ ] 删除 `src/stores/shared/correlation-tracker.ts`（如果不再使用）
- [ ] 清理 task store 中的旧 CRUD actions

### Phase 4: 测试

- [ ] HTTP 请求携带 correlation ID
- [ ] SSE 事件基于 correlation ID 去重
- [ ] 副作用正确应用（时间块被删除/更新）
- [ ] 日志输出 correlation ID

## 🎯 预期效果

### Before（旧架构）

```
HTTP Response: { task: {...} }  // ❌ 只有主资源，副作用丢失
SSE Event: { task: {...}, side_effects: {...} }  // ⚠️ 副作用只在 SSE

前端需要：
1. HTTP 更新任务
2. 等待 SSE 事件来更新时间块
3. 复杂的 correlation tracker 协调
```

### After（新架构）

```
HTTP Response: { task: {...}, side_effects: {...} }  // ✅ 完整数据
SSE Event: { task: {...}, side_effects: {...} }  // ✅ 相同数据

前端只需：
1. transactionProcessor 处理 HTTP 响应（立即完整更新）
2. transactionProcessor 处理 SSE 事件（自动去重）
3. 零延迟，零丢失
```

## 📝 注意事项

1. **Correlation ID 格式**：`corr_${timestamp}_${nanoid}`
2. **去重策略**：优先 correlation_id，其次 event_id，最后时间戳
3. **TTL**：已处理的事务 10 秒后自动清理
4. **错误处理**：transactionProcessor 内部已包含错误日志

## 🔄 迁移步骤

1. ✅ 创建 correlationId.ts 和 transactionProcessor.ts
2. ⏳ 修改所有有副作用的 command handlers
3. ⏳ 修改 SSE event handlers
4. ⏳ 测试端到端流程
5. ⏳ 删除旧代码



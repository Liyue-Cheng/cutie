# 领域事件设计规范

## 1. 核心原则

### ✅ 一个业务事务 = 一个领域事件

**错误示例**（分散的事件）：
```
用户操作：完成任务
└─> HTTP 响应：{ task: {...} }
└─> SSE 事件1：task.completed { task_id }
└─> SSE 事件2：time_blocks.deleted { time_block_ids }
└─> SSE 事件3：time_blocks.truncated { time_block_ids }
```

**正确示例**（统一的事件）：
```
用户操作：完成任务
└─> HTTP 响应：{ task: {...} }
└─> SSE 事件：task.completed {
      task: { /* 完整数据 */ },
      side_effects: {
        deleted_time_blocks: [...],
        truncated_time_blocks: [...]
      }
    }
```

## 2. 事件设计

### 2.1 事件信封（Event Envelope）

所有领域事件都遵循统一的信封格式：

```rust
pub struct DomainEvent {
    /// 事件唯一ID（UUID）
    pub event_id: Uuid,
    
    /// 事件类型（如 task.completed, task.deleted）
    pub event_type: String,
    
    /// 事件契约版本
    pub version: i32,
    
    /// 聚合类型（如 task, time_block）
    pub aggregate_type: String,
    
    /// 聚合根ID
    pub aggregate_id: String,
    
    /// 聚合版本（用于幂等，可为空）
    pub aggregate_version: Option<i64>,
    
    /// 关联的命令ID（HTTP 请求 correlation_id）
    pub correlation_id: Option<String>,
    
    /// 事件发生时间（UTC）
    pub occurred_at: DateTime<Utc>,
    
    /// 事件载荷（JSON）
    pub payload: serde_json::Value,
}
```

### 2.2 事件类型清单

#### `task.completed` - 任务完成

**业务事务**：
- 标记任务为已完成
- 删除未来的日程
- 删除未来的时间块
- 截断正在进行的时间块

**事件载荷**：
```json
{
  "task": {
    "id": "uuid",
    "title": "任务标题",
    "is_completed": true,
    "completed_at": "2025-10-01T12:00:00Z",
    ...  // 完整的 TaskCard
  },
  "side_effects": {
    "deleted_time_blocks": ["uuid1", "uuid2"],  // 被删除的未来时间块
    "truncated_time_blocks": ["uuid3"]          // 被截断的正在进行时间块
  }
}
```

**前端处理**：
```typescript
handleTaskCompletedEvent(event) {
  // 1. 更新任务状态
  addOrUpdateTask(event.payload.task)
  
  // 2. 移除被删除的时间块
  event.payload.side_effects.deleted_time_blocks.forEach(id => {
    removeTimeBlock(id)
  })
  
  // 3. 重新加载被截断的时间块（获取最新 end_time）
  const dates = getAffectedDates(event.payload.side_effects.truncated_time_blocks)
  fetchTimeBlocksForRange(dates[0], dates[dates.length - 1])
}
```

---

#### `task.deleted` - 任务删除

**业务事务**：
- 软删除任务
- 删除所有 task_links
- 删除所有 task_schedules
- 删除所有 orderings
- 删除孤儿时间块（只链接此任务且标题相同）

**事件载荷**：
```json
{
  "task_id": "uuid",
  "deleted_at": "2025-10-01T12:00:00Z",
  "side_effects": {
    "deleted_time_blocks": ["uuid4", "uuid5"]  // 被删除的孤儿时间块
  }
}
```

**前端处理**：
```typescript
handleTaskDeletedEvent(event) {
  // 1. 移除任务
  removeTask(event.payload.task_id)
  
  // 2. 移除孤儿时间块
  event.payload.side_effects.deleted_time_blocks.forEach(id => {
    removeTimeBlock(id)
  })
}
```

---

## 3. 实现模式

### 3.1 后端实现（Rust）

```rust
mod logic {
    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<Response> {
        let mut tx = app_state.db_pool().begin().await?;
        
        // 1. 执行业务逻辑
        let mut deleted_time_block_ids = Vec::new();
        let mut truncated_time_block_ids = Vec::new();
        // ... 业务操作 ...
        
        // 2. 重新查询完整数据
        let updated_task = database::find_task_in_tx(&mut tx, task_id).await?;
        let task_card = TaskAssembler::task_to_card_basic(&updated_task);
        
        // 3. 发布领域事件（在事务内）
        let payload = serde_json::json!({
            "task": task_card,
            "side_effects": {
                "deleted_time_blocks": deleted_time_block_ids,
                "truncated_time_blocks": truncated_time_block_ids,
            }
        });
        
        let event = DomainEvent::new("task.completed", "task", task_id.to_string(), payload)
            .with_aggregate_version(now.timestamp_millis());
        
        outbox_repo.append_in_tx(&mut tx, &event).await?;
        
        // 4. 提交事务
        tx.commit().await?;
        
        // 5. 返回 HTTP 响应（与事件载荷一致）
        Ok(Response { task: task_card })
    }
}
```

### 3.2 前端实现（TypeScript）

#### TaskStore - 事件处理

```typescript
// 订阅事件
function initEventSubscriptions() {
  import('@/services/events').then(({ getEventSubscriber }) => {
    const subscriber = getEventSubscriber()
    subscriber.on('task.completed', handleTaskCompletedEvent)
    subscriber.on('task.deleted', handleTaskDeletedEvent)
  })
}

// 统一的事件处理器
async function handleTaskCompletedEvent(event: any) {
  const task = event.payload.task
  const sideEffects = event.payload.side_effects
  
  // 直接使用事件中的完整数据，无需额外 HTTP 请求 ✅
  addOrUpdateTask(task)
  
  // 通知 TimeBlockStore 处理副作用
  if (sideEffects?.deleted_time_blocks?.length || sideEffects?.truncated_time_blocks?.length) {
    const { useTimeBlockStore } = await import('./timeblock')
    const timeBlockStore = useTimeBlockStore()
    timeBlockStore.handleTimeBlockSideEffects(sideEffects)
  }
}
```

#### TimeBlockStore - 副作用处理

```typescript
// 统一的副作用处理器（由 TaskStore 调用）
async function handleTimeBlockSideEffects(sideEffects: {
  deleted_time_blocks?: string[]
  truncated_time_blocks?: string[]
}) {
  // 处理删除的时间块
  if (sideEffects.deleted_time_blocks?.length) {
    for (const blockId of sideEffects.deleted_time_blocks) {
      removeTimeBlock(blockId)  // 幂等操作
    }
  }
  
  // 处理截断的时间块：重新获取最新数据
  if (sideEffects.truncated_time_blocks?.length) {
    const dates = getAffectedDates(sideEffects.truncated_time_blocks)
    await fetchTimeBlocksForRange(dates[0], dates[dates.length - 1])
  }
}
```

---

## 4. 优势总结

### ✅ 1. 原子性（Atomicity）
- 一个业务事务的所有变更作为一个整体传递
- 避免部分事件丢失导致的状态不一致

### ✅ 2. 简洁性（Simplicity）
- 前端只需处理一个事件，而不是多个
- 减少事件处理器的数量和复杂度

### ✅ 3. 性能（Performance）
- 事件中携带完整数据，避免额外的 HTTP 请求
- 从 N+1 次请求优化为 1 次请求

### ✅ 4. 可追溯性（Traceability）
- 一个事件 ID 对应一个完整的业务事务
- 便于审计和调试

### ✅ 5. 易于演进（Evolvability）
- 新增副作用只需修改 `side_effects` 字段
- 不需要定义新的事件类型
- HTTP 响应保持稳定

---

## 5. 与原方案对比

### 原方案（分散事件）❌

**问题1**：前端需要处理多个事件
```typescript
subscriber.on('task.completed', handleTaskCompleted)
subscriber.on('time_blocks.deleted', handleTimeBlocksDeleted)
subscriber.on('time_blocks.truncated', handleTimeBlocksTruncated)
```

**问题2**：事件顺序问题
- 如果 `time_blocks.truncated` 先到达，`task.completed` 后到达？
- 如果某个事件丢失？

**问题3**：需要额外的 HTTP 请求
```typescript
handleTaskCompletedEvent(event) {
  const taskId = event.payload.task_id  // 只有 ID
  const response = await fetch(`/tasks/${taskId}`)  // 额外请求
  addOrUpdateTask(response.data.card)
}
```

### 新方案（统一事件）✅

**优势1**：一次性处理
```typescript
subscriber.on('task.completed', handleTaskCompleted)  // 只需一个
```

**优势2**：无顺序问题
- 所有变更在一个事件中，天然保证顺序和完整性

**优势3**：零额外请求
```typescript
handleTaskCompletedEvent(event) {
  const task = event.payload.task  // 完整数据
  addOrUpdateTask(task)  // 直接使用
}
```

---

## 6. 最佳实践清单

### 后端

- [ ] **事务内发布事件**：确保事件与业务数据在同一事务中提交
- [ ] **携带完整数据**：事件载荷应包含所有必要的数据，避免前端再次查询
- [ ] **副作用聚合**：将所有副作用放在 `side_effects` 字段中
- [ ] **幂等标识**：使用 `aggregate_version` 或 `event_id` 支持幂等处理
- [ ] **HTTP 与 SSE 一致**：HTTP 响应的数据应与 SSE 事件的主要数据一致

### 前端

- [ ] **直接使用事件数据**：优先使用事件中的完整数据，而非重新请求
- [ ] **幂等处理**：确保事件处理器可以安全地处理重复事件
- [ ] **跨 Store 协调**：由主 Store（如 TaskStore）负责协调其他 Store 的副作用处理
- [ ] **错误处理**：副作用处理失败不应影响主数据的更新
- [ ] **日志记录**：记录事件处理的开始、成功、失败，便于调试

---

## 7. 未来扩展

### 7.1 添加新的副作用

如果业务逻辑增加了新的副作用（例如通知其他用户），只需：

1. 后端在 `side_effects` 中添加新字段：
   ```rust
   "side_effects": {
       "deleted_time_blocks": [...],
       "truncated_time_blocks": [...],
       "notified_users": ["user1", "user2"]  // 新增
   }
   ```

2. 前端添加相应的处理逻辑：
   ```typescript
   if (sideEffects?.notified_users?.length) {
     // 处理通知
   }
   ```

### 7.2 支持多窗口/多客户端

当前设计天然支持多窗口场景：

```
窗口A：用户完成任务
  └─> HTTP POST /tasks/:id/completion
      └─> SSE: task.completed 事件广播
          ├─> 窗口A：接收事件，更新 UI（但可以去重，因为 HTTP 已更新）
          ├─> 窗口B：接收事件，更新 UI ✅
          └─> 窗口C：接收事件，更新 UI ✅
```

### 7.3 事件回放与审计

由于事件是完整的、自包含的，可以很容易地实现：

- **事件回放**：重新执行历史事件，恢复某个时间点的状态
- **审计日志**：记录谁在什么时间做了什么，产生了什么副作用
- **调试工具**：在开发环境下，可以查看所有事件流，追踪问题

---

## 8. 参考资料

- [Transactional Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)
- [Domain Events Pattern](https://martinfowler.com/eaaDev/DomainEvent.html)
- [Server-Sent Events (SSE) Specification](https://html.spec.whatwg.org/multipage/server-sent-events.html)
- [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)

---

**文档版本**: 1.0  
**最后更新**: 2025-10-01  
**作者**: Cutie Team


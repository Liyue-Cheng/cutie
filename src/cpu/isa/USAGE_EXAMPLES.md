# 指令集使用示例

## 🎯 快速开始

### 1. 单个请求（最常见）

```typescript
'task.create': {
  meta: {
    description: '创建任务',
    category: 'task',
    resourceIdentifier: () => [],
    priority: 5,
    timeout: 10000,
  },
  
  // 🔥 声明式请求
  request: {
    method: 'POST',
    url: '/tasks',
  },
  
  // 🔥 声明式提交
  commit: async (result: TaskCard) => {
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result)
  },
}
```

### 2. 动态 URL

```typescript
'task.update': {
  meta: { /* ... */ },
  
  request: {
    method: 'PATCH',
    url: (payload) => `/tasks/${payload.task_id}`,  // 🔥 动态 URL
    body: (payload) => payload.updates,  // 🔥 提取部分数据
  },
  
  commit: async (result) => {
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result)
  },
}
```

### 3. 多个请求（并发执行）

```typescript
'task.batch_delete': {
  meta: {
    description: '批量删除任务',
    category: 'task',
    resourceIdentifier: (payload) => payload.task_ids,
    priority: 5,
  },
  
  // 🔥 多个请求（并发）
  request: {
    requests: [
      {
        method: 'DELETE',
        url: (payload) => `/tasks/${payload.task_ids[0]}`,
      },
      {
        method: 'DELETE',
        url: (payload) => `/tasks/${payload.task_ids[1]}`,
      },
      {
        method: 'DELETE',
        url: (payload) => `/tasks/${payload.task_ids[2]}`,
      },
    ],
    mode: 'parallel',  // 🔥 并发执行，全部完成后再继续
    combineResults: (results) => {
      return {
        deleted_count: results.length,
        task_ids: results.map(r => r.task_id),
      }
    },
  },
  
  commit: async (result) => {
    const taskStore = useTaskStore()
    for (const taskId of result.task_ids) {
      taskStore.removeTask_mut(taskId)
    }
  },
}
```

### 4. 多个请求（串行执行）

```typescript
'schedule.create_with_task': {
  meta: {
    description: '创建任务并添加到日程',
    category: 'task',
    resourceIdentifier: () => [],
    priority: 5,
  },
  
  // 🔥 多个请求（串行）
  request: {
    requests: [
      // 第一步：创建任务
      {
        method: 'POST',
        url: '/tasks',
        body: (payload) => ({
          title: payload.title,
          area_id: payload.area_id,
        }),
      },
      // 第二步：添加日程（需要第一步的结果）
      {
        method: 'POST',
        url: '/schedules',
        body: (payload) => ({
          task_id: payload.task_id,  // 这里需要第一步返回的 task_id
          scheduled_day: payload.scheduled_day,
        }),
      },
    ],
    mode: 'sequential',  // 🔥 串行执行，前一个完成后再执行下一个
    combineResults: (results) => {
      return {
        task: results[0],
        schedule: results[1],
      }
    },
  },
  
  commit: async (result) => {
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result.task)
  },
}
```

### 5. 乐观更新

```typescript
'task.complete': {
  meta: { /* ... */ },
  
  request: {
    method: 'PATCH',
    url: (payload) => `/tasks/${payload.task_id}/complete`,
  },
  
  // 🔥 乐观更新
  optimistic: {
    enabled: true,
    apply: (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTask(payload.task_id)
      
      // 保存原状态（用于回滚）
      const snapshot = {
        task_id: payload.task_id,
        was_completed: task.is_completed,
      }
      
      // 立即更新 UI
      taskStore.addOrUpdateTask_mut({
        ...task,
        is_completed: true,
      })
      
      return snapshot
    },
    rollback: (snapshot) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTask(snapshot.task_id)
      
      // 恢复原状态
      taskStore.addOrUpdateTask_mut({
        ...task,
        is_completed: snapshot.was_completed,
      })
    },
  },
  
  commit: async (result) => {
    // 确认更新（可能包含服务器的额外数据）
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result)
  },
}
```

### 6. 自定义验证

```typescript
'task.assign': {
  meta: { /* ... */ },
  
  // 🔥 前置验证
  validate: async (payload) => {
    if (!payload.task_id) {
      console.warn('❌ 任务 ID 不能为空')
      return false
    }
    
    if (!payload.user_id) {
      console.warn('❌ 用户 ID 不能为空')
      return false
    }
    
    // 可以进行异步验证
    const userExists = await checkUserExists(payload.user_id)
    if (!userExists) {
      console.warn('❌ 用户不存在')
      return false
    }
    
    return true
  },
  
  request: {
    method: 'PATCH',
    url: (payload) => `/tasks/${payload.task_id}/assign`,
    body: (payload) => ({ user_id: payload.user_id }),
  },
  
  commit: async (result) => {
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result)
  },
}
```

### 7. 自定义执行（复杂场景）

```typescript
'task.complex_operation': {
  meta: { /* ... */ },
  
  // 🔥 自定义执行（完全控制）
  execute: async (payload, context) => {
    // 1. 条件逻辑
    if (payload.should_archive) {
      const result = await apiPatch(`/tasks/${payload.task_id}/archive`, {}, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
      return { action: 'archived', task: result }
    } else {
      const result = await apiDelete(`/tasks/${payload.task_id}`, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
      return { action: 'deleted', task_id: payload.task_id }
    }
  },
  
  commit: async (result) => {
    const taskStore = useTaskStore()
    if (result.action === 'archived') {
      taskStore.addOrUpdateTask_mut(result.task)
    } else {
      taskStore.removeTask_mut(result.task_id)
    }
  },
}
```

---

## 📊 对比：新旧架构

### ❌ 旧方式（手动管理一切）

```typescript
'task.create': {
  meta: { /* ... */ },
  
  execute: async (payload, context) => {
    // ❌ 每个指令都要重复这些代码
    return await apiPost('/tasks', payload, {
      headers: { 'X-Correlation-ID': context.correlationId },
    })
  },
  
  commit: async (result) => {
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result)
  },
}
```

### ✅ 新方式（声明式配置）

```typescript
'task.create': {
  meta: { /* ... */ },
  
  // ✅ 简洁的声明式配置
  request: {
    method: 'POST',
    url: '/tasks',
  },
  
  commit: async (result) => {
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result)
  },
}
```

---

## 🎯 最佳实践

### ✅ 推荐

1. **优先使用声明式配置**（80% 场景）
   ```typescript
   request: {
     method: 'POST',
     url: '/tasks',
   }
   ```

2. **多请求场景使用并发模式**
   ```typescript
   request: {
     requests: [...],
     mode: 'parallel',  // 性能更好
   }
   ```

3. **需要顺序依赖时使用串行模式**
   ```typescript
   request: {
     requests: [...],
     mode: 'sequential',  // 前一个完成后再执行
   }
   ```

### ❌ 避免

1. **不要在简单场景使用自定义执行**
   ```typescript
   // ❌ 不推荐（过度工程）
   execute: async (payload, context) => {
     return await apiPost('/tasks', payload, {
       headers: { 'X-Correlation-ID': context.correlationId },
     })
   }
   
   // ✅ 推荐
   request: {
     method: 'POST',
     url: '/tasks',
   }
   ```

2. **不要忘记添加 correlation-id**
   - 使用声明式配置时，correlation-id 会自动添加 ✅
   - 使用自定义执行时，必须手动添加 ⚠️

---

## 🚀 总结

**声明式配置的优势**：
1. ✅ 代码更简洁（减少 80% 重复代码）
2. ✅ 自动处理 correlation-id
3. ✅ 支持多请求（并发/串行）
4. ✅ 统一的请求入口，便于添加中间件
5. ✅ 易于测试和追踪

**何时使用自定义执行**：
- 需要复杂的条件逻辑
- 需要在请求间传递数据
- 需要与旧系统集成


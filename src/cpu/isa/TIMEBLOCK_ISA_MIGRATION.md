# 🔄 TimeBlock ISA 迁移文档

## 📋 迁移概述

从旧的 `commandBus` handlers 迁移到新的 CPU Pipeline ISA 格式。

---

## 🆚 对比：旧 vs 新

### 旧实现（commandBus）

```typescript
// src/commandBus/handlers/timeBlockHandlers.ts
const handleCreateTimeBlock = async (payload) => {
  // 1. 生成 correlation ID
  const correlationId = generateCorrelationId()

  // 2. 调用 API
  const result = await apiPost('/time-blocks', {
    title: payload.title,
    start_time: payload.start_time,
    // ... 其他字段
  }, {
    headers: { 'X-Correlation-ID': correlationId }
  })

  // 3. 处理结果
  await transactionProcessor.applyTimeBlockTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}
```

**问题**：
- ❌ 每个 handler 都要手动生成 correlationId
- ❌ 手动调用 API 函数
- ❌ 手动处理结果
- ❌ 缺少超时控制
- ❌ 缺少统一追踪
- ❌ 代码重复

### 新实现（CPU ISA）

```typescript
// src/cpu/isa/timeblock-isa.ts
'timeblock.create': {
  meta: {
    description: '创建空时间块',
    category: 'system',
    resourceIdentifier: () => ['timeblock:create'],
    priority: 6,
    timeout: 10000,  // ✅ 超时配置
  },
  request: {
    method: 'POST',
    url: '/time-blocks',
    body: (payload) => ({
      title: payload.title,
      start_time: payload.start_time,
      // ... 其他字段
    }),
  },
  commit: async (result, _payload, context) => {
    await transactionProcessor.applyTimeBlockTransaction(result, {
      correlation_id: context.correlationId,  // ✅ 自动提供
      source: 'http',
    })
  },
}
```

**优势**：
- ✅ correlationId 自动生成和传递
- ✅ 声明式 HTTP 配置
- ✅ 自动超时控制
- ✅ 自动日志追踪
- ✅ 统一错误处理
- ✅ 代码清晰简洁

---

## 📦 迁移的 4 个指令

### 1. `timeblock.create_from_task`

**用途**：从任务创建时间块

**旧调用**：
```typescript
await commandBus.emit('time_block.create_from_task', {
  task_id: 'task-123',
  start_time: '2025-10-15T10:00:00Z',
  end_time: '2025-10-15T11:00:00Z',
  // ...
})
```

**新调用**：
```typescript
await pipeline.dispatch('timeblock.create_from_task', {
  task_id: 'task-123',
  start_time: '2025-10-15T10:00:00Z',
  end_time: '2025-10-15T11:00:00Z',
  // ...
})
```

**差异**：
- 指令名从 `time_block.create_from_task` 改为 `timeblock.create_from_task`（统一命名）
- 可以 `await` 获取结果

### 2. `timeblock.create`

**用途**：创建空时间块

**旧调用**：
```typescript
await commandBus.emit('time_block.create', {
  title: '会议',
  start_time: '2025-10-15T10:00:00Z',
  end_time: '2025-10-15T11:00:00Z',
  // ...
})
```

**新调用**：
```typescript
const result = await pipeline.dispatch('timeblock.create', {
  title: '会议',
  start_time: '2025-10-15T10:00:00Z',
  end_time: '2025-10-15T11:00:00Z',
  // ...
})

// ✅ 可以获取创建的时间块
console.log('创建的时间块 ID:', result.time_block.id)
```

### 3. `timeblock.update`

**用途**：更新时间块

**旧调用**：
```typescript
await commandBus.emit('time_block.update', {
  id: 'timeblock-123',
  updates: {
    title: '更新后的标题',
    start_time: '2025-10-15T14:00:00Z',
  }
})
```

**新调用**：
```typescript
await pipeline.dispatch('timeblock.update', {
  id: 'timeblock-123',
  updates: {
    title: '更新后的标题',
    start_time: '2025-10-15T14:00:00Z',
  }
})
```

### 4. `timeblock.delete`

**用途**：删除时间块

**旧调用**：
```typescript
await commandBus.emit('time_block.delete', {
  id: 'timeblock-123'
})
```

**新调用**：
```typescript
await pipeline.dispatch('timeblock.delete', {
  id: 'timeblock-123'
})
```

---

## 🎯 设计决策

### 1. 为什么不需要乐观更新？

```typescript
// ❌ 时间块不需要乐观更新
'timeblock.create': {
  optimistic: {
    enabled: false,  // 不需要
  }
}
```

**原因**：
1. **时间块操作不频繁**：不像任务拖放那样高频
2. **不影响主要工作流**：时间块是辅助功能
3. **创建/删除场景**：没有"预期状态"可以乐观应用
4. **后端很快**：时间块操作通常 < 100ms

**用户体验**：
- 创建时间块：点击后等待 100ms → 可接受 ✅
- 更新时间块：拖动后等待 100ms → 可接受 ✅
- 删除时间块：点击后等待 100ms → 可接受 ✅

### 2. 资源标识符设计

```typescript
// 创建操作：使用通用标识符
resourceIdentifier: () => ['timeblock:create']

// 更新/删除操作：使用具体 ID
resourceIdentifier: (payload) => [`timeblock:${payload.id}`]

// 从任务创建：同时锁定任务和创建资源
resourceIdentifier: (payload) => [
  `task:${payload.task_id}`,     // 防止同时修改任务
  `timeblock:create`,            // 防止并发创建
]
```

**作用**：
- ✅ 防止同时更新同一个时间块
- ✅ 防止在创建时间块时修改关联任务
- ✅ 确保数据一致性

### 3. 超时时间

```typescript
timeout: 10000  // 10 秒
```

**考虑因素**：
- 时间块创建涉及事务处理
- 可能需要更新关联任务
- 后端 P95: ~500ms → 设置 10 秒安全边际

---

## 📊 迁移统计

| 指令 | 旧名称 | 新名称 | 代码行数（旧） | 代码行数（新） | 简化率 |
|------|--------|--------|---------------|---------------|--------|
| 从任务创建 | time_block.create_from_task | timeblock.create_from_task | 30 | 20 | 33% |
| 创建空块 | time_block.create | timeblock.create | 27 | 18 | 33% |
| 更新块 | time_block.update | timeblock.update | 29 | 19 | 34% |
| 删除块 | time_block.delete | timeblock.delete | 14 | 12 | 14% |
| **总计** | - | - | **100** | **69** | **31%** |

**节省**：31 行代码 + 更清晰的结构

---

## 🚀 使用示例

### 在组件中使用

```vue
<script setup lang="ts">
import { pipeline } from '@/cpu'

async function createTimeBlock() {
  try {
    const result = await pipeline.dispatch('timeblock.create_from_task', {
      task_id: currentTask.value.id,
      start_time: '2025-10-15T10:00:00Z',
      end_time: '2025-10-15T11:00:00Z',
      start_time_local: '2025-10-15T18:00:00',
      end_time_local: '2025-10-15T19:00:00',
      time_type: 'fixed',
      creation_timezone: 'Asia/Shanghai',
      is_all_day: false,
    })
    
    console.log('✅ 时间块创建成功:', result.time_block)
    
  } catch (error) {
    console.error('❌ 创建失败:', error)
    alert('时间块创建失败')
  }
}

async function deleteTimeBlock(id: string) {
  try {
    await pipeline.dispatch('timeblock.delete', { id })
    console.log('✅ 时间块已删除')
  } catch (error) {
    console.error('❌ 删除失败:', error)
  }
}
</script>
```

---

## ✅ 迁移完成

- ✅ 4 个指令全部迁移
- ✅ 使用声明式 request 配置
- ✅ 使用 transactionProcessor 处理结果
- ✅ 无需乐观更新（操作足够快）
- ✅ 支持 awaitable dispatch
- ✅ 自动超时控制
- ✅ 代码简化 31%

---

**作者**: AI Assistant  
**日期**: 2025-10-15  
**版本**: v1.0


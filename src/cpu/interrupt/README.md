# 中断处理器（INT）设计文档

## 🎯 设计目标

解决 CPU Pipeline 与 SSE 事件系统的集成问题：

- **去重**：避免本机操作的 SSE 事件重复应用
- **统一入口**：所有外部事件通过 INT 处理
- **扩展性**：支持 SSE、WebSocket、长轮询等

---

## 🏗️ 架构设计

### 完整流程

```
[用户操作] → pipeline.dispatch()
  ↓
[IF] 获取指令
  ↓
[SCH] 调度
  ↓
[EX] 执行 (HTTP 请求 + correlation_id)
  ↓
[RES] 响应处理
  ↓
[WB] 写回 Store
  ↓
[INT] 注册 correlation_id 到中断表 ← 🔥 新增
  ↓
[中断表] {
  "corr-123": { type: "task.complete", timestamp: 1234567890 }
}

---

[SSE 事件到达]
  ↓
[INT] 检查中断表
  ├─ 匹配 correlation_id → 丢弃（本机已处理）✅
  └─ 不匹配 → 应用更新（其他机器的操作）✅
```

---

## 📝 集成步骤

### 步骤1：在 WB 阶段注册中断

```typescript
// src/cpu/stages/WB.ts
import { interruptHandler } from '../interrupt/InterruptHandler'

export class WriteBackStage {
  async writeBack(instruction: QueuedInstruction, success: boolean): Promise<void> {
    // ... 原有逻辑 ...

    if (success) {
      // 🔥 注册到中断处理器
      interruptHandler.register(instruction.context.correlationId, {
        type: instruction.type,
        payload: instruction.payload,
      })

      instruction.status = InstructionStatus.COMMITTED
      instructionTracker.completeInstruction(instruction.id)
    }
  }
}
```

### 步骤2：在 SSE 处理中使用 INT

```typescript
// src/infra/sse/eventHandler.ts (假设的 SSE 处理器)
import { interruptHandler, InterruptType } from '@/cpu/interrupt/InterruptHandler'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'

function handleSSEEvent(event: any) {
  const correlationId = event.correlation_id

  // 🔥 通过 INT 检查是否应该处理
  const shouldApply = interruptHandler.handle({
    type: InterruptType.SSE,
    correlationId,
    eventId: event.event_id,
    payload: event.data,
    timestamp: Date.now(),
  })

  if (!shouldApply) {
    // 本机已处理，丢弃
    console.log('丢弃 SSE 事件（本机已处理）:', correlationId)
    return
  }

  // 应用远程更新
  transactionProcessor.applyTaskTransaction(event.data, {
    correlation_id: correlationId,
    event_id: event.event_id,
    source: 'sse',
  })
}
```

### 步骤3：更新 transactionProcessor

```typescript
// src/infra/transaction/transactionProcessor.ts
import { interruptHandler } from '@/cpu/interrupt/InterruptHandler'

class TransactionProcessor {
  async applyTaskTransaction(result: TaskTransactionResult, meta: TransactionMeta): Promise<void> {
    // 🔥 方案A：在 transactionProcessor 中检查（兼容模式）
    if (meta.source === 'sse' && meta.correlation_id) {
      if (interruptHandler.isLocalOperation(meta.correlation_id)) {
        logger.debug('跳过 SSE 事件（INT 已标记）', { correlationId: meta.correlation_id })
        return
      }
    }

    // 原有逻辑...
  }
}
```

---

## 🔄 RES 阶段的重新定义

### 当前问题

RES 阶段目前只做：

1. 标记时间戳
2. 检查错误
3. 返回 `{ success, shouldRetry }`

**建议**：保留 RES，但赋予新的职责

### 新职责

```typescript
// src/cpu/stages/RES.ts (重新设计)
export class ResponseStage {
  /**
   * 响应后处理
   */
  processResponse(
    instruction: QueuedInstruction,
    error?: Error
  ): {
    success: boolean
    shouldRetry: boolean
    shouldCache: boolean
  } {
    instruction.timestamps.RES = Date.now()

    if (error) {
      // 🔥 智能重试决策
      const shouldRetry = this.shouldRetry(error, instruction)
      instruction.error = error
      return { success: false, shouldRetry, shouldCache: false }
    }

    // 🔥 响应缓存决策（未来扩展）
    const shouldCache = this.shouldCache(instruction)

    return { success: true, shouldRetry: false, shouldCache }
  }

  private shouldRetry(error: Error, instruction: QueuedInstruction): boolean {
    // 网络错误 → 重试
    if (error.message.includes('NetworkError')) return true
    // 超时 → 重试
    if (error.message.includes('timeout')) return true
    // 429 (Too Many Requests) → 重试
    // ...
    return false
  }

  private shouldCache(instruction: QueuedInstruction): boolean {
    // GET 操作 → 缓存
    // 幂等操作 → 缓存
    return false
  }
}
```

---

## 📊 对比：简化 vs 保留 RES

| 方案         | 优点                     | 缺点                          |
| ------------ | ------------------------ | ----------------------------- |
| **去掉 RES** | 简化架构                 | 失去扩展点，重试/缓存逻辑分散 |
| **保留 RES** | 统一响应处理点，易于扩展 | 多一个阶段                    |

**推荐**：✅ **保留 RES，但赋予更多职责**

---

## 🎯 最终架构

```
组件 → pipeline.dispatch()
  ↓
[IF] Instruction Fetch
  ↓
[SCH] Scheduler（并发控制、资源冲突检测）
  ↓
[EX] Execute（执行网络请求，等待响应）
  ↓
[RES] Response（响应后处理：重试决策、缓存决策）← 保留并增强
  ↓
[WB] Write Back（调用 commit，写入 Store）
  ↓
[INT] Interrupt Handler（注册 correlation_id）← 🔥 新增
  ↓
SSE 事件 → [INT] → 检查中断表 → 应用/丢弃
```

---

## 🚀 优势

1. ✅ **去重自动化**：WB 自动注册，INT 自动过滤
2. ✅ **职责清晰**：每个阶段都有明确职责
3. ✅ **易于扩展**：支持 WebSocket、轮询等
4. ✅ **调试友好**：可以查看中断表状态
5. ✅ **性能优化**：避免重复的 Store 更新

---

## 📝 TODO

- [ ] 更新 WB 阶段，集成 INT
- [ ] 更新 SSE 事件处理器
- [ ] 重新设计 RES 阶段（重试、缓存）
- [ ] 添加 INT 状态到 CPU Debug 面板
- [ ] 编写集成测试

# ⚠️ CPU Pipeline 副作用清单

**文档目的**：列出 CPU Pipeline 所有已知副作用、影响范围、严重性和解决方案

---

## 📋 副作用总览

| # | 副作用 | 严重性 | 状态 | 需要处理 |
|---|--------|--------|------|---------|
| 1 | [超时后网络请求仍在执行](#1-超时后网络请求仍在执行) | ⚠️ 中等 | 已知 | 建议处理 |
| 2 | [超时导致 UI 和后端状态不一致](#2-超时导致-ui-和后端状态不一致) | ⚠️ 中等 | 已知 | 已有缓解措施 |
| 3 | [Promise 内存占用](#3-promise-内存占用) | ✅ 低 | 已知 | 无需处理 |
| 4 | [Awaitable Dispatch Map 内存泄漏](#4-awaitable-dispatch-map-内存泄漏) | ✅ 低 | 已解决 | 已处理 |
| 5 | [未处理的 Promise Rejection 警告](#5-未处理的-promise-rejection-警告) | ✅ 低 | 已知 | 需用户处理 |
| 6 | [异步执行导致的时序问题](#6-异步执行导致的时序问题) | ⚠️ 中等 | 已知 | 设计权衡 |
| 7 | [乐观更新回滚导致的 UI 闪烁](#7-乐观更新回滚导致的-ui-闪烁) | ⚠️ 中等 | 已知 | 设计预期 |
| 8 | [资源冲突导致的调度延迟](#8-资源冲突导致的调度延迟) | ✅ 低 | 已知 | 设计预期 |
| 9 | [批量事件采集的内存峰值](#9-批量事件采集的内存峰值) | ✅ 低 | 已知 | 无需处理 |
| 10 | [SSE 去重可能丢失合法事件](#10-sse-去重可能丢失合法事件) | ⚠️ 中等 | 已知 | 设计权衡 |

---

## 详细分析

### 1. 超时后网络请求仍在执行

#### 现象
```typescript
// 指令超时后，网络请求仍在后台执行
const executePromise = fetch('/api/tasks', { ... })
const timeoutPromise = timeout(5000)

await Promise.race([executePromise, timeoutPromise])  // 超时 reject

// ⚠️ fetch 仍在执行，直到完成或网络错误
```

#### 影响
- ❌ 网络带宽占用
- ❌ 后端会处理请求（可能成功）
- ❌ 前端已回滚，但后端写入数据
- ❌ SSE 事件会返回（虽然会被去重）

#### 严重性
⚠️ **中等** - 可能导致 UI 和后端不一致

#### 解决方案

**当前状态**：未处理

**缓解措施**：
1. 后端幂等性（使用 correlationId）
2. SSE 最终一致性
3. 合理设置超时时间

**未来改进**：
```typescript
// 使用 AbortController 取消请求
const abortController = new AbortController()

setTimeout(() => {
  abortController.abort()  // 真正取消请求
}, timeout)

fetch(url, { signal: abortController.signal })
```

**优先级**：P2（中优先级）

---

### 2. 超时导致 UI 和后端状态不一致

#### 现象
```typescript
// 时间线：
// T+0ms:  应用乐观更新（UI 显示"已创建"）
// T+100ms: 发送网络请求
// T+5000ms: 超时，回滚乐观更新（UI 显示"失败"）
// T+8000ms: 后端实际完成，返回成功
// T+8100ms: SSE 事件到达（UI 重新显示"已创建"）

// ⚠️ 用户看到：成功 → 失败 → 成功（闪烁 3 次）
```

#### 影响
- ❌ 用户体验差（UI 闪烁）
- ❌ 用户困惑（以为失败，实际成功）
- ❌ 可能导致重复操作

#### 严重性
⚠️ **中等** - 影响用户体验

#### 解决方案

**当前状态**：已有缓解措施

**缓解措施**：
1. SSE 会在后端成功后同步状态（最终一致）
2. INT 去重防止重复更新
3. 合理设置超时时间（大于后端 P95 延迟）

**最佳实践**：
```typescript
// 根据后端实际性能设置超时
'task.create': {
  meta: {
    timeout: 10000,  // 后端 P95: 2000ms → 设置 5 倍安全边际
  }
}
```

**优先级**：P3（低优先级，可接受）

---

### 3. Promise 内存占用

#### 现象
```typescript
// 超时后，executePromise 仍占用内存
const executePromise = fetch(...)  // Promise 对象
await Promise.race([executePromise, timeout(5000)])

// executePromise 仍存在，直到请求完成
```

#### 影响
- ⚠️ 每个超时的请求占用 ~500 bytes
- ⚠️ 1000 个超时请求 = ~500 KB

#### 严重性
✅ **低** - 影响极小

#### 解决方案

**当前状态**：无需处理

**原因**：
1. 浏览器自动垃圾回收
2. 请求完成后立即释放
3. 实际场景中超时很少发生

**优先级**：P4（无需处理）

---

### 4. Awaitable Dispatch Map 内存泄漏

#### 现象
```typescript
// 如果指令永远不完成，Map 会一直保存 resolver
this.promiseResolvers.set(instruction.id, { resolve, reject })

// 如果流水线 bug 或 stop，resolver 永远不释放
```

#### 影响
- ❌ 每个未完成指令占用 ~200 bytes
- ❌ 1000 个未完成指令 = ~200 KB

#### 严重性
✅ **低** - 已解决

#### 解决方案

**当前状态**：✅ 已解决

**实现**：
```typescript
// Pipeline.reset() 清理所有待处理 Promise
reset(): void {
  for (const [, resolver] of this.promiseResolvers.entries()) {
    resolver.reject(new Error('Pipeline was reset'))
  }
  this.promiseResolvers.clear()  // 清空 Map
}
```

**额外保护**（可选）：
```typescript
// 定期清理超时的 Promise（30秒）
setInterval(() => {
  const now = Date.now()
  for (const [id, data] of this.promiseResolvers.entries()) {
    if (now - data.createdAt > 30000) {
      data.reject(new Error('Instruction timeout (global)'))
      this.promiseResolvers.delete(id)
    }
  }
}, 10000)
```

**优先级**：P4（已解决）

---

### 5. 未处理的 Promise Rejection 警告

#### 现象
```typescript
// Fire-and-Forget 模式下，如果指令失败
pipeline.dispatch('task.create', { title: '任务' })
// 不 await，也不 catch

// 控制台警告：
// Uncaught (in promise) Error: xxx
```

#### 影响
- ⚠️ 控制台出现红色警告
- ⚠️ 可能干扰开发调试
- ✅ 不影响功能

#### 严重性
✅ **低** - 仅影响开发体验

#### 解决方案

**当前状态**：需用户处理

**最佳实践**：
```typescript
// ✅ 方式 1：await
try {
  await pipeline.dispatch('task.create', { title: '任务' })
} catch (error) {
  console.error(error)
}

// ✅ 方式 2：catch
pipeline.dispatch('task.create', { title: '任务' })
  .catch(error => console.error(error))

// ❌ 方式 3：忽略（会产生警告）
pipeline.dispatch('task.create', { title: '任务' })
```

**优先级**：P4（文档说明即可）

---

### 6. 异步执行导致的时序问题

#### 现象
```typescript
// 场景：快速连续发射两个冲突指令
pipeline.dispatch('task.update', { id: '123', title: 'A' })
pipeline.dispatch('task.update', { id: '123', title: 'B' })

// 预期：A 先执行，B 后执行
// 实际：两个指令都进入调度器，B 会等待 A 完成

// ⚠️ 但如果 A 失败，B 会立即执行
// 最终结果：title = 'B'
```

#### 影响
- ⚠️ 执行顺序依赖调度器实现
- ⚠️ 如果前一个指令失败，后续指令可能得到不一致的状态
- ✅ 符合资源冲突检测设计

#### 严重性
⚠️ **中等** - 设计权衡

#### 解决方案

**当前状态**：设计预期

**设计原理**：
- 调度器确保同一资源的指令串行执行
- 失败的指令会释放资源，让后续指令继续

**最佳实践**：
```typescript
// 如果需要严格时序，使用 await
await pipeline.dispatch('task.update', { id: '123', title: 'A' })
await pipeline.dispatch('task.update', { id: '123', title: 'B' })

// 或者设计幂等指令（不依赖前一个的结果）
```

**优先级**：P4（设计预期，无需修改）

---

### 7. 乐观更新回滚导致的 UI 闪烁

#### 现象
```typescript
// 时间线：
// T+0ms:  拖动任务到新日期（UI 立即更新）
// T+100ms: 发送网络请求
// T+200ms: 后端返回 500 错误
// T+202ms: 回滚乐观更新（UI 变回原位）

// ⚠️ 用户看到：任务移动到新位置 → 弹回原位
```

#### 影响
- ⚠️ UI 闪烁
- ⚠️ 用户体验差
- ✅ 数据一致性正确

#### 严重性
⚠️ **中等** - 设计预期

#### 解决方案

**当前状态**：设计预期

**设计原理**：
- 乐观更新假设成功，失败时回滚
- 回滚确保 UI 和后端状态一致

**缓解措施**：
1. 提高后端成功率（修复 database locked 等问题）✅ 已完成
2. 合理的超时时间设置
3. 后端性能优化

**监控指标**：
```typescript
// 监控回滚率，应该 < 5%
const stats = cpuLogger.analyzeOptimisticRollbackRate()
if (stats.rollbackRate > 0.05) {
  console.warn('⚠️ 回滚率过高，检查后端问题')
}
```

**优先级**：P3（可接受，持续监控）

---

### 8. 资源冲突导致的调度延迟

#### 现象
```typescript
// 同一任务的两个操作
pipeline.dispatch('task.update', { id: '123', title: 'A' })
pipeline.dispatch('task.complete', { id: '123' })

// 第二个指令会等待第一个完成
// ⚠️ 延迟 = 第一个指令的执行时间
```

#### 影响
- ⚠️ 用户操作延迟响应
- ⚠️ 在资源冲突多时，延迟累积
- ✅ 确保数据一致性

#### 严重性
✅ **低** - 设计预期

#### 解决方案

**当前状态**：设计预期

**设计原理**：
- 调度器确保同一资源的指令串行执行
- 防止并发修改导致的数据竞争

**监控工具**：
```typescript
// 查看资源冲突热点
const conflicts = cpuLogger.analyzeResourceConflicts()
console.table(conflicts)

// 输出：
// task:abc123  23 次冲突  平均等待 156ms
// ⚠️ 如果某个资源冲突过多，考虑优化业务逻辑
```

**优先级**：P4（设计预期，无需修改）

---

### 9. 批量事件采集的内存峰值

#### 现象
```typescript
// CPUEventCollector 批量刷新事件
private eventQueue: CPUEvent[] = []
private maxBatchSize: number = 100

// 峰值场景：同时发射 1000 个指令
// eventQueue 会暂时存储大量事件
```

#### 影响
- ⚠️ 短时间内存峰值（~100KB）
- ✅ 50ms 后自动刷新并清空
- ✅ 不影响功能

#### 严重性
✅ **低** - 可忽略

#### 解决方案

**当前状态**：无需处理

**设计保护**：
1. 批量大小限制（100 条）
2. 定时刷新（50ms）
3. 达到批量大小立即刷新

**监控**：
```typescript
// 监控事件队列大小
console.log('Event queue size:', cpuEventCollector['eventQueue'].length)
// 正常 < 100，峰值时可能达到 100
```

**优先级**：P4（无需处理）

---

### 10. SSE 去重可能丢失合法事件

#### 现象
```typescript
// 场景：
// 1. 用户 A 在浏览器 A 修改任务
// 2. 用户 B 在浏览器 B 也修改同一任务（几乎同时）
// 3. 两个操作都成功，后端发送两个 SSE 事件
// 4. 浏览器 A 的 INT 表中有自己的 correlationId
// 5. 浏览器 B 的事件到达时，可能与 A 的 correlationId 冲突

// ⚠️ 虽然概率极低，但理论上可能丢失事件
```

#### 影响
- ⚠️ 多用户协作时，可能丢失其他用户的更新
- ⚠️ TTL 过长会增加误判概率
- ✅ TTL 过短会增加漏过概率

#### 严重性
⚠️ **中等** - 多用户协作场景

#### 解决方案

**当前状态**：已知，设计权衡

**当前配置**：
```typescript
// INT 表 TTL: 10 秒
private readonly TTL = 10000
```

**设计权衡**：
- TTL 过长 → 可能丢失其他用户的更新
- TTL 过短 → 可能漏过本机的 SSE（产生重复更新）

**最佳实践**：
```typescript
// 后端在 SSE 中包含操作来源信息
{
  "correlation_id": "corr_xxx",
  "user_id": "user_123",  // 🔥 区分不同用户
  "payload": { ... }
}

// 前端只去重自己的事件
if (event.correlation_id && event.user_id === currentUserId) {
  // 是本机操作，去重
}
```

**优先级**：P3（未来优化）

---

### 11. 日志系统的性能开销

#### 现象
```typescript
// 每个指令会产生 5-10 个事件
// 每个事件会：
// 1. 存入 eventQueue
// 2. 批量刷新到 CPULogger
// 3. 建立索引
// 4. 更新统计
```

#### 影响
- ⚠️ CPU 开销（约 0.1-0.5%）
- ⚠️ 内存占用（10000 事件约 5MB）
- ✅ 异步处理，不阻塞流水线

#### 严重性
✅ **低** - 可忽略

#### 解决方案

**当前状态**：已优化

**优化措施**：
1. 批量刷新（50ms）
2. queueMicrotask 异步处理
3. 事件数量限制（10000 条）
4. 自动清理旧事件

**监控**：
```typescript
const stats = cpuLogger.getStats()
console.log('存储使用:', stats.storageUsage, '/', stats.maxStorage)
```

**优先级**：P4（无需处理）

---

### 12. 控制台输出的性能影响

#### 现象
```typescript
// console.log 的性能开销
cpuConsole.onInstructionSuccess(instruction, duration)
// 会调用 console.groupCollapsed, console.log, console.table 等
```

#### 影响
- ⚠️ 高频打印会降低性能（每次约 0.1-0.5ms）
- ⚠️ DEBUG 级别下会打印 payload（大对象序列化慢）
- ✅ 可通过级别控制

#### 严重性
✅ **低** - 可控

#### 解决方案

**当前状态**：已有控制机制

**控制方式**：
```typescript
// 开发环境：VERBOSE（详细）
cpuConsole.setLevel(ConsoleLevel.VERBOSE)

// 生产环境：MINIMAL（只看失败）
cpuConsole.setLevel(ConsoleLevel.MINIMAL)

// 性能测试：SILENT（完全关闭）
cpuConsole.setLevel(ConsoleLevel.SILENT)
```

**优先级**：P4（无需处理）

---

## 📊 严重性分级标准

| 级别 | 定义 | 影响 | 示例 |
|------|------|------|------|
| 🔴 **高** | 导致数据丢失或损坏 | 功能不可用 | - |
| ⚠️ **中等** | 影响用户体验或性能 | 功能可用但体验差 | 超时后请求仍执行、UI 闪烁 |
| ✅ **低** | 理论上存在，实际影响小 | 可忽略 | Promise 内存占用 |

---

## 🎯 处理优先级

| 优先级 | 定义 | 处理策略 |
|--------|------|---------|
| **P1** | 立即修复 | 阻塞发布 |
| **P2** | 计划修复 | 下个版本 |
| **P3** | 可选优化 | 有时间再做 |
| **P4** | 无需处理 | 文档说明 |

---

## 📝 当前状态总结

### 需要处理（P2）
1. ⚠️ 超时后网络请求仍在执行（添加 AbortController 支持）

### 可选优化（P3）
2. ⚠️ 超时导致 UI 和后端不一致（优化超时时间配置）
3. ⚠️ 乐观更新回滚导致的 UI 闪烁（提高后端成功率）
4. ⚠️ SSE 去重可能丢失合法事件（添加用户 ID 区分）

### 无需处理（P4）
5. ✅ Promise 内存占用（自动 GC）
6. ✅ Awaitable Dispatch Map 内存泄漏（已解决）
7. ✅ 未处理的 Promise Rejection 警告（文档说明）
8. ✅ 异步执行导致的时序问题（设计预期）
9. ✅ 资源冲突导致的调度延迟（设计预期）
10. ✅ 批量事件采集的内存峰值（可忽略）
11. ✅ 日志系统的性能开销（已优化）
12. ✅ 控制台输出的性能影响（可控）

---

## 🔍 监控与检测

### 日常监控

```typescript
import { cpuLogger, cpuDebugger } from '@/cpu/logging'

// 1. 监控回滚率（每天）
const rollbackStats = cpuLogger.analyzeOptimisticRollbackRate()
if (rollbackStats.rollbackRate > 0.05) {
  console.warn('⚠️ 回滚率过高:', rollbackStats.rollbackRate)
}

// 2. 监控资源冲突（每周）
const conflicts = cpuLogger.analyzeResourceConflicts()
if (conflicts[0]?.conflictCount > 100) {
  console.warn('⚠️ 资源冲突过多:', conflicts[0])
}

// 3. 监控性能（每周）
const slowest = cpuDebugger.getSlowestInstructions(10)
if (slowest[0]?.duration > 5000) {
  console.warn('⚠️ 发现慢指令:', slowest[0])
}

// 4. 监控错误率（实时）
const stats = cpuDebugger.getRealtimeStats(60)
if (stats.errorRate > 0.1) {
  console.warn('⚠️ 错误率过高:', stats.errorRate)
}
```

### 性能基准

| 指标 | 目标值 | 警告阈值 | 检测方法 |
|------|--------|---------|---------|
| 回滚率 | < 2% | > 5% | `analyzeOptimisticRollbackRate()` |
| 平均延迟 | < 200ms | > 500ms | `analyzeInstructionPerformance()` |
| P95 延迟 | < 500ms | > 1000ms | `analyzeInstructionPerformance()` |
| 错误率 | < 5% | > 10% | `getRealtimeStats()` |
| 资源冲突 | < 10 次/分钟 | > 50 次/分钟 | `analyzeResourceConflicts()` |

---

## 🎉 结论

### 总体评估
- ✅ **大部分副作用可控**
- ✅ **严重问题已解决**（Map 泄漏、backend 并发）
- ⚠️ **中等问题已有缓解**（SSE 去重、超时配置）
- ✅ **设计预期的副作用可接受**（资源冲突延迟、乐观更新回滚）

### 推荐行动
1. **立即**：无（当前实现可用于生产）
2. **短期**（1-2 周）：添加 AbortController 支持
3. **中期**（1-2 月）：优化 SSE 去重逻辑（添加 user_id）
4. **长期**：持续监控性能指标

### 风险评估
- 🟢 **整体风险：低**
- 🟢 **可用于生产环境**
- 🟡 **建议持续监控回滚率和错误率**

---

**最后更新**: 2025-10-15  
**版本**: v1.0  
**状态**: 所有已知副作用已记录并评估


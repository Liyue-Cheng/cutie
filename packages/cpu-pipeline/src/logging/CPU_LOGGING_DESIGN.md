# 🚀 CPU Pipeline 专用日志与调试系统设计

**设计目标**: 为 CPU Pipeline 打造专业级指令追踪、性能分析和调试工具  
**设计原则**: 零依赖、高性能、智能化、可视化

---

## 📋 目录

1. [现有问题分析](#现有问题分析)
2. [架构设计](#架构设计)
3. [核心组件](#核心组件)
4. [API 设计](#api-设计)
5. [调试器 UI 增强](#调试器-ui-增强)
6. [实施计划](#实施计划)

---

## 🔍 现有问题分析

### 旧 Logger 的问题

```typescript
// ❌ 问题1: 使用通用 logger，不了解 CPU 指令的特殊需求
logger.info('System:Pipeline', 'WB: 指令完成', { instructionId, type })

// ❌ 问题2: 简单的 console.log，缺乏结构化
console.log(`🎯 指令完成: ${trace.type}`, this.formatTraceInfo(trace))

// ❌ 问题3: 无法进行复杂查询
// "找出所有执行超过 100ms 的 schedule.update 指令"
// "找出所有触发了回滚的指令"
// "分析资源冲突导致的调度延迟"
```

### InstructionTracker 的局限

1. **功能单一**: 只记录时间戳和状态，缺少深度信息
2. **无持久化**: 刷新页面后丢失所有历史
3. **无聚合分析**: 无法统计平均耗时、成功率等
4. **无关联分析**: 无法追踪 `correlationId` 的完整链路
5. **调试困难**: 无法重放指令、无法导出数据

---

## 🏗️ 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────┐
│                   CPU Pipeline                       │
│  IF → SCH → EX → RES → WB → INT                     │
└───────────────┬─────────────────────────────────────┘
                │ 发送指令事件
                ↓
┌─────────────────────────────────────────────────────┐
│            CPUEventCollector (事件采集器)             │
│  - 捕获所有指令生命周期事件                            │
│  - 零侵入式设计，不影响流水线性能                       │
└───────────────┬─────────────────────────────────────┘
                │ 结构化事件流
                ↓
┌─────────────────────────────────────────────────────┐
│             CPULogger (日志记录器)                    │
│  - 结构化存储                                        │
│  - 智能索引                                          │
│  - 自动分析                                          │
└───────────────┬─────────────────────────────────────┘
                │ 查询接口
                ↓
┌─────────────────────────────────────────────────────┐
│           CPUDebugger (调试器)                       │
│  - 实时监控面板                                      │
│  - 性能分析图表                                      │
│  - 指令查询引擎                                      │
│  - 时间旅行调试                                      │
└─────────────────────────────────────────────────────┘
```

### 核心设计原则

1. **零依赖**: 完全独立，不依赖旧 logger
2. **高性能**: 异步批量处理，不阻塞流水线
3. **结构化**: 所有数据都是强类型、可查询的
4. **智能化**: 自动检测异常、分析性能瓶颈
5. **可视化**: 丰富的图表和交互式调试界面

---

## 🧩 核心组件

### 1. CPUEvent (事件模型)

```typescript
/**
 * CPU 指令事件类型
 */
export enum CPUEventType {
  // 指令生命周期
  INSTRUCTION_CREATED = 'instruction.created',
  INSTRUCTION_ISSUED = 'instruction.issued',
  INSTRUCTION_EXECUTING = 'instruction.executing',
  INSTRUCTION_RESPONDED = 'instruction.responded',
  INSTRUCTION_COMMITTED = 'instruction.committed',
  INSTRUCTION_FAILED = 'instruction.failed',

  // 乐观更新
  OPTIMISTIC_APPLIED = 'optimistic.applied',
  OPTIMISTIC_ROLLED_BACK = 'optimistic.rolled_back',

  // 调度器
  SCHEDULER_CONFLICT_DETECTED = 'scheduler.conflict_detected',
  SCHEDULER_INSTRUCTION_QUEUED = 'scheduler.instruction_queued',
  SCHEDULER_INSTRUCTION_DEQUEUED = 'scheduler.instruction_dequeued',

  // 网络
  NETWORK_REQUEST_SENT = 'network.request_sent',
  NETWORK_RESPONSE_RECEIVED = 'network.response_received',
  NETWORK_ERROR = 'network.error',

  // 中断
  INTERRUPT_REGISTERED = 'interrupt.registered',
  INTERRUPT_DISPATCHED = 'interrupt.dispatched',
  INTERRUPT_DEDUPLICATED = 'interrupt.deduplicated',

  // 性能
  PERFORMANCE_WARNING = 'performance.warning',
  PERFORMANCE_BOTTLENECK = 'performance.bottleneck',
}

/**
 * CPU 事件基础接口
 */
export interface CPUEvent {
  // 基础信息
  eventId: string
  eventType: CPUEventType
  timestamp: number

  // 指令上下文
  instructionId: string
  instructionType: string
  correlationId: string

  // 流水线状态
  pipelineStage: PipelineStage
  instructionStatus: InstructionStatus

  // 性能指标
  latency?: number // 该事件的延迟（相对于上一个事件）
  duration?: number // 该阶段的持续时间

  // 事件数据
  payload: any

  // 元数据
  metadata?: {
    resourceIds?: string[]
    priority?: number
    retryCount?: number
    tags?: string[]
  }
}

/**
 * 特定事件类型的详细接口
 */
export interface InstructionCreatedEvent extends CPUEvent {
  eventType: CPUEventType.INSTRUCTION_CREATED
  payload: {
    instructionType: string
    payload: any
    origin: 'user' | 'system' | 'sse'
  }
}

export interface OptimisticAppliedEvent extends CPUEvent {
  eventType: CPUEventType.OPTIMISTIC_APPLIED
  payload: {
    snapshot: any
    changes: any // 应用了什么变更
  }
}

export interface OptimisticRolledBackEvent extends CPUEvent {
  eventType: CPUEventType.OPTIMISTIC_ROLLED_BACK
  payload: {
    snapshot: any
    reason: string
    error?: Error
  }
}

export interface SchedulerConflictEvent extends CPUEvent {
  eventType: CPUEventType.SCHEDULER_CONFLICT_DETECTED
  payload: {
    conflictingInstructions: string[]
    conflictingResources: string[]
    waitTime: number
  }
}

export interface NetworkRequestEvent extends CPUEvent {
  eventType: CPUEventType.NETWORK_REQUEST_SENT
  payload: {
    method: string
    url: string
    headers: Record<string, string>
  }
}

export interface NetworkResponseEvent extends CPUEvent {
  eventType: CPUEventType.NETWORK_RESPONSE_RECEIVED
  payload: {
    status: number
    latency: number
    size: number
  }
}

export interface PerformanceWarningEvent extends CPUEvent {
  eventType: CPUEventType.PERFORMANCE_WARNING
  payload: {
    metric: 'latency' | 'throughput' | 'queue_depth'
    threshold: number
    actual: number
    suggestion: string
  }
}
```

### 2. CPUEventCollector (事件采集器)

```typescript
/**
 * CPU 事件采集器
 *
 * 职责：
 * 1. 在流水线各阶段捕获事件
 * 2. 批量异步发送给 CPULogger
 * 3. 零侵入、零延迟
 */
export class CPUEventCollector {
  private eventQueue: CPUEvent[] = []
  private flushInterval: number = 50 // 50ms 批量刷新
  private maxBatchSize: number = 100
  private enabled: boolean = true

  /**
   * 发送事件（异步，不阻塞流水线）
   */
  emit(event: Partial<CPUEvent>): void {
    if (!this.enabled) return

    const fullEvent: CPUEvent = {
      eventId: this.generateEventId(),
      timestamp: Date.now(),
      ...event,
    } as CPUEvent

    this.eventQueue.push(fullEvent)

    // 触发批量刷新
    if (this.eventQueue.length >= this.maxBatchSize) {
      this.flush()
    }
  }

  /**
   * 批量刷新事件
   */
  private flush(): void {
    if (this.eventQueue.length === 0) return

    const batch = this.eventQueue.splice(0, this.maxBatchSize)

    // 异步发送给 Logger（使用 queueMicrotask 确保不阻塞）
    queueMicrotask(() => {
      cpuLogger.ingestBatch(batch)
    })
  }

  /**
   * 便捷方法：指令创建
   */
  onInstructionCreated(instruction: QueuedInstruction): void {
    this.emit({
      eventType: CPUEventType.INSTRUCTION_CREATED,
      instructionId: instruction.id,
      instructionType: instruction.type,
      correlationId: instruction.context.correlationId,
      pipelineStage: PipelineStage.IF,
      instructionStatus: InstructionStatus.PENDING,
      payload: {
        instructionType: instruction.type,
        payload: instruction.payload,
        origin: 'user',
      },
    })
  }

  /**
   * 便捷方法：乐观更新应用
   */
  onOptimisticApplied(
    instructionId: string,
    instructionType: string,
    correlationId: string,
    snapshot: any,
    changes: any
  ): void {
    this.emit({
      eventType: CPUEventType.OPTIMISTIC_APPLIED,
      instructionId,
      instructionType,
      correlationId,
      pipelineStage: PipelineStage.EX,
      instructionStatus: InstructionStatus.EXECUTING,
      payload: { snapshot, changes },
    })
  }

  /**
   * 便捷方法：乐观更新回滚
   */
  onOptimisticRolledBack(
    instructionId: string,
    instructionType: string,
    correlationId: string,
    snapshot: any,
    reason: string,
    error?: Error
  ): void {
    this.emit({
      eventType: CPUEventType.OPTIMISTIC_ROLLED_BACK,
      instructionId,
      instructionType,
      correlationId,
      pipelineStage: PipelineStage.WB,
      instructionStatus: InstructionStatus.FAILED,
      payload: { snapshot, reason, error: error?.message },
      metadata: { tags: ['rollback', 'failure'] },
    })
  }

  /**
   * 便捷方法：资源冲突检测
   */
  onSchedulerConflict(
    instructionId: string,
    instructionType: string,
    correlationId: string,
    conflictingInstructions: string[],
    conflictingResources: string[],
    waitTime: number
  ): void {
    this.emit({
      eventType: CPUEventType.SCHEDULER_CONFLICT_DETECTED,
      instructionId,
      instructionType,
      correlationId,
      pipelineStage: PipelineStage.SCH,
      instructionStatus: InstructionStatus.ISSUED,
      payload: {
        conflictingInstructions,
        conflictingResources,
        waitTime,
      },
      metadata: {
        resourceIds: conflictingResources,
        tags: ['conflict', 'scheduler'],
      },
    })
  }

  /**
   * 便捷方法：网络请求
   */
  onNetworkRequest(
    instructionId: string,
    instructionType: string,
    correlationId: string,
    method: string,
    url: string
  ): void {
    this.emit({
      eventType: CPUEventType.NETWORK_REQUEST_SENT,
      instructionId,
      instructionType,
      correlationId,
      pipelineStage: PipelineStage.EX,
      instructionStatus: InstructionStatus.EXECUTING,
      payload: { method, url },
      metadata: { tags: ['network'] },
    })
  }

  /**
   * 便捷方法：网络响应
   */
  onNetworkResponse(
    instructionId: string,
    instructionType: string,
    correlationId: string,
    status: number,
    latency: number,
    size: number
  ): void {
    this.emit({
      eventType: CPUEventType.NETWORK_RESPONSE_RECEIVED,
      instructionId,
      instructionType,
      correlationId,
      pipelineStage: PipelineStage.RES,
      instructionStatus: InstructionStatus.RESPONDED,
      payload: { status, latency, size },
      latency,
      metadata: { tags: ['network'] },
    })

    // 🔥 自动检测性能警告
    if (latency > 1000) {
      this.emit({
        eventType: CPUEventType.PERFORMANCE_WARNING,
        instructionId,
        instructionType,
        correlationId,
        pipelineStage: PipelineStage.RES,
        instructionStatus: InstructionStatus.RESPONDED,
        payload: {
          metric: 'latency',
          threshold: 1000,
          actual: latency,
          suggestion: `网络请求耗时 ${latency}ms，超过阈值 1000ms`,
        },
        metadata: { tags: ['performance', 'warning'] },
      })
    }
  }

  private generateEventId(): string {
    return `evt_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
  }
}

export const cpuEventCollector = new CPUEventCollector()
```

### 3. CPULogger (日志记录器)

```typescript
/**
 * CPU 日志记录器
 *
 * 职责：
 * 1. 存储和索引所有 CPU 事件
 * 2. 提供强大的查询 API
 * 3. 自动分析和聚合
 */
export class CPULogger {
  // 存储
  private events: CPUEvent[] = []
  private maxEvents: number = 10000 // 保留最近 10000 条事件

  // 索引
  private eventsByInstruction = new Map<string, CPUEvent[]>()
  private eventsByCorrelation = new Map<string, CPUEvent[]>()
  private eventsByType = new Map<CPUEventType, CPUEvent[]>()

  // 统计
  private stats = {
    totalEvents: 0,
    eventCounts: new Map<CPUEventType, number>(),
    instructionCounts: new Map<string, number>(),
  }

  /**
   * 批量接收事件
   */
  ingestBatch(events: CPUEvent[]): void {
    for (const event of events) {
      this.ingestEvent(event)
    }
  }

  /**
   * 接收单个事件
   */
  private ingestEvent(event: CPUEvent): void {
    // 存储
    this.events.push(event)
    if (this.events.length > this.maxEvents) {
      const removed = this.events.shift()!
      this.removeFromIndexes(removed)
    }

    // 索引
    this.addToIndex(this.eventsByInstruction, event.instructionId, event)
    this.addToIndex(this.eventsByCorrelation, event.correlationId, event)
    this.addToIndex(this.eventsByType, event.eventType, event)

    // 统计
    this.stats.totalEvents++
    this.stats.eventCounts.set(
      event.eventType,
      (this.stats.eventCounts.get(event.eventType) || 0) + 1
    )
    this.stats.instructionCounts.set(
      event.instructionType,
      (this.stats.instructionCounts.get(event.instructionType) || 0) + 1
    )
  }

  private addToIndex<K>(map: Map<K, CPUEvent[]>, key: K, event: CPUEvent): void {
    if (!map.has(key)) {
      map.set(key, [])
    }
    map.get(key)!.push(event)
  }

  private removeFromIndexes(event: CPUEvent): void {
    // 从索引中移除（简化实现，实际可能需要更复杂的清理逻辑）
  }

  // ==================== 查询 API ====================

  /**
   * 查询：获取指令的完整事件链
   */
  getInstructionTrace(instructionId: string): CPUEvent[] {
    return this.eventsByInstruction.get(instructionId) || []
  }

  /**
   * 查询：获取 correlationId 的完整链路
   */
  getCorrelationTrace(correlationId: string): CPUEvent[] {
    return this.eventsByCorrelation.get(correlationId) || []
  }

  /**
   * 查询：按类型过滤事件
   */
  getEventsByType(type: CPUEventType): CPUEvent[] {
    return this.eventsByType.get(type) || []
  }

  /**
   * 查询：按指令类型过滤
   */
  getEventsByInstructionType(instructionType: string): CPUEvent[] {
    return this.events.filter((e) => e.instructionType === instructionType)
  }

  /**
   * 查询：按时间范围过滤
   */
  getEventsByTimeRange(startTime: number, endTime: number): CPUEvent[] {
    return this.events.filter((e) => e.timestamp >= startTime && e.timestamp <= endTime)
  }

  /**
   * 查询：按标签过滤
   */
  getEventsByTags(tags: string[]): CPUEvent[] {
    return this.events.filter((e) => tags.some((tag) => e.metadata?.tags?.includes(tag)))
  }

  /**
   * 高级查询：复杂条件
   */
  query(filter: {
    instructionType?: string
    eventType?: CPUEventType
    pipelineStage?: PipelineStage
    instructionStatus?: InstructionStatus
    timeRange?: { start: number; end: number }
    tags?: string[]
    minLatency?: number
    maxLatency?: number
  }): CPUEvent[] {
    let results = this.events

    if (filter.instructionType) {
      results = results.filter((e) => e.instructionType === filter.instructionType)
    }

    if (filter.eventType) {
      results = results.filter((e) => e.eventType === filter.eventType)
    }

    if (filter.pipelineStage) {
      results = results.filter((e) => e.pipelineStage === filter.pipelineStage)
    }

    if (filter.instructionStatus) {
      results = results.filter((e) => e.instructionStatus === filter.instructionStatus)
    }

    if (filter.timeRange) {
      results = results.filter(
        (e) => e.timestamp >= filter.timeRange!.start && e.timestamp <= filter.timeRange!.end
      )
    }

    if (filter.tags) {
      results = results.filter((e) => filter.tags!.some((tag) => e.metadata?.tags?.includes(tag)))
    }

    if (filter.minLatency !== undefined) {
      results = results.filter((e) => e.latency !== undefined && e.latency >= filter.minLatency!)
    }

    if (filter.maxLatency !== undefined) {
      results = results.filter((e) => e.latency !== undefined && e.latency <= filter.maxLatency!)
    }

    return results
  }

  // ==================== 分析 API ====================

  /**
   * 分析：指令性能统计
   */
  analyzeInstructionPerformance(instructionType: string): {
    count: number
    successRate: number
    avgLatency: number
    p50: number
    p95: number
    p99: number
  } {
    const instructions = Array.from(this.eventsByInstruction.entries()).filter(
      ([_, events]) => events[0]?.instructionType === instructionType
    )

    if (instructions.length === 0) {
      return {
        count: 0,
        successRate: 0,
        avgLatency: 0,
        p50: 0,
        p95: 0,
        p99: 0,
      }
    }

    const latencies: number[] = []
    let successCount = 0

    for (const [_, events] of instructions) {
      const commitEvent = events.find((e) => e.eventType === CPUEventType.INSTRUCTION_COMMITTED)
      const failEvent = events.find((e) => e.eventType === CPUEventType.INSTRUCTION_FAILED)

      if (commitEvent) {
        successCount++
        const duration = commitEvent.timestamp - events[0].timestamp
        latencies.push(duration)
      } else if (failEvent) {
        const duration = failEvent.timestamp - events[0].timestamp
        latencies.push(duration)
      }
    }

    latencies.sort((a, b) => a - b)

    return {
      count: instructions.length,
      successRate: successCount / instructions.length,
      avgLatency: latencies.reduce((a, b) => a + b, 0) / latencies.length || 0,
      p50: this.percentile(latencies, 0.5),
      p95: this.percentile(latencies, 0.95),
      p99: this.percentile(latencies, 0.99),
    }
  }

  /**
   * 分析：资源冲突热点
   */
  analyzeResourceConflicts(): Array<{
    resource: string
    conflictCount: number
    avgWaitTime: number
    involvedInstructions: string[]
  }> {
    const conflicts = this.getEventsByType(CPUEventType.SCHEDULER_CONFLICT_DETECTED)

    const resourceMap = new Map<
      string,
      { count: number; totalWait: number; instructions: Set<string> }
    >()

    for (const event of conflicts) {
      const { conflictingResources, waitTime, conflictingInstructions } = event.payload

      for (const resource of conflictingResources) {
        if (!resourceMap.has(resource)) {
          resourceMap.set(resource, {
            count: 0,
            totalWait: 0,
            instructions: new Set(),
          })
        }

        const entry = resourceMap.get(resource)!
        entry.count++
        entry.totalWait += waitTime
        conflictingInstructions.forEach((id: string) => entry.instructions.add(id))
      }
    }

    return Array.from(resourceMap.entries())
      .map(([resource, data]) => ({
        resource,
        conflictCount: data.count,
        avgWaitTime: data.totalWait / data.count,
        involvedInstructions: Array.from(data.instructions),
      }))
      .sort((a, b) => b.conflictCount - a.conflictCount)
  }

  /**
   * 分析：乐观更新回滚率
   */
  analyzeOptimisticRollbackRate(): {
    totalOptimistic: number
    rollbackCount: number
    rollbackRate: number
    byInstructionType: Record<string, { total: number; rollbacks: number; rate: number }>
  } {
    const appliedEvents = this.getEventsByType(CPUEventType.OPTIMISTIC_APPLIED)
    const rolledBackEvents = this.getEventsByType(CPUEventType.OPTIMISTIC_ROLLED_BACK)

    const byType: Record<string, { total: number; rollbacks: number; rate: number }> = {}

    // 统计每种指令类型的乐观更新和回滚
    for (const event of appliedEvents) {
      if (!byType[event.instructionType]) {
        byType[event.instructionType] = { total: 0, rollbacks: 0, rate: 0 }
      }
      byType[event.instructionType].total++
    }

    for (const event of rolledBackEvents) {
      if (!byType[event.instructionType]) {
        byType[event.instructionType] = { total: 0, rollbacks: 0, rate: 0 }
      }
      byType[event.instructionType].rollbacks++
    }

    // 计算回滚率
    for (const type in byType) {
      byType[type].rate = byType[type].rollbacks / byType[type].total
    }

    return {
      totalOptimistic: appliedEvents.length,
      rollbackCount: rolledBackEvents.length,
      rollbackRate: rolledBackEvents.length / appliedEvents.length || 0,
      byInstructionType: byType,
    }
  }

  /**
   * 分析：流水线吞吐量
   */
  analyzeThroughput(timeWindowMs: number = 60000): {
    instructionsPerSecond: number
    eventsPerSecond: number
    avgPipelineUtilization: number
  } {
    const now = Date.now()
    const startTime = now - timeWindowMs

    const recentEvents = this.getEventsByTimeRange(startTime, now)
    const instructionIds = new Set(recentEvents.map((e) => e.instructionId))

    return {
      instructionsPerSecond: (instructionIds.size / timeWindowMs) * 1000,
      eventsPerSecond: (recentEvents.length / timeWindowMs) * 1000,
      avgPipelineUtilization: 0, // TODO: 计算流水线利用率
    }
  }

  /**
   * 工具：计算百分位数
   */
  private percentile(values: number[], p: number): number {
    if (values.length === 0) return 0
    const index = Math.ceil(values.length * p) - 1
    return values[index] || 0
  }

  /**
   * 获取统计信息
   */
  getStats() {
    return {
      ...this.stats,
      totalInstructions: this.eventsByInstruction.size,
      totalCorrelations: this.eventsByCorrelation.size,
      storageUsage: this.events.length,
      maxStorage: this.maxEvents,
    }
  }

  /**
   * 导出数据（用于离线分析）
   */
  exportData(filter?: any): {
    events: CPUEvent[]
    stats: any
    exportTime: number
  } {
    const events = filter ? this.query(filter) : this.events

    return {
      events,
      stats: this.getStats(),
      exportTime: Date.now(),
    }
  }

  /**
   * 清空数据
   */
  clear(): void {
    this.events = []
    this.eventsByInstruction.clear()
    this.eventsByCorrelation.clear()
    this.eventsByType.clear()
    this.stats = {
      totalEvents: 0,
      eventCounts: new Map(),
      instructionCounts: new Map(),
    }
  }
}

export const cpuLogger = new CPULogger()
```

### 4. CPUDebugger (调试器)

```typescript
/**
 * CPU 调试器
 *
 * 职责：
 * 1. 提供调试器 API
 * 2. 支持时间旅行调试
 * 3. 支持指令重放
 */
export class CPUDebugger {
  /**
   * 查询：执行最慢的指令
   */
  getSlowestInstructions(limit: number = 10): Array<{
    instructionId: string
    instructionType: string
    duration: number
    events: CPUEvent[]
  }> {
    const instructionTraces = Array.from(cpuLogger['eventsByInstruction'].entries())

    const withDuration = instructionTraces.map(([instructionId, events]) => {
      const firstEvent = events[0]
      const lastEvent = events[events.length - 1]
      const duration = lastEvent.timestamp - firstEvent.timestamp

      return {
        instructionId,
        instructionType: firstEvent.instructionType,
        duration,
        events,
      }
    })

    return withDuration.sort((a, b) => b.duration - a.duration).slice(0, limit)
  }

  /**
   * 查询：失败的指令
   */
  getFailedInstructions(): Array<{
    instructionId: string
    instructionType: string
    error: string
    events: CPUEvent[]
  }> {
    const failEvents = cpuLogger.getEventsByType(CPUEventType.INSTRUCTION_FAILED)

    return failEvents.map((failEvent) => ({
      instructionId: failEvent.instructionId,
      instructionType: failEvent.instructionType,
      error: failEvent.payload?.error || 'Unknown error',
      events: cpuLogger.getInstructionTrace(failEvent.instructionId),
    }))
  }

  /**
   * 查询：触发回滚的指令
   */
  getRolledBackInstructions(): Array<{
    instructionId: string
    instructionType: string
    reason: string
    events: CPUEvent[]
  }> {
    const rollbackEvents = cpuLogger.getEventsByType(CPUEventType.OPTIMISTIC_ROLLED_BACK)

    return rollbackEvents.map((event) => ({
      instructionId: event.instructionId,
      instructionType: event.instructionType,
      reason: event.payload.reason,
      events: cpuLogger.getInstructionTrace(event.instructionId),
    }))
  }

  /**
   * 查询：资源冲突链
   */
  getResourceConflictChain(instructionId: string): Array<{
    instructionId: string
    instructionType: string
    blockedBy: string[]
    waitTime: number
  }> {
    const events = cpuLogger.getInstructionTrace(instructionId)
    const conflictEvents = events.filter(
      (e) => e.eventType === CPUEventType.SCHEDULER_CONFLICT_DETECTED
    )

    return conflictEvents.map((event) => ({
      instructionId: event.instructionId,
      instructionType: event.instructionType,
      blockedBy: event.payload.conflictingInstructions,
      waitTime: event.payload.waitTime,
    }))
  }

  /**
   * 时间旅行：重放指令
   */
  replayInstruction(instructionId: string): {
    success: boolean
    events: CPUEvent[]
    timeline: Array<{ time: number; stage: string; event: string }>
  } {
    const events = cpuLogger.getInstructionTrace(instructionId)

    const timeline = events.map((event) => ({
      time: event.timestamp,
      stage: event.pipelineStage,
      event: event.eventType,
    }))

    return {
      success: events.some((e) => e.eventType === CPUEventType.INSTRUCTION_COMMITTED),
      events,
      timeline,
    }
  }

  /**
   * 诊断：分析指令为什么慢
   */
  diagnoseSlowInstruction(instructionId: string): {
    instructionId: string
    totalDuration: number
    bottleneck: { stage: string; duration: number; percentage: number }
    breakdown: Array<{ stage: string; duration: number; percentage: number }>
    suggestions: string[]
  } {
    const events = cpuLogger.getInstructionTrace(instructionId)
    if (events.length === 0) {
      throw new Error(`Instruction ${instructionId} not found`)
    }

    const firstEvent = events[0]
    const lastEvent = events[events.length - 1]
    const totalDuration = lastEvent.timestamp - firstEvent.timestamp

    // 计算每个阶段的耗时
    const stageBreakdown = new Map<string, number>()
    for (let i = 1; i < events.length; i++) {
      const prevEvent = events[i - 1]
      const currEvent = events[i]
      const duration = currEvent.timestamp - prevEvent.timestamp
      const stage = `${prevEvent.pipelineStage}→${currEvent.pipelineStage}`
      stageBreakdown.set(stage, (stageBreakdown.get(stage) || 0) + duration)
    }

    const breakdown = Array.from(stageBreakdown.entries())
      .map(([stage, duration]) => ({
        stage,
        duration,
        percentage: (duration / totalDuration) * 100,
      }))
      .sort((a, b) => b.duration - a.duration)

    const bottleneck = breakdown[0]

    // 生成建议
    const suggestions: string[] = []
    if (bottleneck.stage.includes('EX')) {
      suggestions.push('网络请求耗时较长，考虑优化后端性能或使用缓存')
    }
    if (bottleneck.stage.includes('SCH')) {
      suggestions.push('调度器等待时间较长，存在资源冲突')
    }
    if (bottleneck.percentage > 80) {
      suggestions.push(
        `${bottleneck.stage} 占总耗时 ${bottleneck.percentage.toFixed(1)}%，是主要瓶颈`
      )
    }

    return {
      instructionId,
      totalDuration,
      bottleneck,
      breakdown,
      suggestions,
    }
  }

  /**
   * 实时监控：获取最近 N 秒的统计
   */
  getRealtimeStats(windowSeconds: number = 5): {
    instructionsPerSecond: number
    avgLatency: number
    errorRate: number
    topInstructionTypes: Array<{ type: string; count: number }>
  } {
    const now = Date.now()
    const startTime = now - windowSeconds * 1000
    const recentEvents = cpuLogger.getEventsByTimeRange(startTime, now)

    const instructionIds = new Set(recentEvents.map((e) => e.instructionId))
    const failedIds = new Set(
      recentEvents
        .filter((e) => e.eventType === CPUEventType.INSTRUCTION_FAILED)
        .map((e) => e.instructionId)
    )

    const latencies = recentEvents.filter((e) => e.latency !== undefined).map((e) => e.latency!)

    const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length || 0

    const typeCount = new Map<string, number>()
    for (const event of recentEvents) {
      if (event.eventType === CPUEventType.INSTRUCTION_CREATED) {
        typeCount.set(event.instructionType, (typeCount.get(event.instructionType) || 0) + 1)
      }
    }

    const topInstructionTypes = Array.from(typeCount.entries())
      .map(([type, count]) => ({ type, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 5)

    return {
      instructionsPerSecond: instructionIds.size / windowSeconds,
      avgLatency,
      errorRate: failedIds.size / instructionIds.size || 0,
      topInstructionTypes,
    }
  }
}

export const cpuDebugger = new CPUDebugger()
```

---

## 📊 调试器 UI 增强

### 新增功能

1. **性能分析面板**
   - 指令类型的平均耗时、P95、P99
   - 流水线各阶段的耗时分布
   - 热力图：指令执行密度

2. **资源冲突可视化**
   - 依赖图：展示哪些指令在等待哪些资源
   - 冲突时间线：按时间展示资源冲突
   - 热点资源排行

3. **乐观更新监控**
   - 回滚率统计
   - 回滚原因分类
   - 回滚指令详情

4. **实时监控大屏**
   - 指令吞吐量（IPS）
   - 平均延迟
   - 错误率
   - 流水线利用率

5. **指令查询器**
   - 高级过滤（类型、状态、时间范围、延迟）
   - 时间旅行：回放指令执行过程
   - 导出功能：导出查询结果为 JSON/CSV

6. **诊断工具**
   - "为什么这个指令慢？" - 自动分析瓶颈
   - "为什么触发回滚？" - 回滚原因分析
   - "资源冲突链" - 追踪资源争用

### UI 组件示例

```vue
<!-- src/views/CPUDebugView.vue -->
<template>
  <div class="cpu-debug-view">
    <!-- 1. 实时监控 -->
    <section class="realtime-section">
      <h2>🎯 实时监控</h2>
      <div class="metrics-grid">
        <MetricCard
          title="指令吞吐量"
          :value="`${realtimeStats.instructionsPerSecond.toFixed(2)} IPS`"
        />
        <MetricCard title="平均延迟" :value="`${realtimeStats.avgLatency.toFixed(0)} ms`" />
        <MetricCard
          title="错误率"
          :value="`${(realtimeStats.errorRate * 100).toFixed(1)}%`"
          :variant="realtimeStats.errorRate > 0.05 ? 'danger' : 'success'"
        />
      </div>
    </section>

    <!-- 2. 性能分析 -->
    <section class="performance-section">
      <h2>📊 性能分析</h2>
      <InstructionPerformanceTable :data="performanceData" />
      <PipelineStageChart :data="stageBreakdown" />
    </section>

    <!-- 3. 资源冲突 -->
    <section class="conflict-section">
      <h2>⚠️ 资源冲突</h2>
      <ResourceConflictHeatmap :data="conflictHotspots" />
    </section>

    <!-- 4. 乐观更新 -->
    <section class="optimistic-section">
      <h2>🔄 乐观更新</h2>
      <OptimisticRollbackStats :data="rollbackStats" />
    </section>

    <!-- 5. 指令查询 -->
    <section class="query-section">
      <h2>🔍 指令查询</h2>
      <InstructionQueryBuilder @query="handleQuery" />
      <InstructionTraceViewer :traces="queryResults" />
    </section>
  </div>
</template>
```

---

## 🚀 实施计划

### Phase 1: 核心基础（1-2 天）

- [ ] 实现 `CPUEvent` 类型定义
- [ ] 实现 `CPUEventCollector`
- [ ] 实现 `CPULogger` 基础功能
- [ ] 在流水线各阶段集成事件采集

### Phase 2: 查询与分析（1-2 天）

- [ ] 完善 `CPULogger` 查询 API
- [ ] 实现性能分析函数
- [ ] 实现资源冲突分析
- [ ] 实现乐观更新分析

### Phase 3: 调试器（2-3 天）

- [ ] 实现 `CPUDebugger` 核心功能
- [ ] 实现诊断工具
- [ ] 实现时间旅行调试
- [ ] 实现数据导出

### Phase 4: UI 增强（2-3 天）

- [ ] 重构 `CPUDebugView.vue`
- [ ] 实现性能分析面板
- [ ] 实现资源冲突可视化
- [ ] 实现指令查询器
- [ ] 实现实时监控大屏

### Phase 5: 优化与文档（1-2 天）

- [ ] 性能优化（批量处理、索引优化）
- [ ] 编写使用文档
- [ ] 编写最佳实践指南
- [ ] 单元测试

---

## 📝 使用示例

### 基础使用

```typescript
// 1. 在流水线中集成事件采集
// src/cpu/stages/EX.ts
export class ExecuteStage {
  async execute(instruction: QueuedInstruction): Promise<void> {
    // 记录执行开始
    cpuEventCollector.emit({
      eventType: CPUEventType.INSTRUCTION_EXECUTING,
      instructionId: instruction.id,
      instructionType: instruction.type,
      correlationId: instruction.context.correlationId,
      pipelineStage: PipelineStage.EX,
      instructionStatus: InstructionStatus.EXECUTING,
    })

    // 应用乐观更新
    if (isa.optimistic?.enabled) {
      const snapshot = isa.optimistic.apply(instruction.payload, instruction.context)

      cpuEventCollector.onOptimisticApplied(
        instruction.id,
        instruction.type,
        instruction.context.correlationId,
        snapshot,
        { /* 变更内容 */ }
      )
    }

    // 执行网络请求
    cpuEventCollector.onNetworkRequest(
      instruction.id,
      instruction.type,
      instruction.context.correlationId,
      'PATCH',
      '/api/tasks/123'
    )

    const result = await executeRequest(...)

    cpuEventCollector.onNetworkResponse(
      instruction.id,
      instruction.type,
      instruction.context.correlationId,
      200,
      125, // 延迟
      1024 // 大小
    )
  }
}
```

### 调试器使用

```typescript
// 2. 在调试界面中使用
import { cpuDebugger, cpuLogger } from '@/cpu/logging'

// 查询最慢的指令
const slowest = cpuDebugger.getSlowestInstructions(10)
console.log('最慢的 10 条指令:', slowest)

// 诊断慢指令
const diagnosis = cpuDebugger.diagnoseSlowInstruction('instr_xxx')
console.log('瓶颈分析:', diagnosis.bottleneck)
console.log('建议:', diagnosis.suggestions)

// 分析性能
const perf = cpuLogger.analyzeInstructionPerformance('task.update')
console.log(`task.update 成功率: ${(perf.successRate * 100).toFixed(1)}%`)
console.log(`平均延迟: ${perf.avgLatency.toFixed(0)}ms (P95: ${perf.p95.toFixed(0)}ms)`)

// 分析资源冲突
const conflicts = cpuLogger.analyzeResourceConflicts()
console.log('冲突最多的资源:', conflicts[0])

// 导出数据
const data = cpuLogger.exportData({
  instructionType: 'schedule.update',
  timeRange: { start: Date.now() - 3600000, end: Date.now() },
})
console.log('导出数据:', data)
```

---

## 🎯 预期效果

1. **零依赖**：完全独立的日志系统，不依赖旧 logger
2. **高性能**：异步批量处理，对流水线性能影响 < 1%
3. **强大查询**：支持复杂的过滤和聚合查询
4. **智能分析**：自动检测瓶颈、异常和性能问题
5. **可视化**：丰富的图表和交互式调试界面
6. **可导出**：支持导出数据进行离线分析

---

## 📚 参考资料

- Chrome DevTools Performance API
- OpenTelemetry Tracing
- AWS X-Ray
- DataDog APM

---

**作者**: AI Assistant  
**版本**: v1.0  
**日期**: 2025-10-15

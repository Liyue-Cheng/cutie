/**
 * CPU 日志记录器
 *
 * 职责：
 * 1. 存储和索引所有 CPU 事件
 * 2. 提供强大的查询 API
 * 3. 自动分析和聚合
 */

import type { PipelineStage, InstructionStatus } from '../types'
import { CPUEventType, type CPUEvent } from './types'

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
    // 从索引中移除（简化实现）
    const instrEvents = this.eventsByInstruction.get(event.instructionId)
    if (instrEvents) {
      const index = instrEvents.indexOf(event)
      if (index > -1) instrEvents.splice(index, 1)
    }

    const corrEvents = this.eventsByCorrelation.get(event.correlationId)
    if (corrEvents) {
      const index = corrEvents.indexOf(event)
      if (index > -1) corrEvents.splice(index, 1)
    }

    const typeEvents = this.eventsByType.get(event.eventType)
    if (typeEvents) {
      const index = typeEvents.indexOf(event)
      if (index > -1) typeEvents.splice(index, 1)
    }
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


/**
 * CPU 调试器
 *
 * 职责：
 * 1. 提供调试器 API
 * 2. 支持时间旅行调试
 * 3. 支持指令重放
 */

import { cpuLogger } from './CPULogger'
import { CPUEventType, type CPUEvent } from './types'

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

    const withDuration = instructionTraces
      .map(([instructionId, events]) => {
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
      .filter((item) => item.duration > 0)

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


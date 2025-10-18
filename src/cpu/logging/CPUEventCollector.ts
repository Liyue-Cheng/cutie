/**
 * CPU 事件采集器
 *
 * 职责：
 * 1. 在流水线各阶段捕获事件
 * 2. 批量异步发送给 CPULogger
 * 3. 零侵入、零延迟
 */

import type { QueuedInstruction } from '../types'
import { PipelineStage, InstructionStatus } from '../types'
import { CPUEventType, type CPUEvent } from './types'

export class CPUEventCollector {
  private eventQueue: CPUEvent[] = []
  private flushInterval: number = 50 // 50ms 批量刷新
  private maxBatchSize: number = 100
  private enabled: boolean = true
  private flushTimer: number | null = null

  constructor() {
    this.startAutoFlush()
  }

  /**
   * 启动自动刷新
   */
  private startAutoFlush(): void {
    this.flushTimer = window.setInterval(() => {
      this.flush()
    }, this.flushInterval)
  }

  /**
   * 停止自动刷新
   */
  destroy(): void {
    if (this.flushTimer !== null) {
      clearInterval(this.flushTimer)
      this.flushTimer = null
    }
    this.flush() // 最后刷新一次
  }

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
      // 动态导入避免循环依赖
      import('./CPULogger').then(({ cpuLogger }) => {
        cpuLogger.ingestBatch(batch)
      })
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
      callSource: instruction.context.callSource, // 🔍 记录调用源
      payload: {
        instructionType: instruction.type,
        payload: instruction.payload,
        origin: 'user',
      },
    })
  }

  /**
   * 便捷方法：指令发射
   */
  onInstructionIssued(instruction: QueuedInstruction): void {
    this.emit({
      eventType: CPUEventType.INSTRUCTION_ISSUED,
      instructionId: instruction.id,
      instructionType: instruction.type,
      correlationId: instruction.context.correlationId,
      pipelineStage: PipelineStage.SCH,
      instructionStatus: InstructionStatus.ISSUED,
      payload: {},
    })
  }

  /**
   * 便捷方法：指令执行
   */
  onInstructionExecuting(instruction: QueuedInstruction): void {
    this.emit({
      eventType: CPUEventType.INSTRUCTION_EXECUTING,
      instructionId: instruction.id,
      instructionType: instruction.type,
      correlationId: instruction.context.correlationId,
      pipelineStage: PipelineStage.EX,
      instructionStatus: InstructionStatus.EXECUTING,
      payload: {},
    })
  }

  /**
   * 便捷方法：指令完成
   */
  onInstructionCommitted(instruction: QueuedInstruction): void {
    // 计算总执行时间
    const totalDuration = instruction.timestamps.WB && instruction.timestamps.IF
      ? instruction.timestamps.WB - instruction.timestamps.IF
      : undefined

    // 计算各阶段耗时
    const stageDurations = {
      ifToSch: instruction.timestamps.SCH && instruction.timestamps.IF
        ? instruction.timestamps.SCH - instruction.timestamps.IF : undefined,
      schToEx: instruction.timestamps.EX && instruction.timestamps.SCH
        ? instruction.timestamps.EX - instruction.timestamps.SCH : undefined,
      exToWb: instruction.timestamps.WB && instruction.timestamps.EX
        ? instruction.timestamps.WB - instruction.timestamps.EX : undefined,
    }

    this.emit({
      eventType: CPUEventType.INSTRUCTION_COMMITTED,
      instructionId: instruction.id,
      instructionType: instruction.type,
      correlationId: instruction.context.correlationId,
      pipelineStage: PipelineStage.WB,
      instructionStatus: InstructionStatus.COMMITTED,
      callSource: instruction.context.callSource,
      payload: {
        // 🔥 包含完整的指令执行信息
        originalPayload: instruction.payload,
        result: instruction.result,
        totalDuration,
        stageDurations,
        writeBackExecution: instruction.writeBackExecution,
        hasOptimisticUpdate: !!instruction.optimisticSnapshot,
        retryCount: instruction.context.retryCount,
        timestamps: instruction.timestamps,
      },
      latency: totalDuration,
      metadata: {
        tags: ['committed', 'success'],
        ...(instruction.writeBackExecution?.hasCommit && { hasCommit: true }),
        ...(instruction.optimisticSnapshot && { optimisticUpdate: true }),
      },
    })
  }

  /**
   * 便捷方法：指令失败
   */
  onInstructionFailed(instruction: QueuedInstruction, error: Error): void {
    // 计算总执行时间（失败时也需要记录）
    const totalDuration = instruction.timestamps.WB && instruction.timestamps.IF
      ? instruction.timestamps.WB - instruction.timestamps.IF
      : undefined

    // 计算各阶段耗时
    const stageDurations = {
      ifToSch: instruction.timestamps.SCH && instruction.timestamps.IF
        ? instruction.timestamps.SCH - instruction.timestamps.IF : undefined,
      schToEx: instruction.timestamps.EX && instruction.timestamps.SCH
        ? instruction.timestamps.EX - instruction.timestamps.SCH : undefined,
      exToWb: instruction.timestamps.WB && instruction.timestamps.EX
        ? instruction.timestamps.WB - instruction.timestamps.EX : undefined,
    }

    this.emit({
      eventType: CPUEventType.INSTRUCTION_FAILED,
      instructionId: instruction.id,
      instructionType: instruction.type,
      correlationId: instruction.context.correlationId,
      pipelineStage: PipelineStage.WB,
      instructionStatus: InstructionStatus.FAILED,
      callSource: instruction.context.callSource,
      payload: {
        // 🔥 包含完整的指令执行信息
        originalPayload: instruction.payload,
        error: error.message,
        stack: error.stack,
        totalDuration,
        stageDurations,
        writeBackExecution: instruction.writeBackExecution,
        hasOptimisticUpdate: !!instruction.optimisticSnapshot,
        rollbackExecuted: instruction.writeBackExecution?.rollbackExecuted,
        retryCount: instruction.context.retryCount,
        timestamps: instruction.timestamps,
      },
      latency: totalDuration,
      metadata: {
        tags: ['error', 'failure'],
        ...(instruction.writeBackExecution?.rollbackExecuted && { rollbackExecuted: true }),
        ...(instruction.optimisticSnapshot && { optimisticUpdate: true }),
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
      metadata: { tags: ['optimistic'] },
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
      pipelineStage: PipelineStage.EX,
      instructionStatus: InstructionStatus.EXECUTING,
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
        pipelineStage: PipelineStage.EX,
        instructionStatus: InstructionStatus.EXECUTING,
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

  /**
   * 生成事件 ID
   */
  private generateEventId(): string {
    return `evt_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
  }

  /**
   * 启用/禁用采集
   */
  enable(): void {
    this.enabled = true
  }

  disable(): void {
    this.enabled = false
  }

  isEnabled(): boolean {
    return this.enabled
  }
}

// 导出全局单例
export const cpuEventCollector = new CPUEventCollector()

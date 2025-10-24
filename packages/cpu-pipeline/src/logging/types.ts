/**
 * CPU 日志系统类型定义
 */

import type { PipelineStage, InstructionStatus } from '../types'

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
 * 控制台级别
 */
export enum ConsoleLevel {
  SILENT = 0, // 不输出任何内容
  MINIMAL = 1, // 只输出成功/失败
  NORMAL = 2, // 输出关键阶段
  VERBOSE = 3, // 输出所有细节
  DEBUG = 4, // 输出调试信息（包括 payload）
}

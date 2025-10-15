/**
 * 指令追踪类型定义
 */

import type { InstructionStatus } from '../types'

/**
 * 各阶段时间戳
 */
export interface PhaseTimestamps {
  IF: number
  SCH?: number
  EX?: number
  RES?: number
  WB?: number
}

/**
 * 指令追踪记录
 */
export interface InstructionTrace {
  /** 指令ID */
  instructionId: string
  /** 指令类型 */
  type: string
  /** 指令负载 */
  payload: any
  /** 关联ID */
  correlationId: string
  /** 各阶段时间戳 */
  timestamps: PhaseTimestamps
  /** 网络请求结果 */
  networkResult?: any
  /** 当前状态 */
  status: InstructionStatus
  /** 总耗时 (ms) */
  duration?: number
  /** 错误信息 */
  error?: Error
}


/**
 * CPU流水线核心类型定义
 */

/**
 * 流水线阶段
 */
export const PipelineStage = {
  IF: 'IF', // Instruction Fetch
  SCH: 'SCH', // Scheduler
  EX: 'EX', // Execute
  RES: 'RES', // Response
  WB: 'WB', // Write Back
} as const

export type PipelineStage = (typeof PipelineStage)[keyof typeof PipelineStage]

/**
 * 指令状态
 */
export const InstructionStatus = {
  PENDING: 'pending', // 在IF缓冲区等待调度
  ISSUED: 'issued', // 已发射到EX阶段
  EXECUTING: 'executing', // 正在执行
  RESPONDED: 'responded', // 已收到响应
  COMMITTED: 'committed', // 已提交完成
  FAILED: 'failed', // 执行失败
} as const

export type InstructionStatus = (typeof InstructionStatus)[keyof typeof InstructionStatus]

/**
 * 指令执行上下文
 */
export interface InstructionContext {
  /** 指令唯一ID */
  instructionId: string
  /** 关联ID（用于SSE去重等） */
  correlationId: string
  /** 时间戳 */
  timestamp: number
  /** 来源 */
  source: 'user' | 'system' | 'test'
  /** 重试次数 */
  retryCount: number
}

/**
 * 队列中的指令
 */
export interface QueuedInstruction<TPayload = any> {
  /** 指令ID */
  id: string
  /** 指令类型 */
  type: string
  /** 指令负载 */
  payload: TPayload
  /** 执行上下文 */
  context: InstructionContext
  /** 当前状态 */
  status: InstructionStatus
  /** 各阶段时间戳 */
  timestamps: {
    IF?: number
    SCH?: number
    EX?: number
    RES?: number
    WB?: number
  }
  /** 执行结果 */
  result?: any
  /** 错误信息 */
  error?: Error
}

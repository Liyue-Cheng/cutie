/**
 * InstructionTracker - 简化四级流水线追踪
 *
 * 专为 CommandBus + Store 架构设计：
 *
 * [IF] Instruction Fetch  - 指令已经获取
 * [EX] Execute           - 指令已发射 (commandBus.emit)
 * [RES] Result           - 收到结果 (HTTP/SSE响应)
 * [WB] Write Back        - 写回Store (state更新)
 */

import { logger, LogTags } from './logger'

/**
 * 流水线阶段
 */
export enum Stage {
  IF = 'IF',     // 指令获取
  EX = 'EX',     // 指令发射
  RES = 'RES',   // 收到结果
  WB = 'WB',     // 写回Store
}

/**
 * 结果来源类型
 */
export enum ResultSource {
  HTTP = 'HTTP',           // HTTP API 响应
  SSE = 'SSE',            // SSE 事件
  CACHE = 'CACHE',        // 缓存命中
  LOCAL = 'LOCAL',        // 本地计算
}

/**
 * 执行状态
 */
export enum Status {
  SUCCESS = 'SUCCESS',
  FAILED = 'FAILED',
  TIMEOUT = 'TIMEOUT',
}

/**
 * 阶段记录
 */
interface StageRecord {
  stage: Stage
  timestamp: number
  status: Status
  duration?: number
  details?: Record<string, any>
}

/**
 * 指令追踪器
 */
export class InstructionTracker {
  private instructionId: string
  private operation: string
  private startTime: number
  private stages: StageRecord[] = []
  private currentStage: Stage = Stage.IF

  constructor(operation: string) {
    this.instructionId = `cmd-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`
    this.operation = operation
    this.startTime = Date.now()

    this.logStage('指令追踪开始')
  }

  /**
   * [IF] 指令已经获取
   */
  fetch(input: Record<string, any>): this {
    this.currentStage = Stage.IF

    this.logStage('指令已获取', {
      input,
      inputSize: JSON.stringify(input).length
    })

    return this
  }

  /**
   * [EX] 指令已发射
   */
  execute(command: string, payload?: Record<string, any>): this {
    this.currentStage = Stage.EX

    this.logStage('指令已发射', {
      command,
      payload,
      via: 'commandBus.emit'
    })

    return this
  }

  /**
   * [RES] 收到结果
   */
  result(
    source: ResultSource,
    data: Record<string, any>,
    status: Status = Status.SUCCESS,
    details?: Record<string, any>
  ): this {
    this.currentStage = Stage.RES

    this.logStage('收到结果', {
      source,
      status,
      dataSize: JSON.stringify(data).length,
      hasError: status === Status.FAILED,
      ...details
    })

    return this
  }

  /**
   * [WB] 写回Store
   */
  writeBack(
    stores: string[],
    mutations: string[],
    effects?: string[]
  ): this {
    this.currentStage = Stage.WB

    // 计算总耗时
    const totalDuration = Date.now() - this.startTime

    this.logStage('写回Store完成', {
      stores,
      mutations,
      effects,
      totalDuration,
      stageCount: this.stages.length
    })

    // 输出执行摘要
    this.outputSummary(totalDuration)

    return this
  }

  /**
   * 错误处理
   */
  error(error: Error, source?: string): this {
    const status = Status.FAILED

    this.logStage('执行失败', {
      error: error.message,
      errorName: error.name,
      source,
      currentStage: this.currentStage
    }, status)

    // 输出错误摘要
    this.outputSummary(Date.now() - this.startTime, error)

    return this
  }

  /**
   * 超时处理
   */
  timeout(): this {
    const status = Status.TIMEOUT

    this.logStage('执行超时', {
      currentStage: this.currentStage,
      duration: Date.now() - this.startTime
    }, status)

    this.outputSummary(Date.now() - this.startTime)
    return this
  }

  /**
   * 获取指令ID
   */
  getInstructionId(): string {
    return this.instructionId
  }

  /**
   * 记录阶段日志
   */
  private logStage(
    message: string,
    details?: Record<string, any>,
    status: Status = Status.SUCCESS
  ): void {
    const timestamp = Date.now()
    const duration = this.stages.length > 0
      ? timestamp - this.stages[this.stages.length - 1].timestamp
      : 0

    // 记录阶段
    this.stages.push({
      stage: this.currentStage,
      timestamp,
      status,
      duration,
      details
    })

    // 输出日志（带计时信息）
    const totalDuration = timestamp - this.startTime
    const stageDurationText = duration > 0 ? `+${duration}ms` : '+0ms'
    const totalDurationText = `${totalDuration}ms`

    const logMessage = `[${this.instructionId}] [${this.operation}] ${this.currentStage}: ${message} (${stageDurationText} / total: ${totalDurationText})`
    const logContext = {
      instructionId: this.instructionId,
      operation: this.operation,
      stage: this.currentStage,
      status,
      stageDuration: duration,
      totalDuration,
      ...details
    }

    // ✅ 禁用旧的日志输出，现在由 CPUConsole 负责
    // if (status === Status.FAILED) {
    //   logger.error(LogTags.INSTRUCTION_TRACKER, logMessage, new Error(details?.error || 'Unknown error'), logContext)
    // } else if (status === Status.TIMEOUT) {
    //   logger.warn(LogTags.INSTRUCTION_TRACKER, logMessage, logContext)
    // } else {
    //   logger.info(LogTags.INSTRUCTION_TRACKER, logMessage, logContext)
    // }
  }

  /**
   * 输出执行摘要
   */
  private outputSummary(totalDuration: number, error?: Error): void {
    const pipeline = this.stages.map(stage => ({
      stage: stage.stage,
      duration: stage.duration,
      status: stage.status
    }))

    const summary = {
      instructionId: this.instructionId,
      operation: this.operation,
      totalDuration,
      stagesCompleted: this.stages.length,
      pipeline,
      success: !error && this.stages.every(s => s.status === Status.SUCCESS)
    }

    // ✅ 禁用旧的摘要输出，现在由 CPUConsole 负责
    // if (error) {
    //   logger.error(
    //     LogTags.INSTRUCTION_TRACKER,
    //     `指令执行摘要 [${this.instructionId}] - 失败`,
    //     error,
    //     summary
    //   )
    // } else {
    //   logger.info(LogTags.INSTRUCTION_TRACKER, `指令执行摘要 [${this.instructionId}] - 完成`, summary)
    // }
  }
}

/**
 * 创建指令追踪器
 */
export function createTracker(operation: string): InstructionTracker {
  return new InstructionTracker(operation)
}

/**
 * 快捷方法：追踪CommandBus操作
 */
export function trackCommand(
  command: string,
  input: Record<string, any>
): InstructionTracker {
  return createTracker(`command.${command}`)
    .fetch(input)
    .execute(command, input)
}

/**
 * 快捷方法：追踪Store加载操作
 */
export function trackLoad(
  storeName: string,
  method: string,
  params?: Record<string, any>
): InstructionTracker {
  return createTracker(`${storeName}.${method}`)
    .fetch(params || {})
    .execute(`${storeName}_DMA.${method}`, params)
}

/**
 * 装饰器：自动追踪方法执行
 */
export function Track(operation?: string) {
  return function(target: any, propertyName: string, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value
    const op = operation || `${target.constructor.name}.${propertyName}`

    descriptor.value = async function(...args: any[]) {
      const tracker = createTracker(op)

      try {
        // [IF] 获取指令
        tracker.fetch({ args })

        // [EX] 发射指令
        tracker.execute(propertyName, { method: propertyName })

        // 执行原方法
        const result = await originalMethod.apply(this, args)

        // [RES] 收到结果
        tracker.result(ResultSource.LOCAL, { result })

        // [WB] 写回（这里需要根据实际情况调整）
        tracker.writeBack([target.constructor.name], [propertyName])

        return result
      } catch (error) {
        tracker.error(error as Error, propertyName)
        throw error
      }
    }

    return descriptor
  }
}
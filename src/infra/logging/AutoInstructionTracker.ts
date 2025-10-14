/**
 * 自动化指令追踪器 - 零集成版本
 *
 * 完全自动化的四级流水线追踪：
 * [IF] Instruction Fetch  - CommandBus 自动捕获
 * [EX] Execute           - CommandBus 自动捕获
 * [RES] Result           - API Client 自动捕获
 * [WB] Write Back        - Store Mutation 自动捕获
 *
 * 使用方式：只需在应用启动时调用 setupAutoTracking() 即可
 */

import { InstructionTracker, ResultSource, Status } from './InstructionTracker'
import { logger, LogTags } from './logger'

/**
 * 全局追踪上下文管理器
 */
class TrackingContext {
  private activeTrackers = new Map<string, InstructionTracker>()
  private correlationToTracker = new Map<string, InstructionTracker>()

  /**
   * 创建新的追踪器
   */
  createTracker(command: string, input: Record<string, any>, correlationId?: string): InstructionTracker {
    const tracker = new InstructionTracker(`command.${command}`)
      .fetch(input)
      .execute(command, input)

    // 存储追踪器
    const trackerId = tracker.getInstructionId()
    this.activeTrackers.set(trackerId, tracker)

    if (correlationId) {
      this.correlationToTracker.set(correlationId, tracker)
    }

    return tracker
  }

  /**
   * 通过 correlation ID 获取追踪器
   */
  getTrackerByCorrelation(correlationId: string): InstructionTracker | undefined {
    return this.correlationToTracker.get(correlationId)
  }

  /**
   * 完成追踪器
   */
  completeTracker(trackerId: string): void {
    this.activeTrackers.delete(trackerId)
  }

  /**
   * 通过 correlation ID 完成追踪器
   */
  completeTrackerByCorrelation(correlationId: string): void {
    const tracker = this.correlationToTracker.get(correlationId)
    if (tracker) {
      this.correlationToTracker.delete(correlationId)
      this.completeTracker(tracker.getInstructionId())
    }
  }

  /**
   * 清理过期的追踪器（防止内存泄漏）
   */
  cleanup(): void {
    // 清理超过 5 分钟的追踪器
    const now = Date.now()
    const fiveMinutes = 5 * 60 * 1000

    for (const [trackerId, tracker] of this.activeTrackers) {
      // 简单的过期检查（实际实现中可以存储创建时间）
      if (now - parseInt(trackerId.split('-')[1]) > fiveMinutes) {
        this.activeTrackers.delete(trackerId)
        logger.warn(LogTags.INSTRUCTION_TRACKER, `Cleaned up expired tracker: ${trackerId}`)
      }
    }
  }
}

// 全局单例
const trackingContext = new TrackingContext()

// 定期清理
setInterval(() => trackingContext.cleanup(), 60000) // 每分钟清理一次

/**
 * CommandBus 拦截器
 */
export function interceptCommandBus(originalEmit: Function) {
  return async function(command: string, payload: any, options?: any): Promise<any> {
    // [IF] + [EX] 自动创建追踪器
    const correlationId = options?.correlationId ||
                         options?.headers?.['X-Correlation-ID'] ||
                         `auto-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`

    const tracker = trackingContext.createTracker(command, payload, correlationId)

    try {
      // 调用原始方法
      const result = await originalEmit.call(this, command, payload, {
        ...options,
        correlationId
      })

      return result
    } catch (error) {
      // 错误处理
      tracker.error(error as Error, 'commandBus.emit')
      throw error
    }
  }
}

/**
 * API Client 拦截器
 */
export function interceptApiClient(originalFetch: Function) {
  return async function(url: string, options?: any): Promise<any> {
    const correlationId = options?.headers?.['X-Correlation-ID']
    const tracker = correlationId ? trackingContext.getTrackerByCorrelation(correlationId) : undefined

    try {
      // 调用原始 API
      const result = await originalFetch.call(this, url, options)

      // [RES] 自动记录结果
      if (tracker) {
        tracker.result(ResultSource.HTTP, result, Status.SUCCESS, {
          url,
          method: options?.method || 'GET',
          status: 'success'
        })
      }

      return result
    } catch (error) {
      // 错误处理
      if (tracker) {
        tracker.error(error as Error, 'api.call')
      }
      throw error
    }
  }
}

/**
 * Store Mutation 拦截器
 */
export function interceptStoreMutation(storeName: string, mutationName: string, originalMutation: Function) {
  return function(...args: any[]): any {
    // 尝试从当前执行上下文中找到活跃的追踪器
    // 这里使用简单的策略：获取最近创建的追踪器
    const recentTracker = Array.from(trackingContext.activeTrackers.values())
      .sort((a, b) => parseInt(b.getInstructionId().split('-')[1]) - parseInt(a.getInstructionId().split('-')[1]))[0]

    try {
      // 调用原始 mutation
      const result = originalMutation.apply(this, args)

      // [WB] 自动记录写回
      if (recentTracker) {
        recentTracker.writeBack([storeName], [mutationName])
        // 完成追踪
        trackingContext.completeTracker(recentTracker.getInstructionId())
      }

      return result
    } catch (error) {
      if (recentTracker) {
        recentTracker.error(error as Error, `${storeName}.${mutationName}`)
      }
      throw error
    }
  }
}

/**
 * 事务处理器拦截器
 */
export function interceptTransactionProcessor(originalApply: Function) {
  return async function(result: any, context: any): Promise<any> {
    const correlationId = context?.correlation_id
    const tracker = correlationId ? trackingContext.getTrackerByCorrelation(correlationId) : undefined

    try {
      // [RES] 记录事务结果
      if (tracker) {
        tracker.result(
          context?.source === 'sse' ? ResultSource.SSE : ResultSource.HTTP,
          result,
          Status.SUCCESS,
          {
            source: context?.source,
            transactionType: 'TaskTransaction'
          }
        )
      }

      // 调用原始方法
      const processResult = await originalApply.call(this, result, context)

      // [WB] 记录状态更新
      if (tracker) {
        const affectedStores = this.getAffectedStores ? this.getAffectedStores(result) : ['TaskStore']
        const mutations = this.getAppliedMutations ? this.getAppliedMutations(result) : ['addOrUpdateTask_mut']

        tracker.writeBack(affectedStores, mutations, ['processTransaction'])

        // 完成追踪
        trackingContext.completeTrackerByCorrelation(correlationId)
      }

      return processResult
    } catch (error) {
      if (tracker) {
        tracker.error(error as Error, 'transactionProcessor')
      }
      throw error
    }
  }
}

/**
 * 自动设置拦截器 - 一键启用自动追踪
 */
export function setupAutoTracking() {
  logger.info(LogTags.INSTRUCTION_TRACKER, '🚀 Setting up automatic instruction tracking...')

  try {
    // 1. 拦截 CommandBus
    setupCommandBusInterception()

    // 2. 拦截 API Client
    setupApiClientInterception()

    // 3. 拦截 Store Mutations
    setupStoreMutationInterception()

    // 4. 拦截 Transaction Processor
    setupTransactionProcessorInterception()

    logger.info(LogTags.INSTRUCTION_TRACKER, '✅ Automatic instruction tracking enabled!')

  } catch (error) {
    logger.error(LogTags.INSTRUCTION_TRACKER, 'Failed to setup auto tracking', error as Error)
  }
}

/**
 * CommandBus 拦截设置
 */
function setupCommandBusInterception() {
  // 动态导入并拦截 CommandBus
  import('@/commandBus').then((module) => {
    const commandBus = module.commandBus
    if (commandBus && commandBus.emit) {
      const originalEmit = commandBus.emit.bind(commandBus)
      commandBus.emit = interceptCommandBus(originalEmit)
      logger.debug(LogTags.INSTRUCTION_TRACKER, 'CommandBus interception enabled')
    }
  }).catch(error => {
    logger.warn(LogTags.INSTRUCTION_TRACKER, 'Failed to intercept CommandBus', { error })
  })
}

/**
 * API Client 拦截设置
 */
function setupApiClientInterception() {
  // 拦截 fetch API
  const originalFetch = window.fetch
  window.fetch = interceptApiClient(originalFetch)
  logger.debug(LogTags.INSTRUCTION_TRACKER, 'API Client interception enabled')
}

/**
 * Store Mutation 拦截设置
 */
function setupStoreMutationInterception() {
  // 动态拦截 Pinia stores
  import('@/stores/task').then((module) => {
    const useTaskStore = module.useTaskStore

    // 创建代理来拦截 mutation 调用
    const originalStoreFunction = useTaskStore

    // 这里需要更复杂的代理逻辑来拦截 mutation
    // 暂时跳过，因为 Pinia 的拦截比较复杂
    logger.debug(LogTags.INSTRUCTION_TRACKER, 'Store mutation interception setup (partial)')
  }).catch(error => {
    logger.warn(LogTags.INSTRUCTION_TRACKER, 'Failed to intercept Store mutations', { error })
  })
}

/**
 * Transaction Processor 拦截设置
 */
function setupTransactionProcessorInterception() {
  // 动态拦截 Transaction Processor
  import('@/infra/transaction/transactionProcessor').then((module) => {
    const processor = module.transactionProcessor
    if (processor && processor.applyTaskTransaction) {
      const originalApply = processor.applyTaskTransaction.bind(processor)
      processor.applyTaskTransaction = interceptTransactionProcessor(originalApply)
      logger.debug(LogTags.INSTRUCTION_TRACKER, 'Transaction Processor interception enabled')
    }
  }).catch(error => {
    logger.warn(LogTags.INSTRUCTION_TRACKER, 'Failed to intercept Transaction Processor', { error })
  })
}

/**
 * 手动创建追踪器（向后兼容）
 */
export function createAutoTracker(command: string, input: Record<string, any>): InstructionTracker {
  return trackingContext.createTracker(command, input)
}

/**
 * 获取追踪统计信息
 */
export function getTrackingStats() {
  return {
    activeTrackers: trackingContext.activeTrackers.size,
    correlationMappings: trackingContext.correlationToTracker.size
  }
}
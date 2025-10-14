/**
 * 改进版自动化指令追踪器 - 清洁版
 *
 * 特点：
 * 1. 完全自动追踪四级流水线
 * 2. 干净的日志输出，最小化噪音
 * 3. 准确的阶段关联
 */

import { createTracker, ResultSource, Status } from './InstructionTracker'
import { logger, LogTags } from './logger'

// 全局追踪状态
const globalTracking = {
  trackers: new Map<string, any>(),
  correlationToTracker: new Map<string, any>(),
  enabled: false
}

/**
 * 一键启用自动追踪 - 在 main.ts 中调用
 */
export async function enableAutoTracking() {
  if (globalTracking.enabled) return

  logger.info(LogTags.INSTRUCTION_TRACKER, '🚀 Enabling automatic instruction tracking...')

  try {
    // 1. 拦截 CommandBus
    await interceptCommandBus()

    // 2. 拦截全局 fetch
    interceptGlobalFetch()

    // 3. 拦截 transaction processor
    await interceptTransactionProcessor()

    globalTracking.enabled = true
    logger.info(LogTags.INSTRUCTION_TRACKER, '✅ Automatic instruction tracking enabled!')

  } catch (error) {
    logger.error(LogTags.INSTRUCTION_TRACKER, 'Failed to enable auto tracking', error as Error)
  }
}

/**
 * 拦截 CommandBus - 自动创建和管理追踪器
 */
async function interceptCommandBus() {
  const { commandBus } = await import('@/commandBus')

  const originalEmit = commandBus.emit.bind(commandBus)

  commandBus.emit = async function(command: string, payload: any): Promise<any> {
    // [IF] + [EX] 创建追踪器
    const tracker = createTracker(`command.${command}`)
      .fetch(payload || {})
      .execute(command, payload || {})

    const trackerId = tracker.getInstructionId()

    try {
      // 存储追踪器实例
      globalTracking.trackers.set(trackerId, tracker)

      // 调用原始方法
      const result = await originalEmit(command, payload)

      return result

    } catch (error) {
      tracker.error(error as Error, 'commandBus.emit')
      // 清理失败的追踪器
      globalTracking.trackers.delete(trackerId)
      throw error
    }
  }

  logger.debug(LogTags.INSTRUCTION_TRACKER, 'CommandBus interception enabled')
}

/**
 * 拦截全局 fetch - 自动记录 API 响应
 */
function interceptGlobalFetch() {
  const originalFetch = window.fetch

  window.fetch = async function(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
    // 检查是否有 correlation ID
    const headers = init?.headers || {}
    const correlationId = getCorrelationId(headers)

    try {
      const response = await originalFetch(input, init)

      // [RES] 记录 API 响应
      const tracker = findTrackerByCorrelation(correlationId) || findMostRecentTracker()

      if (tracker) {
        if (response.ok) {
          tracker.result(ResultSource.HTTP, {
            status: response.status,
            url: input.toString().split('?')[0] // 移除查询参数以减少噪音
          }, Status.SUCCESS, {
            method: init?.method || 'GET'
          })
        } else {
          tracker.result(ResultSource.HTTP, {
            status: response.status,
            error: true
          }, Status.FAILED)
        }

        // 存储 correlation ID 映射
        if (correlationId) {
          globalTracking.correlationToTracker.set(correlationId, tracker)
        }
      }

      return response

    } catch (error) {
      const tracker = findTrackerByCorrelation(correlationId) || findMostRecentTracker()
      if (tracker) {
        tracker.error(error as Error, 'api.fetch')
      }
      throw error
    }
  }

  logger.debug(LogTags.INSTRUCTION_TRACKER, 'Global fetch interception enabled')
}

/**
 * 拦截 transaction processor - 自动记录状态更新
 */
async function interceptTransactionProcessor() {
  try {
    const { transactionProcessor } = await import('@/infra/transaction/transactionProcessor')

    const originalApply = transactionProcessor.applyTaskTransaction.bind(transactionProcessor)

    transactionProcessor.applyTaskTransaction = async function(result: any, context: any): Promise<any> {
      const correlationId = context?.correlation_id
      const tracker = findTrackerByCorrelation(correlationId) || findMostRecentTracker()

      if (tracker) {
        // [WB] 记录状态更新（简化版本）
        tracker.writeBack(['TaskStore'], ['transaction'], ['updateUI'])

        // 完成追踪并清理
        completeTracker(tracker, correlationId)
      }

      // 调用原始方法
      return await originalApply(result, context)
    }

    logger.debug(LogTags.INSTRUCTION_TRACKER, 'Transaction processor interception enabled')
  } catch (error) {
    logger.warn(LogTags.INSTRUCTION_TRACKER, 'Failed to intercept transaction processor', { error })
  }
}

/**
 * 工具函数：获取 correlation ID
 */
function getCorrelationId(headers: any): string | undefined {
  if (!headers) return undefined

  if (headers instanceof Headers) {
    return headers.get('X-Correlation-ID') || undefined
  }

  return headers['X-Correlation-ID'] || headers['x-correlation-id'] || undefined
}

/**
 * 通过 correlation ID 查找追踪器
 */
function findTrackerByCorrelation(correlationId?: string) {
  if (!correlationId) return null
  return globalTracking.correlationToTracker.get(correlationId) || null
}

/**
 * 找到最新的追踪器（备用方案）
 */
function findMostRecentTracker() {
  if (globalTracking.trackers.size === 0) return null

  // 按时间戳排序，获取最新的
  const trackers = Array.from(globalTracking.trackers.values())
  return trackers.sort((a, b) => {
    const aTime = parseInt(a.getInstructionId().split('-')[1])
    const bTime = parseInt(b.getInstructionId().split('-')[1])
    return bTime - aTime
  })[0] || null
}

/**
 * 完成追踪器并清理
 */
function completeTracker(tracker: any, correlationId?: string) {
  const trackerId = tracker.getInstructionId()
  globalTracking.trackers.delete(trackerId)

  if (correlationId) {
    globalTracking.correlationToTracker.delete(correlationId)
  }
}

/**
 * 获取追踪统计信息
 */
export function getTrackingStats() {
  return {
    enabled: globalTracking.enabled,
    activeTrackers: globalTracking.trackers.size,
    correlationMappings: globalTracking.correlationToTracker.size
  }
}

/**
 * 禁用自动追踪
 */
export function disableAutoTracking() {
  globalTracking.enabled = false
  globalTracking.trackers.clear()
  globalTracking.correlationToTracker.clear()
  logger.info(LogTags.INSTRUCTION_TRACKER, 'Automatic instruction tracking disabled')
}

// 定期清理过期的追踪器（防止内存泄漏）
setInterval(() => {
  const now = Date.now()
  const fiveMinutes = 5 * 60 * 1000

  // 清理过期的追踪器
  for (const [trackerId, tracker] of globalTracking.trackers) {
    const timestamp = parseInt(trackerId.split('-')[1])
    if (now - timestamp > fiveMinutes) {
      globalTracking.trackers.delete(trackerId)
    }
  }

  // 清理过期的 correlation 映射
  for (const [correlationId, tracker] of globalTracking.correlationToTracker) {
    const trackerId = tracker.getInstructionId()
    const timestamp = parseInt(trackerId.split('-')[1])
    if (now - timestamp > fiveMinutes) {
      globalTracking.correlationToTracker.delete(correlationId)
    }
  }
}, 60000) // 每分钟清理一次
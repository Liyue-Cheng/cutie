import { ref } from 'vue'
import { logger, LogTags } from '@/services/logger'

/**
 * Correlation ID 追踪器
 * 
 * 用于追踪 HTTP 请求和 SSE 事件的关联关系，实现：
 * - 请求去重（避免重复处理自己触发的事件）
 * - 性能监控（记录各阶段时间戳）
 * - 自动清理（防止内存泄漏）
 */

export interface PerformanceTimer {
  start: number
  httpSent: number
  httpReceived?: number
  sseReceived?: number
  sideEffectsCompleted?: number
}

/**
 * 创建 Correlation 追踪器
 */
export function createCorrelationTracker() {
  /**
   * 待处理的 Correlation IDs（用于去重和请求追踪）
   */
  const pendingCorrelations = ref(new Set<string>())
  
  /**
   * 性能计时器：记录每个请求的各阶段时间戳
   */
  const performanceTimers = ref(new Map<string, PerformanceTimer>())
  
  /**
   * 生成新的 correlation ID 并开始追踪
   * @param operationName 操作名称（用于日志）
   * @returns correlation ID
   */
  function startTracking(operationName: string): string {
    const correlationId = crypto.randomUUID()
    pendingCorrelations.value.add(correlationId)
    
    const startTime = performance.now()
    performanceTimers.value.set(correlationId, {
      start: startTime,
      httpSent: 0,
    })
    
    logger.debug(LogTags.PERF, 'Operation started', {
      operationName,
      correlationId,
    })
    return correlationId
  }
  
  /**
   * 记录 HTTP 请求发送时间
   */
  function markHttpSent(correlationId: string, operationName: string): void {
    const timer = performanceTimers.value.get(correlationId)
    if (!timer) return
    
    const httpSentTime = performance.now()
    timer.httpSent = httpSentTime
    
    const preparationTime = httpSentTime - timer.start
    logger.debug(LogTags.PERF, 'HTTP request sent', {
      correlationId,
      preparationTime: `${preparationTime.toFixed(2)}ms`,
    })
  }
  
  /**
   * 记录 HTTP 响应接收时间
   */
  function markHttpReceived(correlationId: string, operationName: string): void {
    const timer = performanceTimers.value.get(correlationId)
    if (!timer) return
    
    const httpReceivedTime = performance.now()
    timer.httpReceived = httpReceivedTime
    
    const httpRoundtrip = httpReceivedTime - timer.httpSent
    const totalSoFar = httpReceivedTime - timer.start
    logger.debug(LogTags.PERF, 'HTTP response received', {
      correlationId,
      httpRoundtrip: `${httpRoundtrip.toFixed(2)}ms`,
      totalSoFar: `${totalSoFar.toFixed(2)}ms`,
    })
  }
  
  /**
   * 记录 SSE 事件接收时间
   */
  function markSseReceived(correlationId: string, operationName: string): void {
    const timer = performanceTimers.value.get(correlationId)
    if (!timer) return
    
    const sseReceivedTime = performance.now()
    timer.sseReceived = sseReceivedTime
    
    const sseDelay = sseReceivedTime - (timer.httpReceived || timer.httpSent)
    const totalSoFar = sseReceivedTime - timer.start
    logger.debug(LogTags.PERF, 'SSE event received', {
      correlationId,
      sseDelay: `${sseDelay.toFixed(2)}ms`,
      totalSoFar: `${totalSoFar.toFixed(2)}ms`,
    })
  }
  
  /**
   * 记录副作用处理完成时间并输出总结
   */
  function markSideEffectsCompleted(correlationId: string, operationName: string): void {
    const timer = performanceTimers.value.get(correlationId)
    if (!timer) return
    
    const completedTime = performance.now()
    timer.sideEffectsCompleted = completedTime
    
    const sideEffectsDuration = completedTime - (timer.sseReceived || timer.httpReceived || timer.httpSent)
    const totalDuration = completedTime - timer.start
    
    logger.debug(LogTags.PERF, 'Side effects completed', {
      correlationId,
      sideEffectsDuration: `${sideEffectsDuration.toFixed(2)}ms`,
      totalDuration: `${totalDuration.toFixed(2)}ms`,
    })
    
    // 输出详细总结
    logger.info(LogTags.PERF, `${operationName.toUpperCase()} performance summary`, {
      correlationId,
      preparation: `${(timer.httpSent - timer.start).toFixed(2)}ms`,
      httpRoundtrip: `${((timer.httpReceived || 0) - timer.httpSent).toFixed(2)}ms`,
      sseDelay: `${((timer.sseReceived || 0) - (timer.httpReceived || timer.httpSent)).toFixed(2)}ms`,
      sideEffects: `${sideEffectsDuration.toFixed(2)}ms`,
      total: `${totalDuration.toFixed(2)}ms`,
    })
  }
  
  /**
   * 输出无副作用操作的总结
   */
  function markCompleted(correlationId: string, operationName: string): void {
    const timer = performanceTimers.value.get(correlationId)
    if (!timer) return
    
    const completedTime = timer.sseReceived || timer.httpReceived || performance.now()
    const totalDuration = completedTime - timer.start
    
    logger.info(LogTags.PERF, `${operationName.toUpperCase()} performance summary (no side effects)`, {
      correlationId,
      preparation: `${(timer.httpSent - timer.start).toFixed(2)}ms`,
      httpRoundtrip: `${((timer.httpReceived || 0) - timer.httpSent).toFixed(2)}ms`,
      sseDelay: `${((timer.sseReceived || 0) - (timer.httpReceived || timer.httpSent)).toFixed(2)}ms`,
      total: `${totalDuration.toFixed(2)}ms`,
    })
  }
  
  /**
   * 检查是否是自己触发的操作
   */
  function isOwnOperation(correlationId?: string): boolean {
    return correlationId ? pendingCorrelations.value.has(correlationId) : false
  }
  
  /**
   * 完成追踪并清理资源
   * @param correlationId 
   * @param delayMs 延迟清理时间（毫秒），默认 10 秒
   */
  function finishTracking(correlationId: string, delayMs = 10000): void {
    // 立即从待处理列表中移除
    pendingCorrelations.value.delete(correlationId)
    
    // 延迟清理性能计时器（防止内存泄漏）
    setTimeout(() => {
      performanceTimers.value.delete(correlationId)
    }, delayMs)
  }
  
  /**
   * 清理失败的追踪（用于异常情况）
   */
  function cleanupFailedTracking(correlationId: string): void {
    pendingCorrelations.value.delete(correlationId)
    performanceTimers.value.delete(correlationId)
  }
  
  return {
    pendingCorrelations,
    performanceTimers,
    startTracking,
    markHttpSent,
    markHttpReceived,
    markSseReceived,
    markSideEffectsCompleted,
    markCompleted,
    isOwnOperation,
    finishTracking,
    cleanupFailedTracking,
  }
}
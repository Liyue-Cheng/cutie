/**
 * INT: 中断管理器（Interrupt Controller）
 *
 * 职责：
 * 1. 接收所有外部事件（SSE、WebSocket、轮询等）
 * 2. 去重检查（基于本机操作表）
 * 3. 根据事件类型分发给对应的 handler
 * 4. 统一的中断入口点
 *
 * 架构：
 * [SSE] → INT.dispatch(event) → [去重] → [分发] → [Handler]
 * [WS]  → INT.dispatch(event) → [去重] → [分发] → [Handler]
 * [WB]  → INT.register(correlationId) → [记录本机操作]
 */

import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 中断类型
 */
export const InterruptType = {
  SSE: 'sse' as const, // Server-Sent Events
  WEBSOCKET: 'ws' as const, // WebSocket
  POLLING: 'polling' as const, // 长轮询
} as const

export type InterruptType = (typeof InterruptType)[keyof typeof InterruptType]

/**
 * 中断事件（标准化格式）
 */
export interface InterruptEvent {
  type: InterruptType // 中断类型
  eventType: string // 事件类型（如 task.completed）
  correlationId?: string
  eventId?: string
  payload: any
  timestamp: number
}

/**
 * 中断处理器（回调函数）
 */
export type InterruptEventHandler = (event: InterruptEvent) => void | Promise<void>

/**
 * 中断表条目
 */
interface InterruptEntry {
  correlationId: string
  timestamp: number
  instruction: {
    type: string
    payload: any
  }
}

/**
 * 中断管理器（Controller）
 */
export class InterruptHandler {
  // 中断表：记录本机发起的指令
  private interruptTable = new Map<string, InterruptEntry>()

  // 事件处理器映射：eventType → handlers[]
  private handlers = new Map<string, InterruptEventHandler[]>()

  // TTL：中断表条目的生存时间（10秒）
  private readonly TTL = 10000

  // 清理定时器
  private cleanupTimer: number | null = null

  constructor() {
    // 每5秒清理一次过期条目
    this.cleanupTimer = window.setInterval(() => {
      this.cleanup()
    }, 5000)
  }

  /**
   * 注册本机发起的指令
   *
   * 在 WB 阶段完成后调用
   */
  register(correlationId: string, instruction: { type: string; payload: any }): void {
    this.interruptTable.set(correlationId, {
      correlationId,
      timestamp: Date.now(),
      instruction,
    })

    logger.debug(LogTags.SYSTEM_PIPELINE, 'INT: 注册中断', {
      correlationId,
      type: instruction.type,
      tableSize: this.interruptTable.size,
    })
  }

  /**
   * 注册事件处理器
   *
   * @param eventType 事件类型（如 task.completed）
   * @param handler 处理器函数
   */
  on(eventType: string, handler: InterruptEventHandler): void {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, [])
    }
    this.handlers.get(eventType)!.push(handler)
    logger.debug(LogTags.SYSTEM_PIPELINE, 'INT: 注册事件处理器', {
      eventType,
      handlerCount: this.handlers.get(eventType)!.length,
    })
  }

  /**
   * 取消注册事件处理器
   */
  off(eventType: string, handler: InterruptEventHandler): void {
    const handlers = this.handlers.get(eventType)
    if (handlers) {
      const index = handlers.indexOf(handler)
      if (index > -1) {
        handlers.splice(index, 1)
      }
    }
  }

  /**
   * 分发中断事件（统一入口）
   *
   * 所有外部事件（SSE、WebSocket 等）都通过此方法进入系统
   */
  dispatch(event: InterruptEvent): void {
    const { correlationId, type, eventType } = event

    logger.debug(LogTags.SYSTEM_PIPELINE, 'INT: 收到中断', {
      type,
      eventType,
      correlationId,
    })

    // 🔥 去重检查
    if (correlationId) {
      const entry = this.interruptTable.get(correlationId)
      if (entry) {
        logger.debug(LogTags.SYSTEM_PIPELINE, 'INT: 丢弃重复事件（本机已处理）', {
          correlationId,
          eventType,
          originalType: entry.instruction.type,
          age: Date.now() - entry.timestamp,
        })
        return // 丢弃
      }
    }

    // 🔥 分发给对应的 handlers
    const handlers = this.handlers.get(eventType) || []
    if (handlers.length === 0) {
      logger.warn(LogTags.SYSTEM_PIPELINE, 'INT: 没有注册的处理器', { eventType })
      return
    }

    logger.info(LogTags.SYSTEM_PIPELINE, 'INT: 分发中断', {
      type,
      eventType,
      correlationId,
      handlerCount: handlers.length,
    })

    for (const handler of handlers) {
      try {
        const result = handler(event)
        if (result instanceof Promise) {
          result.catch((err) => {
            logger.error(
              LogTags.SYSTEM_PIPELINE,
              'INT: 处理器错误（async）',
              err instanceof Error ? err : new Error(String(err)),
              { eventType }
            )
          })
        }
      } catch (err) {
        logger.error(
          LogTags.SYSTEM_PIPELINE,
          'INT: 处理器错误（sync）',
          err instanceof Error ? err : new Error(String(err)),
          { eventType }
        )
      }
    }
  }

  /**
   * 检查是否是本机操作
   */
  isLocalOperation(correlationId: string): boolean {
    return this.interruptTable.has(correlationId)
  }

  /**
   * 清理过期条目
   */
  private cleanup(): void {
    const now = Date.now()
    const before = this.interruptTable.size

    for (const [correlationId, entry] of this.interruptTable.entries()) {
      if (now - entry.timestamp > this.TTL) {
        this.interruptTable.delete(correlationId)
      }
    }

    const after = this.interruptTable.size
    if (before !== after) {
      logger.debug(LogTags.SYSTEM_PIPELINE, 'INT: 清理过期条目', {
        before,
        after,
        cleaned: before - after,
      })
    }
  }

  /**
   * 获取中断表统计
   */
  getStats() {
    return {
      tableSize: this.interruptTable.size,
      entries: Array.from(this.interruptTable.values()).map((e) => ({
        correlationId: e.correlationId,
        type: e.instruction.type,
        age: Date.now() - e.timestamp,
      })),
    }
  }

  /**
   * 销毁
   */
  destroy(): void {
    if (this.cleanupTimer !== null) {
      clearInterval(this.cleanupTimer)
      this.cleanupTimer = null
    }
    this.interruptTable.clear()
  }
}

// 导出单例
export const interruptHandler = new InterruptHandler()

/**
 * INT: 中断处理器（Interrupt Handler）
 *
 * 职责：
 * 1. 注册本机发起的指令（通过 correlation_id）
 * 2. 拦截所有 SSE 事件
 * 3. 去重：丢弃本机已处理的事件
 * 4. 转发：应用其他机器的操作
 *
 * 架构：
 * WB 完成 → INT.register(correlationId)
 * SSE 到达 → INT.handle(event) → 检查 → 应用/丢弃
 */

import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 中断类型
 */
export enum InterruptType {
  SSE = 'sse', // Server-Sent Events
  WEBSOCKET = 'ws', // WebSocket
  POLLING = 'polling', // 长轮询
}

/**
 * 中断事件
 */
export interface InterruptEvent {
  type: InterruptType
  correlationId?: string
  eventId?: string
  payload: any
  timestamp: number
}

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
 * 中断处理器
 */
export class InterruptHandler {
  // 中断表：记录本机发起的指令
  private interruptTable = new Map<string, InterruptEntry>()

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
   * 处理中断事件（SSE/WebSocket 等）
   *
   * @returns true = 应用更新, false = 丢弃（本机已处理）
   */
  handle(event: InterruptEvent): boolean {
    const { correlationId, type, payload } = event

    if (!correlationId) {
      // 没有 correlation_id 的事件，直接应用
      logger.debug(LogTags.SYSTEM_PIPELINE, 'INT: 无 correlation_id，直接应用', { type })
      return true
    }

    // 检查中断表
    const entry = this.interruptTable.get(correlationId)

    if (entry) {
      // 🔥 本机已处理，丢弃 SSE 事件
      logger.debug(LogTags.SYSTEM_PIPELINE, 'INT: 丢弃重复事件（本机已处理）', {
        correlationId,
        type,
        originalType: entry.instruction.type,
        age: Date.now() - entry.timestamp,
      })
      return false
    }

    // 其他机器的操作，应用更新
    logger.info(LogTags.SYSTEM_PIPELINE, 'INT: 应用远程更新', {
      correlationId,
      type,
      payload,
    })
    return true
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

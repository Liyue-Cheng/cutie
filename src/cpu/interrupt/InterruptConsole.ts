/**
 * 中断控制台打印系统
 *
 * 职责：
 * 1. 实时打印中断处理过程
 * 2. 美观的彩色输出
 * 3. 追踪所有中断的处理结果
 * 4. 可折叠的详细信息
 */

import type { InterruptEvent } from './InterruptHandler'

export enum InterruptAction {
  RECEIVED = 'received', // 收到中断
  DEDUPLICATED = 'deduplicated', // 去重丢弃
  DISPATCHED = 'dispatched', // 已分发
  HANDLER_SUCCESS = 'handler_success', // 处理成功
  HANDLER_ERROR = 'handler_error', // 处理错误
  NO_HANDLER = 'no_handler', // 无处理器
}

export interface InterruptHandlerResult {
  handlerName: string
  success: boolean
  error?: Error
  duration: number
}

export class InterruptConsole {
  private enabled: boolean = true

  constructor() {
    this.loadSettings()
  }

  private loadSettings(): void {
    const savedEnabled = localStorage.getItem('interrupt-console-enabled')
    if (savedEnabled !== null) {
      this.enabled = savedEnabled === 'true'
    }
  }

  enable(): void {
    this.enabled = true
    localStorage.setItem('interrupt-console-enabled', 'true')
  }

  disable(): void {
    this.enabled = false
    localStorage.setItem('interrupt-console-enabled', 'false')
  }

  isEnabled(): boolean {
    return this.enabled
  }

  /**
   * 格式化时间戳
   */
  private formatTime(): string {
    const now = new Date()
    const hours = now.getHours().toString().padStart(2, '0')
    const minutes = now.getMinutes().toString().padStart(2, '0')
    const seconds = now.getSeconds().toString().padStart(2, '0')
    return `${hours}:${minutes}:${seconds}`
  }

  // ==================== 打印方法 ====================

  /**
   * 中断接收
   */
  onInterruptReceived(event: InterruptEvent): void {
    if (!this.enabled) return

    console.groupCollapsed(
      `%c[中断接收] %c${this.formatTime()} %c${event.eventType}%c %c${event.type.toUpperCase()}`,
      'color: #3b82f6; font-weight: bold',
      'color: #666; font-size: 11px',
      'color: #3b82f6; font-weight: bold; background: #3b82f615; padding: 2px 6px; border-radius: 3px',
      'color: #3b82f6',
      'color: #06b6d4; font-weight: bold'
    )

    console.log('%c📥 中断信息:', 'color: #3b82f6; font-weight: bold')
    console.table({
      'Event Type': event.eventType,
      'Interrupt Type': event.type,
      'Correlation ID': event.correlationId || 'N/A',
      'Event ID': event.eventId || 'N/A',
      Timestamp: new Date(event.timestamp).toISOString(),
    })

    console.log('%c📦 Payload:', 'color: #3b82f6; font-weight: bold')
    console.log(event.payload)

    console.groupEnd()
  }

  /**
   * 中断去重（丢弃）
   */
  onInterruptDeduplicated(
    event: InterruptEvent,
    localEntry: { type: string; timestamp: number }
  ): void {
    if (!this.enabled) return

    const age = Date.now() - localEntry.timestamp

    console.groupCollapsed(
      `%c[中断去重] %c${this.formatTime()} %c${event.eventType}%c %c${age}ms前已处理`,
      'color: #f59e0b; font-weight: bold',
      'color: #666; font-size: 11px',
      'color: #f59e0b; font-weight: bold; background: #f59e0b15; padding: 2px 6px; border-radius: 3px',
      'color: #f59e0b',
      'color: #f59e0b; font-weight: bold'
    )

    console.log('%c🔥 去重信息:', 'color: #f59e0b; font-weight: bold')
    console.table({
      'Event Type': event.eventType,
      'Correlation ID': event.correlationId,
      'Local Instruction': localEntry.type,
      'Age (ms)': age,
      Action: '✖️ 丢弃（本机已处理）',
    })

    console.groupEnd()
  }

  /**
   * 中断分发（无处理器）
   */
  onNoHandler(event: InterruptEvent): void {
    if (!this.enabled) return

    console.groupCollapsed(
      `%c[中断警告] %c${this.formatTime()} %c${event.eventType}%c %c无处理器`,
      'color: #ef4444; font-weight: bold',
      'color: #666; font-size: 11px',
      'color: #ef4444; font-weight: bold; background: #ef444415; padding: 2px 6px; border-radius: 3px',
      'color: #ef4444',
      'color: #ef4444; font-weight: bold'
    )

    console.log('%c⚠️ 警告:', 'color: #ef4444; font-weight: bold')
    console.log(`没有为事件类型 "${event.eventType}" 注册处理器`)

    console.groupEnd()
  }

  /**
   * 中断分发
   */
  onInterruptDispatched(
    event: InterruptEvent,
    handlerCount: number,
    results: InterruptHandlerResult[]
  ): void {
    if (!this.enabled) return

    const allSuccess = results.every((r) => r.success)
    const totalDuration = results.reduce((sum, r) => sum + r.duration, 0)

    console.groupCollapsed(
      `%c[中断处理] %c${this.formatTime()} %c${event.eventType}%c %c${totalDuration}ms%c %c${allSuccess ? '✓' : '⚠'}`,
      allSuccess ? 'color: #10b981; font-weight: bold' : 'color: #ef4444; font-weight: bold',
      'color: #666; font-size: 11px',
      (allSuccess ? 'color: #10b981' : 'color: #ef4444') +
        '; font-weight: bold; background: ' +
        (allSuccess ? '#10b98115' : '#ef444415') +
        '; padding: 2px 6px; border-radius: 3px',
      allSuccess ? 'color: #10b981' : 'color: #ef4444',
      'color: #10b981; font-weight: bold',
      'color: #10b981',
      'color: #10b981; font-weight: bold; font-size: 16px'
    )

    console.log('%c📤 分发信息:', 'color: #10b981; font-weight: bold')
    console.table({
      'Event Type': event.eventType,
      'Correlation ID': event.correlationId || 'N/A',
      'Handler Count': handlerCount,
      'Total Duration (ms)': totalDuration,
      Status: allSuccess ? '✓ 全部成功' : '⚠ 部分失败',
    })

    // 🔥 显示每个处理器的结果
    console.log('%c📋 处理器结果:', 'color: #10b981; font-weight: bold')
    results.forEach((result, index) => {
      const status = result.success ? '✓' : '✖'
      const color = result.success ? '#10b981' : '#ef4444'

      console.log(
        `%c${status} Handler ${index + 1}: %c${result.handlerName}%c (${result.duration}ms)`,
        `color: ${color}; font-weight: bold`,
        'color: #666',
        'color: #666'
      )

      if (result.error) {
        console.error('   Error:', result.error)
      }
    })

    // 🔥 显示 Payload
    console.log('%c📦 Event Payload:', 'color: #10b981; font-weight: bold')
    console.log(event.payload)

    console.groupEnd()
  }

  /**
   * 中断处理器错误
   */
  onHandlerError(event: InterruptEvent, handlerName: string, error: Error, isAsync: boolean): void {
    if (!this.enabled) return

    console.groupCollapsed(
      `%c[处理器错误] %c${this.formatTime()} %c${event.eventType}%c %c${isAsync ? 'async' : 'sync'}`,
      'color: #ef4444; font-weight: bold',
      'color: #666; font-size: 11px',
      'color: #ef4444; font-weight: bold; background: #ef444415; padding: 2px 6px; border-radius: 3px',
      'color: #ef4444',
      'color: #ef4444; font-weight: bold'
    )

    console.log('%c❌ 错误信息:', 'color: #ef4444; font-weight: bold')
    console.table({
      'Event Type': event.eventType,
      Handler: handlerName,
      'Error Type': isAsync ? 'Async' : 'Sync',
      'Error Message': error.message,
    })

    console.error('Stack Trace:', error)

    console.groupEnd()
  }

  /**
   * 中断注册（WB阶段）
   */
  onInterruptRegistered(correlationId: string, instructionType: string, tableSize: number): void {
    if (!this.enabled) return

    console.log(
      `%c[中断注册] %c${this.formatTime()} %c${instructionType}%c %c表大小: ${tableSize}`,
      'color: #8b5cf6; font-weight: bold',
      'color: #666; font-size: 11px',
      'color: #8b5cf6; font-weight: bold; background: #8b5cf615; padding: 2px 6px; border-radius: 3px',
      'color: #8b5cf6',
      'color: #666; font-size: 11px'
    )
  }

  /**
   * 中断表清理
   */
  onCleanup(before: number, after: number, cleaned: number): void {
    if (!this.enabled || cleaned === 0) return

    console.log(
      `%c[中断清理] %c${this.formatTime()} %c清理了 ${cleaned} 个过期条目%c (${before} → ${after})`,
      'color: #6366f1; font-weight: bold',
      'color: #666; font-size: 11px',
      'color: #6366f1; font-weight: bold',
      'color: #666; font-size: 11px'
    )
  }
}

// 导出单例
export const interruptConsole = new InterruptConsole()

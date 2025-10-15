/**
 * 全局命令总线
 *
 * 职责：
 * - 接收来自UI组件的命令
 * - 分发命令到对应的处理器
 * - 统一的错误处理
 * - 日志记录
 *
 * 架构：
 * 组件 → commandBus.emit() → handler → store → API
 */

import { logger, LogTags } from '@/infra/logging/logger'
import type { Command, CommandHandler, CommandHandlerMap } from './types'

class CommandBus {
  /**
   * 命令处理器注册表
   */
  private handlers: Partial<CommandHandlerMap> = {}

  /**
   * 注册命令处理器
   */
  on<T extends Command['type']>(
    type: T,
    handler: CommandHandler<Extract<Command, { type: T }>>
  ): void {
    if (this.handlers[type]) {
      logger.warn(LogTags.SYSTEM_COMMAND, `Handler for ${type} is being overwritten`)
    }

    this.handlers[type] = handler as any
  }

  /**
   * 发射命令
   *
   * @param type 命令类型
   * @param payload 命令负载
   * @param options 可选参数（用于追踪器等）
   * @returns Promise<void>
   * @throws Error 如果命令处理失败
   */
  async emit<T extends Command['type']>(
    type: T,
    payload: Extract<Command, { type: T }>['payload'],
    options?: { correlationId?: string; [key: string]: any }
  ): Promise<void> {
    const handler = this.handlers[type]

    if (!handler) {
      const errorMsg = `No handler registered for command: ${type}`
      logger.error(LogTags.SYSTEM_COMMAND, errorMsg, new Error(errorMsg))
      throw new Error(errorMsg)
    }

    try {
      await handler(payload as any)
    } catch (error) {
      logger.error(
        LogTags.SYSTEM_COMMAND,
        `Command execution failed: ${type}`,
        error instanceof Error ? error : new Error(String(error)),
        { payload, correlationId: options?.correlationId }
      )
      throw error
    }
  }

  /**
   * 批量注册处理器
   */
  registerHandlers(handlers: Partial<CommandHandlerMap>): void {
    Object.entries(handlers).forEach(([type, handler]) => {
      if (handler) {
        this.on(type as Command['type'], handler as any)
      }
    })
  }

  /**
   * 取消注册处理器（用于测试或热重载）
   */
  off(type: Command['type']): void {
    delete this.handlers[type]
  }

  /**
   * 清空所有处理器（用于测试）
   */
  clear(): void {
    this.handlers = {}
  }

  /**
   * 获取已注册的命令类型列表
   */
  getRegisteredCommands(): string[] {
    return Object.keys(this.handlers)
  }
}

// 导出全局单例
export const commandBus = new CommandBus()

// 开发环境：暴露到 window 用于调试
if (import.meta.env.DEV) {
  ;(window as any).commandBus = {
    emit: (type: string, payload: any) => commandBus.emit(type as any, payload),
    getRegisteredCommands: () => commandBus.getRegisteredCommands(),
    help: () => {
      console.log(`
🎯 Command Bus 使用指南

全局命令总线用于统一处理所有用户操作：

示例：
  commandBus.emit('task.complete', { id: 'task-123' })
  commandBus.emit('task.create', { title: '新任务' })

已注册的命令：
  ${commandBus.getRegisteredCommands().join('\n  ')}

💡 在组件中使用：
  import { commandBus } from '@/commandBus'
  
  async function handleComplete() {
    await commandBus.emit('task.complete', { id: task.id })
  }
      `)
    },
  }
}

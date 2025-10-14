/**
 * 命令总线 - 统一导出
 *
 * 使用方法：
 *
 * 1. 在 main.ts 中初始化：
 *    import { initCommandBus } from '@/commandBus'
 *    initCommandBus()
 *
 * 2. 在组件中使用：
 *    import { commandBus } from '@/commandBus'
 *    await commandBus.emit('task.complete', { id: 'task-123' })
 */

export { commandBus } from './CommandBus'
export * from './types'
export * from './handlers'

import { commandBus } from './CommandBus'
import { allHandlers } from './handlers'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 初始化命令总线
 *
 * 注册所有命令处理器
 * 应在应用启动时调用（main.ts）
 */
export function initCommandBus(): void {
  logger.info(LogTags.SYSTEM_COMMAND, 'Initializing command bus')

  // 注册所有处理器
  commandBus.registerHandlers(allHandlers)

  const registeredCommands = commandBus.getRegisteredCommands()
  logger.info(
    LogTags.SYSTEM_COMMAND,
    `Command bus initialized with ${registeredCommands.length} handlers`,
    {
      commands: registeredCommands,
    }
  )
}

/**
 * 命令处理器入口
 *
 * 统一导出所有命令处理器
 */

export * from './taskHandlers'
export * from './scheduleHandlers'
export * from './timeBlockHandlers'
export * from './viewPreferenceHandlers'

import { taskHandlers } from './taskHandlers'
import { scheduleHandlers } from './scheduleHandlers'
import { timeBlockHandlers } from './timeBlockHandlers'
import { viewPreferenceHandlers } from './viewPreferenceHandlers'
import type { CommandHandlerMap } from '../types'

/**
 * 所有命令处理器的集合
 */
export const allHandlers: Partial<CommandHandlerMap> = {
  ...taskHandlers,
  ...scheduleHandlers,
  ...timeBlockHandlers,
  ...viewPreferenceHandlers,
}

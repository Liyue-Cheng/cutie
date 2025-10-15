/**
 * 命令处理器入口
 *
 * 统一导出所有命令处理器
 *
 * 🔄 迁移状态：
 * - ✅ taskHandlers → 已迁移到 CPU ISA (task-isa.ts)
 * - ✅ scheduleHandlers → 已迁移到 CPU ISA (schedule-isa.ts)
 * - ✅ timeBlockHandlers → 已迁移到 CPU ISA (timeblock-isa.ts)
 * - ⏳ viewPreferenceHandlers → 仍使用 commandBus（排序功能）
 */

export * from './viewPreferenceHandlers'

import { viewPreferenceHandlers } from './viewPreferenceHandlers'
import type { CommandHandlerMap } from '../types'

/**
 * 所有命令处理器的集合
 *
 * 注意：大部分指令已迁移到 CPU Pipeline ISA
 * 这里只保留尚未迁移的处理器
 */
export const allHandlers: Partial<CommandHandlerMap> = {
  ...viewPreferenceHandlers,
}

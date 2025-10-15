/**
 * ISA指令集聚合导出
 */

import type { ISADefinition } from './types'
import { DebugISA } from './debug-isa'
import { TaskISA } from './task-isa'
import { ScheduleISA } from './schedule-isa'

/**
 * 完整的ISA定义
 */
export const ISA: ISADefinition = {
  ...DebugISA,
  ...TaskISA,
  ...ScheduleISA,
}

// 导出类型
export type { InstructionDefinition, InstructionMeta, ISADefinition } from './types'

/**
 * ISA指令集聚合导出
 */

import type { ISADefinition } from './types'
import { DebugISA } from './debug-isa'
import { TaskISA } from './task-isa'
import { ScheduleISA } from './schedule-isa'
import { TimeBlockISA } from './timeblock-isa'
import { ViewPreferenceISA } from './viewpreference-isa'
import { TemplateISA } from './template-isa'
import { RecurrenceISA } from './recurrence-isa'

/**
 * 完整的ISA定义
 */
export const ISA: ISADefinition = {
  ...DebugISA,
  ...TaskISA,
  ...ScheduleISA,
  ...TimeBlockISA,
  ...ViewPreferenceISA,
  ...TemplateISA,
  ...RecurrenceISA,
}

// 导出类型
export type { InstructionDefinition, InstructionMeta, ISADefinition } from './types'

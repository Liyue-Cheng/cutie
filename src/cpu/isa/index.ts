/**
 * ISA指令集聚合导出
 */

import type { ISADefinition } from 'front-cpu'
import { DebugISA } from './debug-isa'
import { TaskISA } from './task-isa'
import { ScheduleISA } from './schedule-isa'
import { TimeBlockISA } from './timeblock-isa'
import { TimeBlockRecurrenceISA } from './timeblock-recurrence-isa'
import { TemplateISA } from './template-isa'
import { RecurrenceISA } from './recurrence-isa'
import { UserSettingsISA } from './user-settings-isa'
import { AreaISA } from './area-isa'
import { ProjectISA } from './project-isa'
import { ShutdownRitualISA } from './shutdown-ritual-isa'

/**
 * 完整的ISA定义
 */
export const ISA: ISADefinition = {
  ...DebugISA,
  ...TaskISA,
  ...ScheduleISA,
  ...TimeBlockISA,
  ...TimeBlockRecurrenceISA,
  ...TemplateISA,
  ...RecurrenceISA,
  ...UserSettingsISA,
  ...AreaISA,
  ...ProjectISA,
  ...ShutdownRitualISA,
}

// 导出类型
export type { InstructionDefinition, InstructionMeta, ISADefinition } from 'front-cpu'

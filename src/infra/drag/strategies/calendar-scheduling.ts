/**
 * 日历调度策略
 *
 * 所有拖放到日历的策略：
 * - staging -> calendar (全日/分时)
 * - daily -> calendar (全日/分时)
 * - 任何视图 -> calendar (全日/分时)
 */

import type { Strategy } from '../types'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'
import { isTaskCard } from '@/types/dtos'

/**
 * 策略：任何视图 -> Calendar（全日）
 */
export const anyToCalendarAllDayStrategy: Strategy = {
  id: 'any-to-calendar-allday',
  name: 'Any to Calendar (All Day)',

  conditions: {
    source: {
      objectType: 'task', // 只支持任务拖放到日历
    },
    target: {
      viewKey: /^calendar-allday-/, // 匹配 calendar-allday-{ISO}
    },
    priority: 100,
  },

  action: {
    name: 'create_allday_timeblock',
    description: '拖放到日历全日区域，创建全天时间块',

    async execute(ctx) {
      // 类型守卫
      if (!isTaskCard(ctx.draggedObject)) {
        throw new Error('Expected task object')
      }
      const task = ctx.draggedObject

      try {
        // 从 targetContext 解析时间信息
        const targetConfig = ctx.targetContext.calendarConfig
        if (!targetConfig) {
          return {
            success: false,
            message: '❌ 缺少日历配置信息',
          }
        }

        const { startTime, endTime } = targetConfig
        const viewType = ctx.targetContext?.calendarViewType
        const calendarDate: string | null =
          ctx.targetContext?.calendarDate ??
          (startTime ? new Date(startTime).toISOString().split('T')[0] : null)

        // 月视图：只创建日程，不创建时间块
        if (viewType === 'dayGridMonth') {
          if (!calendarDate) {
            return {
              success: false,
              message: '❌ 无法解析日历日期',
            }
          }

          await pipeline.dispatch('schedule.create', {
            task_id: task.id,
            scheduled_day: calendarDate,
          })

          logger.info(LogTags.DRAG_STRATEGY, 'Scheduled task via calendar month drop', {
            taskId: task.id,
            calendarDate,
          })

          return {
            success: true,
            message: '✅ 已排期任务',
            affectedViews: [ctx.sourceViewId, 'calendar'],
          }
        }

        // 🎯 步骤 1: 如果是 tiny 任务，先更新 estimated_duration
        if (task.estimated_duration === null || task.estimated_duration === 0) {
          await pipeline.dispatch('task.update', {
            id: task.id,
            updates: { estimated_duration: 15 },
          })
        }

        // 🎯 步骤 2: 创建时间块
        const createPayload = {
          task_id: task.id,
          start_time: startTime,
          end_time: endTime,
          start_time_local: '00:00:00',
          end_time_local: '23:59:59',
          time_type: 'FLOATING' as const,
          creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          is_all_day: true,
        }

        await pipeline.dispatch('time_block.create_from_task', createPayload)

        logger.info(LogTags.DRAG_STRATEGY, 'Created all-day time block', {
          taskId: task.id,
          startTime,
          endTime,
        })

        return {
          success: true,
          message: '✅ 已创建全天时间块',
          affectedViews: [ctx.sourceViewId, 'calendar'],
        }
      } catch (error) {
        logger.error(
          LogTags.DRAG_STRATEGY,
          'Failed to create all-day time block',
          error instanceof Error ? error : new Error(String(error))
        )
        return {
          success: false,
          message: `❌ 创建时间块失败: ${error instanceof Error ? error.message : String(error)}`,
        }
      }
    },
  },

  tags: ['calendar', 'allday', 'timeblock'],
}

/**
 * 策略：任何视图 -> Calendar（分时）
 */
export const anyToCalendarTimedStrategy: Strategy = {
  id: 'any-to-calendar-timed',
  name: 'Any to Calendar (Timed)',

  conditions: {
    source: {
      objectType: 'task', // 只支持任务拖放到日历
    },
    target: {
      viewKey: /^calendar-[^a]/, // 匹配 calendar-{ISO}（排除 calendar-allday-）
    },
    priority: 100,
  },

  action: {
    name: 'create_timed_timeblock',
    description: '拖放到日历分时区域，创建分时时间块',

    async execute(ctx) {
      // 类型守卫
      if (!isTaskCard(ctx.draggedObject)) {
        throw new Error('Expected task object')
      }
      const task = ctx.draggedObject

      try {
        // 从 targetContext 解析时间信息
        const targetConfig = ctx.targetContext.calendarConfig
        if (!targetConfig) {
          return {
            success: false,
            message: '❌ 缺少日历配置信息',
          }
        }

        let { startTime, endTime } = targetConfig

        // 🔥 截断到当日 24:00
        const start = new Date(startTime)
        let end = new Date(endTime)
        const dayEnd = new Date(start)
        dayEnd.setHours(0, 0, 0, 0)
        dayEnd.setDate(dayEnd.getDate() + 1)

        if (end.getTime() > dayEnd.getTime()) {
          end = dayEnd
        }

        // 计算本地时间字符串
        const startTimeLocal = start.toTimeString().split(' ')[0] || '00:00:00' // HH:mm:ss
        const endTimeLocal = end.toTimeString().split(' ')[0] || '23:59:59'

        // 🎯 步骤 1: 如果是 tiny 任务，先更新 estimated_duration
        if (task.estimated_duration === null || task.estimated_duration === 0) {
          await pipeline.dispatch('task.update', {
            id: task.id,
            updates: { estimated_duration: 15 },
          })
        }

        // 🎯 步骤 2: 创建时间块
        const createPayload = {
          task_id: task.id,
          start_time: start.toISOString(),
          end_time: end.toISOString(),
          start_time_local: startTimeLocal,
          end_time_local: endTimeLocal,
          time_type: 'FLOATING' as const,
          creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          is_all_day: false,
        }

        pipeline.dispatch('time_block.create_from_task', createPayload)

        logger.info(LogTags.DRAG_STRATEGY, 'Created timed time block', {
          taskId: task.id,
          startTime: start.toISOString(),
          endTime: end.toISOString(),
        })

        return {
          success: true,
          message: '✅ 已创建时间块',
          affectedViews: [ctx.sourceViewId, 'calendar'],
        }
      } catch (error) {
        logger.error(
          LogTags.DRAG_STRATEGY,
          'Failed to create timed time block',
          error instanceof Error ? error : new Error(String(error))
        )
        return {
          success: false,
          message: `❌ 创建时间块失败: ${error instanceof Error ? error.message : String(error)}`,
        }
      }
    },
  },

  tags: ['calendar', 'timed', 'timeblock'],
}

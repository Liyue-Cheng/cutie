/**
 * 任务调度策略（策略链实现 - 生产模式）
 *
 * 每个策略执行真实的业务逻辑：
 * - 创建/更新/删除日程（通过 CommandBus）
 * - 更新源视图排序（通过 CommandBus）
 * - 更新目标视图排序（通过 CommandBus）
 */

import type { Strategy } from '../types'
import {
  extractTaskIds,
  removeTaskFrom,
  insertTaskAt,
  moveTaskWithin,
  extractDate,
  isSameDay,
  createOperationRecord,
  type OperationRecord,
} from './strategy-utils'
import { commandBus } from '@/commandBus'

/**
 * 策略 1：Staging → Daily
 *
 * 操作链：
 * 1. 创建日程 (task.create_with_schedule)
 * 2. 从 Staging 移除 (view.update_sorting)
 * 3. 插入到 Daily (view.update_sorting)
 */
export const stagingToDailyStrategy: Strategy = {
  id: 'staging-to-daily',
  name: 'Staging to Daily Schedule',

  conditions: {
    source: {
      viewKey: 'misc::staging',
      taskStatus: 'staging',
    },
    target: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
    },
    priority: 100,
  },

  action: {
    name: 'schedule_task',
    description: '将暂存区任务安排到指定日期（3步操作）',

    async execute(ctx) {
      const targetDate = extractDate(ctx.targetZone)!
      const operations: OperationRecord[] = []

      try {
        // 🎯 步骤 1: 创建日程
        const createPayload = {
          title: ctx.task.title,
          scheduled_day: targetDate,
          area_id: ctx.task.area_id,
          glance_note: ctx.task.glance_note,
        }
        await commandBus.emit('task.create_with_schedule', createPayload)
        operations.push(createOperationRecord('create_schedule', ctx.targetViewId, createPayload))

        // 🎯 步骤 2: 从 Staging 移除（更新排序）
        const sourceSorting = extractTaskIds(ctx.sourceContext)
        const newSourceSorting = removeTaskFrom(sourceSorting, ctx.task.id)
        const sourceSortPayload = {
          view_key: ctx.sourceViewId,
          sorted_task_ids: newSourceSorting,
          original_sorted_task_ids: sourceSorting,
        }
        await commandBus.emit('view.update_sorting', sourceSortPayload)
        operations.push(
          createOperationRecord('update_sorting', ctx.sourceViewId, sourceSortPayload)
        )

        // 🎯 步骤 3: 插入到 Daily（更新排序）
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newTargetSorting = insertTaskAt(targetSorting, ctx.task.id, ctx.dropIndex)
        const targetSortPayload = {
          view_key: ctx.targetViewId,
          sorted_task_ids: newTargetSorting,
          original_sorted_task_ids: targetSorting,
        }
        await commandBus.emit('view.update_sorting', targetSortPayload)
        operations.push(
          createOperationRecord('update_sorting', ctx.targetViewId, targetSortPayload)
        )

        return {
          success: true,
          message: `✅ Scheduled to ${targetDate}`,
          operations,
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ Failed to schedule: ${error instanceof Error ? error.message : String(error)}`,
          operations,
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      }
    },
  },

  tags: ['scheduling', 'staging', 'daily', 'multi-step'],
}

/**
 * 策略 2：Daily → Daily
 *
 * 两种情况：
 *
 * A. 同日期（重新排序）：
 *    1. 更新 Daily 排序 (view.update_sorting)
 *
 * B. 跨日期（重新安排）：
 *    1. 更新日程日期 (schedule.update)
 *    2. 从源 Daily 移除 (view.update_sorting)
 *    3. 插入到目标 Daily (view.update_sorting)
 */
export const dailyToDailyStrategy: Strategy = {
  id: 'daily-to-daily',
  name: 'Daily to Daily Reschedule',

  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
      taskStatus: 'scheduled',
    },
    target: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
    },
    priority: 90,
  },

  action: {
    name: 'reschedule_task',
    description: '在不同日期之间移动任务或同日期内重新排序',

    async execute(ctx) {
      const sourceDate = extractDate(ctx.sourceViewId)!
      const targetDate = extractDate(ctx.targetZone)!
      const operations: OperationRecord[] = []

      try {
        // 🔹 情况 A: 同日期重新排序
        if (isSameDay(ctx.sourceViewId, ctx.targetZone)) {
          const sorting = extractTaskIds(ctx.sourceContext)
          const newSorting = moveTaskWithin(sorting, ctx.task.id, ctx.dropIndex ?? sorting.length)
          const sortPayload = {
            view_key: ctx.sourceViewId,
            sorted_task_ids: newSorting,
            original_sorted_task_ids: sorting,
          }
          await commandBus.emit('view.update_sorting', sortPayload)
          operations.push(createOperationRecord('update_sorting', ctx.sourceViewId, sortPayload))

          return {
            success: true,
            message: `✅ Reordered in ${sourceDate}`,
            reorderOnly: true,
            operations,
            affectedViews: [ctx.sourceViewId],
          }
        }

        // 🔹 情况 B: 跨日期重新安排
        // 🎯 步骤 1: 更新日程日期
        const updatePayload = {
          task_id: ctx.task.id,
          scheduled_day: sourceDate,
          updates: {
            new_date: targetDate,
          },
        }
        await commandBus.emit('schedule.update', updatePayload)
        operations.push(createOperationRecord('update_schedule', ctx.targetViewId, updatePayload))

        // 🎯 步骤 2: 从源 Daily 移除
        const sourceSorting = extractTaskIds(ctx.sourceContext)
        const newSourceSorting = removeTaskFrom(sourceSorting, ctx.task.id)
        const sourceSortPayload = {
          view_key: ctx.sourceViewId,
          sorted_task_ids: newSourceSorting,
          original_sorted_task_ids: sourceSorting,
        }
        await commandBus.emit('view.update_sorting', sourceSortPayload)
        operations.push(
          createOperationRecord('update_sorting', ctx.sourceViewId, sourceSortPayload)
        )

        // 🎯 步骤 3: 插入到目标 Daily
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newTargetSorting = insertTaskAt(targetSorting, ctx.task.id, ctx.dropIndex)
        const targetSortPayload = {
          view_key: ctx.targetViewId,
          sorted_task_ids: newTargetSorting,
          original_sorted_task_ids: targetSorting,
        }
        await commandBus.emit('view.update_sorting', targetSortPayload)
        operations.push(
          createOperationRecord('update_sorting', ctx.targetViewId, targetSortPayload)
        )

        return {
          success: true,
          message: `✅ Rescheduled from ${sourceDate} to ${targetDate}`,
          operations,
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ Failed to reschedule: ${error instanceof Error ? error.message : String(error)}`,
          operations,
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      }
    },
  },

  tags: ['scheduling', 'daily', 'reschedule', 'multi-step'],
}

/**
 * 策略 3：Daily → Staging
 *
 * 操作链：
 * 1. 删除日程 (schedule.delete)
 * 2. 从 Daily 移除 (view.update_sorting)
 * 3. 插入到 Staging (view.update_sorting)
 */
export const dailyToStagingStrategy: Strategy = {
  id: 'daily-to-staging',
  name: 'Daily to Staging Return',

  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
      taskStatus: 'scheduled',
    },
    target: {
      viewKey: 'misc::staging',
    },
    priority: 95,
  },

  action: {
    name: 'return_to_staging',
    description: '将任务退回暂存区（3步操作）',

    async canExecute(ctx) {
      // 已完成的任务不能退回
      if (ctx.task.is_completed) {
        console.warn(`⚠️ Cannot return completed task to staging`)
        return false
      }
      return true
    },

    async execute(ctx) {
      const sourceDate = extractDate(ctx.sourceViewId)!
      const operations: OperationRecord[] = []

      try {
        // 🎯 步骤 1: 删除日程
        const deletePayload = {
          task_id: ctx.task.id,
          scheduled_day: sourceDate,
        }
        await commandBus.emit('schedule.delete', deletePayload)
        operations.push(createOperationRecord('delete_schedule', ctx.sourceViewId, deletePayload))

        // 🎯 步骤 2: 从 Daily 移除
        const sourceSorting = extractTaskIds(ctx.sourceContext)
        const newSourceSorting = removeTaskFrom(sourceSorting, ctx.task.id)
        const sourceSortPayload = {
          view_key: ctx.sourceViewId,
          sorted_task_ids: newSourceSorting,
          original_sorted_task_ids: sourceSorting,
        }
        await commandBus.emit('view.update_sorting', sourceSortPayload)
        operations.push(
          createOperationRecord('update_sorting', ctx.sourceViewId, sourceSortPayload)
        )

        // 🎯 步骤 3: 插入到 Staging
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newTargetSorting = insertTaskAt(targetSorting, ctx.task.id, ctx.dropIndex)
        const targetSortPayload = {
          view_key: ctx.targetViewId,
          sorted_task_ids: newTargetSorting,
          original_sorted_task_ids: targetSorting,
        }
        await commandBus.emit('view.update_sorting', targetSortPayload)
        operations.push(
          createOperationRecord('update_sorting', ctx.targetViewId, targetSortPayload)
        )

        return {
          success: true,
          message: `✅ Returned from ${sourceDate} to staging`,
          operations,
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ Failed to return to staging: ${error instanceof Error ? error.message : String(error)}`,
          operations,
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      }
    },
  },

  tags: ['scheduling', 'staging', 'daily', 'return', 'multi-step'],
}

/**
 * 策略 4：Daily 内部重排序
 *
 * 操作链：
 * 1. 更新 Daily 排序 (view.update_sorting)
 *
 * 注意：这是独立的 Daily 内部排序策略，与 dailyToDailyStrategy 不同：
 * - 此策略：专门处理同日期内的排序（高优先级，精确匹配）
 * - dailyToDailyStrategy：处理跨日期移动（低优先级，通用匹配）
 */
export const dailyReorderStrategy: Strategy = {
  id: 'daily-reorder',
  name: 'Daily Internal Reorder',

  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
      taskStatus: 'scheduled',
    },
    target: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
      // 🔥 自定义检查：确保是同一天
      customCheck: (targetZone: string, session) => {
        return isSameDay(session.source.viewKey, targetZone)
      },
    },
    priority: 92, // 比 dailyToDailyStrategy (90) 高，优先匹配同日期
  },

  action: {
    name: 'reorder_in_daily',
    description: '在同一天内重新排序（1步操作）',

    async execute(ctx) {
      const date = extractDate(ctx.sourceViewId)!
      const operations: OperationRecord[] = []

      try {
        const sorting = extractTaskIds(ctx.sourceContext)
        const newSorting = moveTaskWithin(sorting, ctx.task.id, ctx.dropIndex ?? sorting.length)
        const sortPayload = {
          view_key: ctx.sourceViewId,
          sorted_task_ids: newSorting,
          original_sorted_task_ids: sorting,
        }
        await commandBus.emit('view.update_sorting', sortPayload)
        operations.push(createOperationRecord('update_sorting', ctx.sourceViewId, sortPayload))

        return {
          success: true,
          message: `✅ Reordered in ${date}`,
          reorderOnly: true,
          operations,
          affectedViews: [ctx.sourceViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ Failed to reorder: ${error instanceof Error ? error.message : String(error)}`,
          operations,
          affectedViews: [ctx.sourceViewId],
        }
      }
    },
  },

  tags: ['scheduling', 'daily', 'reorder'],
}

/**
 * 策略 5：Staging 内部重排序
 *
 * 操作链：
 * 1. 更新 Staging 排序 (view.update_sorting)
 */
export const stagingReorderStrategy: Strategy = {
  id: 'staging-reorder',
  name: 'Staging Internal Reorder',

  conditions: {
    source: {
      viewKey: 'misc::staging',
    },
    target: {
      viewKey: 'misc::staging',
    },
    priority: 80,
  },

  action: {
    name: 'reorder_in_staging',
    description: '在暂存区内重新排序（1步操作）',

    async execute(ctx) {
      const operations: OperationRecord[] = []

      try {
        const sorting = extractTaskIds(ctx.targetContext)
        const newSorting = moveTaskWithin(sorting, ctx.task.id, ctx.dropIndex ?? sorting.length)
        const sortPayload = {
          view_key: ctx.targetZone,
          sorted_task_ids: newSorting,
          original_sorted_task_ids: sorting,
        }
        await commandBus.emit('view.update_sorting', sortPayload)
        operations.push(createOperationRecord('update_sorting', ctx.targetZone, sortPayload))

        return {
          success: true,
          message: `✅ Reordered in staging`,
          reorderOnly: true,
          operations,
          affectedViews: [ctx.sourceViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ Failed to reorder: ${error instanceof Error ? error.message : String(error)}`,
          operations,
          affectedViews: [ctx.sourceViewId],
        }
      }
    },
  },

  tags: ['scheduling', 'staging', 'reorder'],
}

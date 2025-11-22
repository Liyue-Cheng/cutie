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
import { pipeline } from '@/cpu'
import { isTaskCard } from '@/types/dtos'

function isLexoRankView(viewKey: string): boolean {
  return Boolean(viewKey)
}

function buildLexoRankPayload(viewKey: string, order: string[], taskId: string) {
  const index = order.indexOf(taskId)
  if (index === -1) {
    return null
  }

  const prev = index > 0 ? order[index - 1] : null
  const next = index < order.length - 1 ? order[index + 1] : null

  return {
    task_id: taskId,
    view_context: viewKey,
    prev_task_id: prev,
    next_task_id: next,
  }
}

/**
 * 策略 1：Staging → Daily
 *
 * 操作链：
 * 1. 为现有任务创建日程 (schedule.create)
 * 2. 从 Staging 移除 (view.update_sorting)
 * 3. 插入到 Daily (view.update_sorting)
 */
export const stagingToDailyStrategy: Strategy = {
  id: 'staging-to-daily',
  name: 'Staging to Daily Schedule',

  conditions: {
    source: {
      viewKey: 'misc::staging',
      objectType: 'task',
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
      // 类型守卫
      if (!isTaskCard(ctx.draggedObject)) {
        throw new Error('Expected task object')
      }
      const task = ctx.draggedObject

      const targetDate = extractDate(ctx.targetZone)!
      const operations: OperationRecord[] = []

      try {
        // 🎯 步骤 1: 为现有任务创建日程
        const createPayload = {
          task_id: task.id,
          scheduled_day: targetDate,
        }
        await pipeline.dispatch('schedule.create', createPayload)
        operations.push(createOperationRecord('create_schedule', ctx.targetViewId, createPayload))

        // 🎯 步骤 2: 更新目标日视图的排序
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newTargetSorting = insertTaskAt(targetSorting, task.id, ctx.dropIndex)

        const payload = buildLexoRankPayload(ctx.targetViewId, newTargetSorting, task.id)
        if (payload) {
          await pipeline.dispatch('task.update_sort_position', payload)
          operations.push(createOperationRecord('update_sort_position', ctx.targetViewId, payload))
        }

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
 * 三种情况：
 *
 * A. 同日期（重新排序）：
 *    1. 更新 Daily 排序 (view.update_sorting)
 *
 * B. 过去 → 今天/未来（保留历史）：
 *    1. 保留源日程（不删除、不更新）
 *    2. 创建目标日程 (schedule.create)
 *    3. 从源 Daily 移除 (view.update_sorting)
 *    4. 插入到目标 Daily (view.update_sorting)
 *
 * C. 其他跨日期（标准改期）：
 *    1. 更新/删除源日程
 *    2. 从源 Daily 移除 (view.update_sorting)
 *    3. 插入到目标 Daily (view.update_sorting)
 */
export const dailyToDailyStrategy: Strategy = {
  id: 'daily-to-daily',
  name: 'Daily to Daily Reschedule',

  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
      objectType: 'task',
      // 🔥 允许 scheduled 和 staging 状态
      // staging 状态表示任务只在过去有日程（今天及未来无日程）
      taskStatus: ['scheduled', 'staging'],
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
      // 类型守卫
      if (!isTaskCard(ctx.draggedObject)) {
        throw new Error('Expected task object')
      }
      const task = ctx.draggedObject

      const sourceDate = extractDate(ctx.sourceViewId)!
      const targetDate = extractDate(ctx.targetZone)!
      const operations: OperationRecord[] = []

      try {
        // 🔹 情况 A: 同日期重新排序
        if (isSameDay(ctx.sourceViewId, ctx.targetZone)) {
          const sorting = extractTaskIds(ctx.sourceContext)
          const newSorting = moveTaskWithin(sorting, task.id, ctx.dropIndex ?? sorting.length)

          const payload = buildLexoRankPayload(ctx.sourceViewId, newSorting, task.id)
          if (payload) {
            await pipeline.dispatch('task.update_sort_position', payload)
            operations.push(
              createOperationRecord('update_sort_position', ctx.sourceViewId, payload)
            )
          }

          return {
            success: true,
            message: `✅ Reordered in ${sourceDate}`,
            reorderOnly: true,
            operations,
            affectedViews: [ctx.sourceViewId],
          }
        }

        // 🔹 获取今天的日期
        const today = new Date().toISOString().split('T')[0]!

        // 🔹 判断是否是"过去 → 今天/未来"的场景
        const isFromPast = sourceDate < today
        const isToTodayOrFuture = targetDate >= today
        const isPastToFuture = isFromPast && isToTodayOrFuture

        // 🔹 情况 B: 过去 → 今天/未来（保留历史）
        if (isPastToFuture) {
          // 🔥 检查目标日期是否已有日程
          const hasTargetSchedule =
            task.schedules?.some((schedule) => schedule.scheduled_day === targetDate) ?? false

          if (!hasTargetSchedule) {
            // 🎯 步骤 1: 创建目标日程（保留源日程）
            const createPayload = {
              task_id: task.id,
              scheduled_day: targetDate,
            }
            await pipeline.dispatch('schedule.create', createPayload)
            operations.push(
              createOperationRecord('create_schedule', ctx.targetViewId, createPayload)
            )
          }
          // 如果目标已有日程，跳过创建，只更新排序

          // ✅ 保留历史：不从源 Daily 移除排序，避免任务仍因历史存在而在源列表掉到底部

          // 🎯 步骤 3: 插入到目标 Daily
          const targetSorting = extractTaskIds(ctx.targetContext)
          const newTargetSorting = insertTaskAt(targetSorting, task.id, ctx.dropIndex)

          const payload = buildLexoRankPayload(ctx.targetViewId, newTargetSorting, task.id)
          if (payload) {
            await pipeline.dispatch('task.update_sort_position', payload)
            operations.push(
              createOperationRecord('update_sort_position', ctx.targetViewId, payload)
            )
          }

          return {
            success: true,
            message: hasTargetSchedule
              ? `✅ Moved from ${sourceDate} to ${targetDate} (past schedule preserved)`
              : `✅ Moved from ${sourceDate} to ${targetDate} (past schedule preserved, new schedule created)`,
            operations,
            affectedViews: [ctx.sourceViewId, ctx.targetViewId],
          }
        }

        // 🔹 情况 C: 其他跨日期（标准改期）
        // 包括：今天 → 未来、未来 → 今天、未来 → 未来、今天 → 今天（已在情况A处理）

        // 🔥 判断是否需要保留源日程（今天 → 未来 且有实际工作记录）
        const sourceSchedule = task.schedules?.find((s) => s.scheduled_day === sourceDate)
        const isFromToday = sourceDate === today
        const isToFuture = targetDate > today
        const hasWorkRecord = sourceSchedule?.outcome !== 'planned' // PRESENCE_LOGGED 或 COMPLETED_ON_DAY
        const shouldKeepSource = isFromToday && isToFuture && hasWorkRecord

        // 🔥 先检查目标日期是否已有日程
        const hasTargetSchedule =
          task.schedules?.some((schedule) => schedule.scheduled_day === targetDate) ?? false

        if (shouldKeepSource && !hasTargetSchedule) {
          // 保留源日程 + 创建新日程
          const createPayload = {
            task_id: task.id,
            scheduled_day: targetDate,
          }
          await pipeline.dispatch('schedule.create', createPayload)
          operations.push(createOperationRecord('create_schedule', ctx.targetViewId, createPayload))
        } else if (hasTargetSchedule) {
          // 🎯 目标日期已有日程，删除源日程（避免冲突）
          const deletePayload = {
            task_id: task.id,
            scheduled_day: sourceDate,
          }
          await pipeline.dispatch('schedule.delete', deletePayload)
          operations.push(createOperationRecord('delete_schedule', ctx.sourceViewId, deletePayload))
        } else {
          // 🎯 目标日期无日程，正常更新日程日期
          const updatePayload = {
            task_id: task.id,
            scheduled_day: sourceDate,
            updates: {
              new_date: targetDate,
            },
          }
          // 🔥 使用 pipeline.dispatch 支持乐观更新
          pipeline.dispatch('schedule.update', updatePayload)
          operations.push(createOperationRecord('update_schedule', ctx.targetViewId, updatePayload))
        }

        // 🎯 步骤 2: 插入到目标 Daily
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newTargetSorting = insertTaskAt(targetSorting, task.id, ctx.dropIndex)

        const payload = buildLexoRankPayload(ctx.targetViewId, newTargetSorting, task.id)
        if (payload) {
          await pipeline.dispatch('task.update_sort_position', payload)
          operations.push(createOperationRecord('update_sort_position', ctx.targetViewId, payload))
        }

        return {
          success: true,
          message: shouldKeepSource
            ? `✅ Rescheduled from ${sourceDate} to ${targetDate} (work record preserved)`
            : hasTargetSchedule
              ? `✅ Moved from ${sourceDate} to ${targetDate} (replaced existing schedule)`
              : `✅ Rescheduled from ${sourceDate} to ${targetDate}`,
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
 * 1. 返回暂存区 (task.return_to_staging) - 后端自动处理所有清理
 * 2. 从 Daily 移除 (view.update_sorting)
 * 3. 插入到 Staging (view.update_sorting)
 */
export const dailyToStagingStrategy: Strategy = {
  id: 'daily-to-staging',
  name: 'Daily to Staging Return',

  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
      objectType: 'task',
      taskStatus: 'scheduled',
    },
    target: {
      viewKey: 'misc::staging',
    },
    priority: 95,
  },

  action: {
    name: 'return_to_staging',
    description: '将任务退回暂存区（后端统一处理）',

    async canExecute() {
      // 已完成的任务可以退回（后端会自动重新打开）
      // 移除客户端检查，让后端统一处理
      return true
    },

    async execute(ctx) {
      // 类型守卫
      if (!isTaskCard(ctx.draggedObject)) {
        throw new Error('Expected task object')
      }
      const task = ctx.draggedObject

      const operations: OperationRecord[] = []

      try {
        // 🎯 步骤 1: 使用后端统一的"返回暂存区"指令
        // 后端会自动：
        // - 删除所有 >= today 的日程
        // - 删除所有 >= today 的时间块链接
        // - 软删除孤儿时间块
        // - 如果已完成，自动重新打开
        const returnPayload = {
          id: task.id,
        }
        await pipeline.dispatch('task.return_to_staging', returnPayload)
        operations.push(createOperationRecord('return_to_staging', ctx.sourceViewId, returnPayload))

        // 🎯 步骤 2: 插入到 Staging
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newTargetSorting = insertTaskAt(targetSorting, task.id, ctx.dropIndex)

        const payload = buildLexoRankPayload(ctx.targetViewId, newTargetSorting, task.id)
        if (payload) {
          await pipeline.dispatch('task.update_sort_position', payload)
          operations.push(createOperationRecord('update_sort_position', ctx.targetViewId, payload))
        }

        return {
          success: true,
          message: `✅ Returned to staging (all future schedules cleared)`,
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
      objectType: 'task',
      // 🔥 允许 scheduled 和 staging 状态
      taskStatus: ['scheduled', 'staging'],
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
      // 类型守卫
      if (!isTaskCard(ctx.draggedObject)) {
        throw new Error('Expected task object')
      }
      const task = ctx.draggedObject

      const date = extractDate(ctx.sourceViewId)!
      const operations: OperationRecord[] = []

      try {
        const sorting = extractTaskIds(ctx.sourceContext)
        const newSorting = moveTaskWithin(sorting, task.id, ctx.dropIndex ?? sorting.length)
        const payload = buildLexoRankPayload(ctx.sourceViewId, newSorting, task.id)
        if (payload) {
          await pipeline.dispatch('task.update_sort_position', payload)
          operations.push(createOperationRecord('update_sort_position', ctx.sourceViewId, payload))
        }

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
      objectType: 'task',
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
      // 类型守卫
      if (!isTaskCard(ctx.draggedObject)) {
        throw new Error('Expected task object')
      }
      const task = ctx.draggedObject

      const operations: OperationRecord[] = []

      try {
        const sorting = extractTaskIds(ctx.targetContext)
        const newSorting = moveTaskWithin(sorting, task.id, ctx.dropIndex ?? sorting.length)
        const newIndex = newSorting.indexOf(task.id)
        const prevTaskId = newIndex > 0 ? newSorting[newIndex - 1] : null
        const nextTaskId =
          newIndex >= 0 && newIndex < newSorting.length - 1 ? newSorting[newIndex + 1] : null

        const payload = {
          task_id: task.id,
          view_context: ctx.targetZone,
          prev_task_id: prevTaskId,
          next_task_id: nextTaskId,
        }

        await pipeline.dispatch('task.update_sort_position', payload)
        operations.push(createOperationRecord('update_sort_position', ctx.targetZone, payload))

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

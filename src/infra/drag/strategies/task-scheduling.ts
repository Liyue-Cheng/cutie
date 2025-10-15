/**
 * 任务调度策略（策略链实现）
 *
 * 每个策略可以执行多个操作：
 * - 创建/更新/删除日程
 * - 更新源视图排序
 * - 更新目标视图排序
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

      console.group('📅 [PRINT MODE] Staging → Daily (Multi-Step)')
      console.log(`📦 Task: "${ctx.task.title}"`)
      console.log(`📤 From: ${ctx.sourceViewId}`)
      console.log(`📥 To: ${ctx.targetViewId} (${targetDate})`)
      console.log(`📌 Drop Index: ${ctx.dropIndex ?? 'append'}`)

      // 🎯 步骤 1: 创建日程
      console.log('\n🔸 Step 1/3: Create Schedule')
      console.log('  Command: task.create_with_schedule')
      const createPayload = {
        title: ctx.task.title,
        scheduled_day: targetDate,
        area_id: ctx.task.area_id,
        glance_note: ctx.task.glance_note,
      }
      console.log('  Payload:', createPayload)
      operations.push(createOperationRecord('create_schedule', ctx.targetViewId, createPayload))

      // 🎯 步骤 2: 从 Staging 移除（更新排序）
      console.log('\n🔸 Step 2/3: Remove from Staging')
      console.log('  Command: view.update_sorting')
      const sourceSorting = extractTaskIds(ctx.sourceContext)
      const newSourceSorting = removeTaskFrom(sourceSorting, ctx.task.id)
      const sourceSortPayload = {
        view_key: ctx.sourceViewId,
        sorted_task_ids: newSourceSorting,
        original_sorted_task_ids: sourceSorting,
      }
      console.log('  View:', ctx.sourceViewId)
      console.log('  Before:', sourceSorting.length, 'tasks')
      console.log('  After:', newSourceSorting.length, 'tasks')
      operations.push(createOperationRecord('update_sorting', ctx.sourceViewId, sourceSortPayload))

      // 🎯 步骤 3: 插入到 Daily（更新排序）
      console.log('\n🔸 Step 3/3: Insert to Daily')
      console.log('  Command: view.update_sorting')
      const targetSorting = extractTaskIds(ctx.targetContext)
      const newTargetSorting = insertTaskAt(targetSorting, ctx.task.id, ctx.dropIndex)
      const targetSortPayload = {
        view_key: ctx.targetViewId,
        sorted_task_ids: newTargetSorting,
        original_sorted_task_ids: targetSorting,
      }
      console.log('  View:', ctx.targetViewId)
      console.log('  Insert at index:', ctx.dropIndex ?? targetSorting.length)
      console.log('  Before:', targetSorting.length, 'tasks')
      console.log('  After:', newTargetSorting.length, 'tasks')
      operations.push(createOperationRecord('update_sorting', ctx.targetViewId, targetSortPayload))

      console.log('\n✅ All 3 operations planned')
      console.groupEnd()

      return {
        success: true,
        message: `[PRINT MODE] Would schedule to ${targetDate} with 3 operations`,
        operations,
        affectedViews: [ctx.sourceViewId, ctx.targetViewId],
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

      // 🔹 情况 A: 同日期重新排序
      if (isSameDay(ctx.sourceViewId, ctx.targetZone)) {
        console.group('🔄 [PRINT MODE] Daily → Daily (Same Day Reorder)')
        console.log(`📦 Task: "${ctx.task.title}"`)
        console.log(`📅 Date: ${sourceDate}`)
        console.log(`📌 New Index: ${ctx.dropIndex ?? 'append'}`)

        console.log('\n🔸 Step 1/1: Reorder in Same Day')
        console.log('  Command: view.update_sorting')
        const sorting = extractTaskIds(ctx.sourceContext)
        const newSorting = moveTaskWithin(sorting, ctx.task.id, ctx.dropIndex ?? sorting.length)
        const sortPayload = {
          view_key: ctx.sourceViewId,
          sorted_task_ids: newSorting,
          original_sorted_task_ids: sorting,
        }
        console.log('  View:', ctx.sourceViewId)
        console.log('  Before:', sorting)
        console.log('  After:', newSorting)
        operations.push(createOperationRecord('update_sorting', ctx.sourceViewId, sortPayload))

        console.log('\n✅ 1 operation planned')
        console.groupEnd()

        return {
          success: true,
          message: `[PRINT MODE] Would reorder in ${sourceDate}`,
          reorderOnly: true,
          operations,
          affectedViews: [ctx.sourceViewId],
        }
      }

      // 🔹 情况 B: 跨日期重新安排
      console.group('📆 [PRINT MODE] Daily → Daily (Cross-Day Reschedule)')
      console.log(`📦 Task: "${ctx.task.title}"`)
      console.log(`📤 From: ${sourceDate}`)
      console.log(`📥 To: ${targetDate}`)
      console.log(`📌 Drop Index: ${ctx.dropIndex ?? 'append'}`)

      // 🎯 步骤 1: 更新日程日期
      console.log('\n🔸 Step 1/3: Update Schedule Date')
      console.log('  Command: schedule.update')
      const updatePayload = {
        task_id: ctx.task.id,
        new_scheduled_day: targetDate,
      }
      console.log('  Payload:', updatePayload)
      operations.push(createOperationRecord('update_schedule', ctx.targetViewId, updatePayload))

      // 🎯 步骤 2: 从源 Daily 移除
      console.log('\n🔸 Step 2/3: Remove from Source Daily')
      console.log('  Command: view.update_sorting')
      const sourceSorting = extractTaskIds(ctx.sourceContext)
      const newSourceSorting = removeTaskFrom(sourceSorting, ctx.task.id)
      const sourceSortPayload = {
        view_key: ctx.sourceViewId,
        sorted_task_ids: newSourceSorting,
        original_sorted_task_ids: sourceSorting,
      }
      console.log('  View:', ctx.sourceViewId)
      console.log('  Before:', sourceSorting.length, 'tasks')
      console.log('  After:', newSourceSorting.length, 'tasks')
      operations.push(createOperationRecord('update_sorting', ctx.sourceViewId, sourceSortPayload))

      // 🎯 步骤 3: 插入到目标 Daily
      console.log('\n🔸 Step 3/3: Insert to Target Daily')
      console.log('  Command: view.update_sorting')
      const targetSorting = extractTaskIds(ctx.targetContext)
      const newTargetSorting = insertTaskAt(targetSorting, ctx.task.id, ctx.dropIndex)
      const targetSortPayload = {
        view_key: ctx.targetViewId,
        sorted_task_ids: newTargetSorting,
        original_sorted_task_ids: targetSorting,
      }
      console.log('  View:', ctx.targetViewId)
      console.log('  Insert at index:', ctx.dropIndex ?? targetSorting.length)
      console.log('  Before:', targetSorting.length, 'tasks')
      console.log('  After:', newTargetSorting.length, 'tasks')
      operations.push(createOperationRecord('update_sorting', ctx.targetViewId, targetSortPayload))

      console.log('\n✅ All 3 operations planned')
      console.groupEnd()

      return {
        success: true,
        message: `[PRINT MODE] Would reschedule from ${sourceDate} to ${targetDate}`,
        operations,
        affectedViews: [ctx.sourceViewId, ctx.targetViewId],
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

      console.group('↩️ [PRINT MODE] Daily → Staging (Return)')
      console.log(`📦 Task: "${ctx.task.title}"`)
      console.log(`📤 From: ${ctx.sourceViewId} (${sourceDate})`)
      console.log(`📥 To: ${ctx.targetViewId}`)
      console.log(`📌 Drop Index: ${ctx.dropIndex ?? 'append'}`)

      // 🎯 步骤 1: 删除日程
      console.log('\n🔸 Step 1/3: Delete Schedule')
      console.log('  Command: schedule.delete')
      const deletePayload = {
        task_id: ctx.task.id,
      }
      console.log('  Payload:', deletePayload)
      operations.push(createOperationRecord('delete_schedule', ctx.sourceViewId, deletePayload))

      // 🎯 步骤 2: 从 Daily 移除
      console.log('\n🔸 Step 2/3: Remove from Daily')
      console.log('  Command: view.update_sorting')
      const sourceSorting = extractTaskIds(ctx.sourceContext)
      const newSourceSorting = removeTaskFrom(sourceSorting, ctx.task.id)
      const sourceSortPayload = {
        view_key: ctx.sourceViewId,
        sorted_task_ids: newSourceSorting,
        original_sorted_task_ids: sourceSorting,
      }
      console.log('  View:', ctx.sourceViewId)
      console.log('  Before:', sourceSorting.length, 'tasks')
      console.log('  After:', newSourceSorting.length, 'tasks')
      operations.push(createOperationRecord('update_sorting', ctx.sourceViewId, sourceSortPayload))

      // 🎯 步骤 3: 插入到 Staging
      console.log('\n🔸 Step 3/3: Insert to Staging')
      console.log('  Command: view.update_sorting')
      const targetSorting = extractTaskIds(ctx.targetContext)
      const newTargetSorting = insertTaskAt(targetSorting, ctx.task.id, ctx.dropIndex)
      const targetSortPayload = {
        view_key: ctx.targetViewId,
        sorted_task_ids: newTargetSorting,
        original_sorted_task_ids: targetSorting,
      }
      console.log('  View:', ctx.targetViewId)
      console.log('  Insert at index:', ctx.dropIndex ?? targetSorting.length)
      console.log('  Before:', targetSorting.length, 'tasks')
      console.log('  After:', newTargetSorting.length, 'tasks')
      operations.push(createOperationRecord('update_sorting', ctx.targetViewId, targetSortPayload))

      console.log('\n✅ All 3 operations planned')
      console.groupEnd()

      return {
        success: true,
        message: `[PRINT MODE] Would return from ${sourceDate} to staging`,
        operations,
        affectedViews: [ctx.sourceViewId, ctx.targetViewId],
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

      console.group('🔄 [PRINT MODE] Daily Internal Reorder')
      console.log(`📦 Task: "${ctx.task.title}"`)
      console.log(`📅 Date: ${date}`)
      console.log(`📌 New Index: ${ctx.dropIndex ?? 'append'}`)

      console.log('\n🔸 Step 1/1: Reorder in Same Day')
      console.log('  Command: view.update_sorting')
      const sorting = extractTaskIds(ctx.sourceContext)
      const newSorting = moveTaskWithin(sorting, ctx.task.id, ctx.dropIndex ?? sorting.length)
      const sortPayload = {
        view_key: ctx.sourceViewId,
        sorted_task_ids: newSorting,
        original_sorted_task_ids: sorting,
      }
      console.log('  View:', ctx.sourceViewId)
      console.log('  Before:', sorting)
      console.log('  After:', newSorting)
      operations.push(createOperationRecord('update_sorting', ctx.sourceViewId, sortPayload))

      console.log('\n✅ 1 operation planned')
      console.groupEnd()

      return {
        success: true,
        message: `[PRINT MODE] Would reorder in ${date}`,
        reorderOnly: true,
        operations,
        affectedViews: [ctx.sourceViewId],
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

      console.group('🔄 [PRINT MODE] Staging Internal Reorder')
      console.log(`📦 Task: "${ctx.task.title}"`)
      console.log(`📌 New Index: ${ctx.dropIndex ?? 'append'}`)

      console.log('\n🔸 Step 1/1: Reorder in Staging')
      console.log('  Command: view.update_sorting')
      const sorting = extractTaskIds(ctx.targetContext)
      const newSorting = moveTaskWithin(sorting, ctx.task.id, ctx.dropIndex ?? sorting.length)
      const sortPayload = {
        view_key: ctx.targetZone,
        sorted_task_ids: newSorting,
        original_sorted_task_ids: sorting,
      }
      console.log('  View:', ctx.targetZone)
      console.log('  Before:', sorting)
      console.log('  After:', newSorting)
      operations.push(createOperationRecord('update_sorting', ctx.targetZone, sortPayload))

      console.log('\n✅ 1 operation planned')
      console.groupEnd()

      return {
        success: true,
        message: `[PRINT MODE] Would reorder in staging`,
        reorderOnly: true,
        operations,
        affectedViews: [ctx.sourceViewId],
      }
    },
  },

  tags: ['scheduling', 'staging', 'reorder'],
}

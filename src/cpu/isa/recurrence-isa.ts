/**
 * 循环规则指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 自动处理 correlation-id
 * 3. 统一的 commit 逻辑
 */

import type { ISADefinition } from '@cutie/cpu-pipeline'
import type { TaskRecurrence } from '@/types/dtos'
import { useViewStore } from '@/stores/view'
import { useTaskStore } from '@/stores/task'
import { useTimeBlockStore } from '@/stores/timeblock'
import * as recurrenceCore from '@/stores/recurrence/core'

export const RecurrenceISA: ISADefinition = {
  'recurrence.create': {
    meta: {
      description: '创建循环规则',
      category: 'system',
      resourceIdentifier: () => [],
      priority: 5,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.template_id?.trim()) {
        console.warn('❌ 模板ID不能为空')
        return false
      }
      if (!payload.rule?.trim()) {
        console.warn('❌ 循环规则不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: '/recurrences',
      body: (payload) => payload,
    },

    commit: async (result: TaskRecurrence) => {
      recurrenceCore.addOrUpdateRecurrence(result)
      // 🔥 创建循环规则后，立即刷新所有日历视图
      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },

  'recurrence.update': {
    meta: {
      description: '更新循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      // ✅ 只验证参数完整性，不验证数据存在性（由后端验证）
      if (!payload.id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置（动态 URL）
    request: {
      method: 'PATCH',
      url: (payload) => `/recurrences/${payload.id}`,
      body: (payload) => {
        const { id, ...updates } = payload
        return updates
      },
    },

    commit: async (result: TaskRecurrence) => {
      recurrenceCore.addOrUpdateRecurrence(result)
      // 🔥 更新循环规则后，立即刷新所有日历视图
      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },

  'recurrence.delete': {
    meta: {
      description: '删除循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      // ✅ 只验证参数完整性，不验证数据存在性（由后端验证）
      if (!payload.id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'DELETE',
      url: (payload) => `/recurrences/${payload.id}`,
    },

    commit: async (_result, payload) => {
      // 1. 清理前端的时间片（workaround：后端删除的时间片需要在前端也删除）
      const taskStore = useTaskStore()
      const timeBlockStore = useTimeBlockStore()

      // 1.1 找到所有属于该循环规则的未完成任务
      const recurrenceTasks = taskStore.allTasks.filter(
        (task) => task.recurrence_id === payload.id && !task.is_completed && !task.is_deleted
      )

      console.log(
        `🔄 [RECURRENCE_DELETE] Found ${recurrenceTasks.length} uncompleted tasks to clean up time blocks`
      )

      // 1.2 收集这些任务关联的时间片
      const taskIdsToClean = new Set(recurrenceTasks.map((t) => t.id))
      const timeBlocksToCheck = new Set<string>()

      // 收集所有可能受影响的时间片ID
      for (const task of recurrenceTasks) {
        if (task.schedules) {
          for (const schedule of task.schedules) {
            if (schedule.time_blocks) {
              for (const timeBlock of schedule.time_blocks) {
                timeBlocksToCheck.add(timeBlock.id)
              }
            }
          }
        }
      }

      console.log(`🔄 [RECURRENCE_DELETE] Found ${timeBlocksToCheck.size} time blocks to check`)

      // 1.3 检查每个时间片，如果它只关联被删除的任务，就删除它
      const timeBlocksToDelete: string[] = []

      for (const timeBlockId of timeBlocksToCheck) {
        const timeBlock = timeBlockStore.getTimeBlockById(timeBlockId)
        if (!timeBlock) continue

        // 检查这个时间片是否只关联了被删除的任务
        const linkedTasks = timeBlock.linked_tasks || []
        const hasOtherTasks = linkedTasks.some((task) => !taskIdsToClean.has(task.id))

        // 如果没有其他任务关联，就删除它（workaround：简化判断，信任后端已经做了来源检查）
        if (!hasOtherTasks) {
          timeBlocksToDelete.push(timeBlockId)
          console.log(
            `🔄 [RECURRENCE_DELETE] Will delete orphan time block ${timeBlockId} (only linked to deleted tasks)`
          )
        }
      }

      // 1.4 删除孤儿时间片
      if (timeBlocksToDelete.length > 0) {
        timeBlockStore.batchRemoveTimeBlocks_mut(timeBlocksToDelete)
        console.log(
          `🔄 [RECURRENCE_DELETE] Deleted ${timeBlocksToDelete.length} orphan time blocks`
        )
      }

      // 2. 从 store 中删除循环规则
      recurrenceCore.removeRecurrence(payload.id)

      // 3. 刷新所有日历视图
      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },

  'recurrence.fetch_all': {
    meta: {
      description: '获取所有循环规则',
      category: 'system',
      resourceIdentifier: () => [],
      priority: 3,
      timeout: 10000,
    },

    // 🔥 声明式请求配置
    request: {
      method: 'GET',
      url: '/recurrences',
    },

    commit: async (result: TaskRecurrence[]) => {
      recurrenceCore.clearAll()
      result.forEach((recurrence) => {
        recurrenceCore.addOrUpdateRecurrence(recurrence)
      })
    },
  },

  'recurrence.fetch_by_template': {
    meta: {
      description: '按模板ID获取循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`template:${payload.template_id}`],
      priority: 3,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.template_id?.trim()) {
        console.warn('❌ 模板ID不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'GET',
      url: (payload) => `/recurrences?template_id=${payload.template_id}`,
    },

    commit: async (result: TaskRecurrence[]) => {
      // 不清空全部，只更新相关的
      result.forEach((recurrence) => {
        recurrenceCore.addOrUpdateRecurrence(recurrence)
      })
    },
  },

  'recurrence.update_template_and_instances': {
    meta: {
      description: '批量更新模板和所有未完成实例',
      category: 'system',
      resourceIdentifier: (payload) => [`recurrence:${payload.recurrence_id}`],
      priority: 7,
      timeout: 30000, // 批量操作可能耗时较长
    },

    validate: async (payload) => {
      // ✅ 只验证参数完整性，不验证数据存在性（由后端验证）
      if (!payload.recurrence_id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'PATCH',
      url: (payload) => `/recurrences/${payload.recurrence_id}/template-and-instances`,
      body: (payload) => {
        const { recurrence_id, ...updates } = payload
        return updates
      },
    },

    commit: async (result) => {
      // 批量操作的结果通常包含统计信息，但不需要更新本地store
      // 因为具体的任务更新会通过SSE事件处理
      console.info('✅ 模板和实例批量更新完成:', result)
      // 🔥 批量更新后，立即刷新所有日历视图
      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },
}

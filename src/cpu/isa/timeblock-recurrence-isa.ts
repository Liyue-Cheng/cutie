/**
 * 时间块循环规则指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 自动处理 correlation-id
 * 3. 统一的 commit 逻辑
 */

import type { ISADefinition } from '@cutie/cpu-pipeline'
import type {
  EditTimeBlockRecurrencePayload,
  TimeBlockRecurrence,
  TimeBlockRecurrenceEditResult,
} from '@/types/dtos'
import { useViewStore } from '@/stores/view'
import { useTimeBlockStore } from '@/stores/timeblock'

// 简单的本地 store（不使用 pinia）
const timeBlockRecurrences = new Map<string, TimeBlockRecurrence>()

export function addOrUpdateTimeBlockRecurrence(recurrence: TimeBlockRecurrence) {
  timeBlockRecurrences.set(recurrence.id, recurrence)
}

export function removeTimeBlockRecurrence(id: string) {
  timeBlockRecurrences.delete(id)
}

export function getTimeBlockRecurrences(): TimeBlockRecurrence[] {
  return Array.from(timeBlockRecurrences.values())
}

export function getTimeBlockRecurrenceById(id: string): TimeBlockRecurrence | undefined {
  return timeBlockRecurrences.get(id)
}

function formatAsYmd(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function getUpcomingRange(days = 60): { startDate: string; endDate: string } {
  const now = new Date()
  const end = new Date(now)
  end.setDate(end.getDate() + days)
  return {
    startDate: formatAsYmd(now),
    endDate: formatAsYmd(end),
  }
}

export const TimeBlockRecurrenceISA: ISADefinition = {
  'timeblock-recurrence.create': {
    meta: {
      description: '创建时间块循环规则',
      category: 'system',
      resourceIdentifier: () => [],
      priority: 5,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.rule?.trim()) {
        console.warn('❌ 循环规则不能为空')
        return false
      }
      if (!payload.duration_minutes || payload.duration_minutes <= 0) {
        console.warn('❌ 时长必须大于0')
        return false
      }
      if (!payload.start_time_local?.trim()) {
        console.warn('❌ 开始时间不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: '/time-block-recurrences',
      body: (payload) => payload,
    },

    commit: async (result: TimeBlockRecurrence) => {
      addOrUpdateTimeBlockRecurrence(result)
      // 🔥 创建循环规则后，立即刷新时间块数据
      // 由于日历可能显示的是一周或更长的范围，需要获取一个较大的时间范围
      const timeBlockStore = useTimeBlockStore()
      const { startDate, endDate } = getUpcomingRange()

      await timeBlockStore.fetchTimeBlocksForRange(startDate, endDate)

      // 🔥 同时刷新任务视图（以保持一致性）
      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },

  'timeblock-recurrence.update': {
    meta: {
      description: '更新时间块循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`timeblock-recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'PATCH',
      url: (payload) => `/time-block-recurrences/${payload.id}`,
      body: (payload) => {
        const { id, ...updates } = payload
        return updates
      },
    },

    commit: async (result: TimeBlockRecurrence) => {
      addOrUpdateTimeBlockRecurrence(result)
      // 🔥 更新循环规则后，立即刷新时间块数据
      const timeBlockStore = useTimeBlockStore()
      const { startDate, endDate } = getUpcomingRange()

      await timeBlockStore.fetchTimeBlocksForRange(startDate, endDate)

      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },

  'timeblock-recurrence.delete': {
    meta: {
      description: '删除时间块循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`timeblock-recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'DELETE',
      url: (payload) => `/time-block-recurrences/${payload.id}`,
    },

    commit: async (_result: unknown, payload: { id: string }) => {
      const timeBlockStore = useTimeBlockStore()

      // 1. 找到所有属于该循环规则的时间块（通过 recurrence_id）
      const recurrenceTimeBlocks = timeBlockStore.allTimeBlocks.filter(
        (tb) => tb.recurrence_id === payload.id
      )

      console.log(
        `🔄 [TB_RECURRENCE_DELETE] Found ${recurrenceTimeBlocks.length} time blocks to remove from store`
      )

      // 2. 从前端 store 中删除这些时间块
      if (recurrenceTimeBlocks.length > 0) {
        const timeBlockIds = recurrenceTimeBlocks.map((tb) => tb.id)
        timeBlockStore.batchRemoveTimeBlocks_mut(timeBlockIds)
        console.log(
          `🔄 [TB_RECURRENCE_DELETE] Removed ${timeBlockIds.length} time blocks from store`
        )
      }

      // 3. 从本地缓存中删除循环规则
      removeTimeBlockRecurrence(payload.id)

      // 4. 刷新所有日历视图
      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },

  'timeblock-recurrence.list': {
    meta: {
      description: '获取时间块循环规则列表',
      category: 'read',
      resourceIdentifier: () => [],
      priority: 3,
      timeout: 10000,
    },

    validate: async () => true,

    request: {
      method: 'GET',
      url: '/time-block-recurrences',
    },

    commit: async (result: TimeBlockRecurrence[]) => {
      // 更新本地缓存
      for (const recurrence of result) {
        addOrUpdateTimeBlockRecurrence(recurrence)
      }
    },
  },

  'timeblock-recurrence.get': {
    meta: {
      description: '获取时间块循环规则详情',
      category: 'read',
      resourceIdentifier: (payload) => [`timeblock-recurrence:${payload.id}`],
      priority: 4,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'GET',
      url: (payload) => `/time-block-recurrences/${payload.id}`,
    },

    commit: async (result: TimeBlockRecurrence) => {
      addOrUpdateTimeBlockRecurrence(result)
    },
  },

  'timeblock-recurrence.stop': {
    meta: {
      description: '停止时间块循环',
      category: 'system',
      resourceIdentifier: (payload) => [`timeblock-recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      if (!payload.stop_date?.trim()) {
        console.warn('❌ 停止日期不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: (payload) => `/time-block-recurrences/${payload.id}/stop`,
      body: (payload) => ({ stop_date: payload.stop_date }),
    },

    commit: async (result: TimeBlockRecurrence, payload: { id: string; stop_date: string }) => {
      const timeBlockStore = useTimeBlockStore()

      // 1. 更新本地缓存
      addOrUpdateTimeBlockRecurrence(result)

      // 2. 从 store 中移除 stop_date 之后的时间块
      const stopDate = new Date(payload.stop_date + 'T00:00:00')
      const nextDay = new Date(stopDate)
      nextDay.setDate(nextDay.getDate() + 1)

      const timeBlocksToRemove = timeBlockStore.allTimeBlocks.filter((tb) => {
        if (tb.recurrence_id !== payload.id) return false
        const tbDate = new Date(tb.start_time)
        return tbDate >= nextDay
      })

      if (timeBlocksToRemove.length > 0) {
        const idsToRemove = timeBlocksToRemove.map((tb) => tb.id)
        timeBlockStore.batchRemoveTimeBlocks_mut(idsToRemove)
        console.log(
          `⏹️ [TB_RECURRENCE_STOP] Removed ${idsToRemove.length} time blocks after ${payload.stop_date}`
        )
      }

      // 3. 刷新视图
      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },

  'timeblock-recurrence.resume': {
    meta: {
      description: '继续时间块循环',
      category: 'system',
      resourceIdentifier: (payload) => [`timeblock-recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: (payload) => `/time-block-recurrences/${payload.id}/resume`,
    },

    commit: async (result: TimeBlockRecurrence) => {
      // 1. 更新本地缓存
      addOrUpdateTimeBlockRecurrence(result)

      // 2. 刷新时间块数据（新实例将通过懒加载生成）
      const timeBlockStore = useTimeBlockStore()
      const { startDate, endDate } = getUpcomingRange()

      await timeBlockStore.fetchTimeBlocksForRange(startDate, endDate)

      // 3. 刷新视图
      const viewStore = useViewStore()
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },

  'timeblock-recurrence.edit': {
    meta: {
      description: '编辑时间块循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`timeblock-recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload: EditTimeBlockRecurrencePayload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ 循环规则ID不能为空')
        return false
      }
      if (!payload.local_now?.trim()) {
        console.warn('❌ local_now 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: (payload) => `/time-block-recurrences/${payload.id}/edit`,
      body: (payload) => {
        const { id, ...rest } = payload
        return rest
      },
    },

    commit: async (result: TimeBlockRecurrenceEditResult) => {
      const timeBlockStore = useTimeBlockStore()
      const viewStore = useViewStore()

      addOrUpdateTimeBlockRecurrence(result.recurrence)

      if (Array.isArray(result.deleted_time_block_ids) && result.deleted_time_block_ids.length > 0) {
        timeBlockStore.batchRemoveTimeBlocks_mut(result.deleted_time_block_ids)
      }

      const { startDate, endDate } = getUpcomingRange()
      await timeBlockStore.fetchTimeBlocksForRange(startDate, endDate)
      await viewStore.refreshAllMountedDailyViewsImmediately()
    },
  },
}

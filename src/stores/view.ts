import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * View Store V6.0 - Daily 视图刷新管理
 *
 * 📋 职责：
 * - 管理已挂载的 daily 视图注册表
 * - 提供 LexoRank 排序功能
 * - 循环任务操作后刷新所有已挂载的 daily 视图
 */

export const useViewStore = defineStore('view', () => {
  // ============================================================
  // STATE
  // ============================================================

  /**
   * 已挂载的 daily 视图注册表
   * key: 'YYYY-MM-DD'
   * value: 引用计数（有多少列正在使用该日期）
   */
  const mountedDailyViews = ref(new Map<string, number>())

  /**
   * 刷新防抖/节流状态
   */
  let refreshDebounceTimer: ReturnType<typeof setTimeout> | null = null
  let isRefreshing = ref(false)

  /**
   * 刷新配置
   */
  const REFRESH_DEBOUNCE_DELAY = 300 // ms

  // ============================================================
  // GETTERS - LexoRank 排序
  // ============================================================

  /**
   * 应用 LexoRank 排序到任务列表
   * @param tasks 原始任务列表（已经过滤好的）
   * @param viewKey 视图标识
   * @returns 排序后的任务列表
   */
  function applySorting(tasks: TaskCard[], viewKey: string): TaskCard[] {
    const tasksWithRank: Array<{ task: TaskCard; rank: string }> = []
    const tasksWithoutRank: Array<{ task: TaskCard; originalIndex: number }> = []

    tasks.forEach((task, index) => {
      const rank = task.sort_positions?.[viewKey]
      if (rank) {
        tasksWithRank.push({ task, rank })
      } else {
        tasksWithoutRank.push({ task, originalIndex: index })
      }
    })

    // 如果没有任何任务有 rank，保持原顺序
    if (tasksWithRank.length === 0) {
      return tasks
    }

    tasksWithRank.sort((a, b) => a.rank.localeCompare(b.rank))
    tasksWithoutRank.sort((a, b) => a.originalIndex - b.originalIndex)

    return [
      ...tasksWithoutRank.map((entry) => entry.task),
      ...tasksWithRank.map((entry) => entry.task),
    ]
  }

  // ============================================================
  // Daily 视图注册与刷新
  // ============================================================

  function registerDailyView(date: string) {
    const current = mountedDailyViews.value.get(date) ?? 0
    mountedDailyViews.value.set(date, current + 1)
    logger.debug(LogTags.STORE_VIEW, 'Registered daily view', { date, count: current + 1 })
  }

  function unregisterDailyView(date: string) {
    const current = mountedDailyViews.value.get(date) ?? 0
    const next = Math.max(0, current - 1)
    if (next === 0) {
      mountedDailyViews.value.delete(date)
    } else {
      mountedDailyViews.value.set(date, next)
    }
    logger.debug(LogTags.STORE_VIEW, 'Unregistered daily view', { date, count: next })
  }

  /**
   * 刷新所有已挂载的 daily 视图（触发后端实例化服务）
   *
   * 特性：
   * - 🚀 并发刷新：使用 Promise.all 同时刷新所有日期
   * - ⏱️ 防抖机制：300ms 内的重复调用会被合并
   * - 🔒 防重入：正在刷新时的新调用会被忽略
   */
  async function refreshAllMountedDailyViews() {
    // 🔒 防重入：如果正在刷新，直接返回
    if (isRefreshing.value) {
      logger.debug(LogTags.STORE_VIEW, 'Refresh already in progress, skipping')
      return
    }

    // ⏱️ 防抖：清除之前的定时器，设置新的防抖定时器
    if (refreshDebounceTimer) {
      clearTimeout(refreshDebounceTimer)
    }

    return new Promise<void>((resolve) => {
      refreshDebounceTimer = setTimeout(async () => {
        try {
          isRefreshing.value = true
          await performConcurrentRefresh()
        } finally {
          isRefreshing.value = false
          refreshDebounceTimer = null
          resolve()
        }
      }, REFRESH_DEBOUNCE_DELAY)
    })
  }

  /**
   * 执行并发刷新的核心逻辑
   */
  async function performConcurrentRefresh() {
    // 延迟导入，避免循环依赖
    const { useTaskStore } = await import('@/stores/task')
    const taskStore = useTaskStore()

    const dates = Array.from(mountedDailyViews.value.keys())

    if (dates.length === 0) {
      logger.debug(LogTags.STORE_VIEW, 'No mounted daily views to refresh')
      return
    }

    logger.info(LogTags.STORE_VIEW, 'Starting concurrent refresh of mounted daily views', {
      dates,
      count: dates.length,
    })

    const startTime = performance.now()

    // 🚀 并发刷新所有日期
    const refreshPromises = dates.map(async (date) => {
      const dateStartTime = performance.now()
      try {
        await taskStore.refreshDailyTasks_DMA(date)

        const duration = performance.now() - dateStartTime
        logger.debug(LogTags.STORE_VIEW, 'Successfully refreshed daily view', {
          date,
          duration: `${duration.toFixed(1)}ms`,
        })

        return { date, success: true, duration }
      } catch (err) {
        const duration = performance.now() - dateStartTime
        const error = err instanceof Error ? err : new Error(String(err))

        logger.error(LogTags.STORE_VIEW, 'Failed to refresh daily view', error, {
          date,
          duration: `${duration.toFixed(1)}ms`,
        })

        return { date, success: false, duration, error }
      }
    })

    const results = await Promise.all(refreshPromises)

    // 统计结果
    const totalDuration = performance.now() - startTime
    const successCount = results.filter((r) => r.success).length
    const failureCount = results.length - successCount
    const avgDuration = results.reduce((sum, r) => sum + r.duration, 0) / results.length

    logger.info(LogTags.STORE_VIEW, 'Completed concurrent refresh of daily views', {
      totalDates: dates.length,
      successCount,
      failureCount,
      totalDuration: `${totalDuration.toFixed(1)}ms`,
      avgDuration: `${avgDuration.toFixed(1)}ms`,
    })

    if (failureCount > 0) {
      const failedDates = results.filter((r) => !r.success).map((r) => r.date)
      logger.warn(LogTags.STORE_VIEW, 'Some daily views failed to refresh', {
        failedDates,
        failureCount,
      })
    }
  }

  /**
   * 立即刷新所有已挂载的 daily 视图（绕过防抖，用于紧急情况）
   */
  async function refreshAllMountedDailyViewsImmediately() {
    // 取消现有的防抖定时器
    if (refreshDebounceTimer) {
      clearTimeout(refreshDebounceTimer)
      refreshDebounceTimer = null
    }

    // 如果正在刷新，等待完成
    if (isRefreshing.value) {
      logger.debug(
        LogTags.STORE_VIEW,
        'Waiting for current refresh to complete before immediate refresh'
      )
      while (isRefreshing.value) {
        await new Promise((resolve) => setTimeout(resolve, 50))
      }
    }

    // 立即执行刷新
    try {
      isRefreshing.value = true
      await performConcurrentRefresh()
    } finally {
      isRefreshing.value = false
    }
  }

  return {
    // STATE
    isRefreshing,

    // GETTERS
    applySorting,

    // Daily 视图注册与刷新
    registerDailyView,
    unregisterDailyView,
    refreshAllMountedDailyViews,
    refreshAllMountedDailyViewsImmediately,
  }
})

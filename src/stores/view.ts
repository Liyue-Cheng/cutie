import { ref, nextTick } from 'vue'
import { defineStore } from 'pinia'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import { apiGet } from '@/stores/shared'

/**
 * View Store V5.0 - 纯状态容器 (Frontend-as-a-CPU 架构)
 *
 * 📋 架构原则：
 * - ✅ State: 寄存器 (只存储数据)
 * - ✅ Mutations: 寄存器写入操作 (_mut 后缀)
 * - ✅ Getters: 导线/多路复用器 (_Mux 后缀)
 * - ❌ 不包含 API 调用（由 Command Handler 负责）
 * - ❌ 不包含业务逻辑（由 Command Handler 负责）
 *
 * 职责：
 * - 只管理视图的排序信息
 * - 不存储任务数据（由 TaskStore 负责）
 * - 不存储任务ID列表（过滤由 TaskStore getter 负责）
 * - 只存储排序权重（持久化由 Command Handler 负责）
 *
 * 数据流：
 * 1. 组件触发指令 → pipeline.dispatch('viewpreference.update_sorting', ...)
 * 2. EX 阶段乐观更新 → viewStore.updateSortingOptimistic_mut(...)
 * 3. EX 阶段调用 API
 * 4. 成功 → WB commit | 失败 → WB 回滚
 */

export const useViewStore = defineStore('view', () => {
  // ============================================================
  // STATE - 只存储排序权重
  // ============================================================

  /**
   * 视图排序权重
   * key: 视图标识 (如 'staging', 'planned', 'daily::2024-10-01')
   * value: Map<taskId, weight>
   */
  const sortWeights = ref(new Map<string, Map<string, number>>())

  /**
   * 加载状态
   */
  const isLoading = ref(false)

  /**
   * 错误信息
   */
  const error = ref<string | null>(null)

  /**
   * 🆕 已挂载的 daily 视图注册表
   * key: 'YYYY-MM-DD'
   * value: 引用计数（有多少列正在使用该日期）
   */
  const mountedDailyViews = ref(new Map<string, number>())

  /**
   * 🆕 刷新防抖/节流状态
   */
  let refreshDebounceTimer: ReturnType<typeof setTimeout> | null = null
  let isRefreshing = ref(false)

  /**
   * 🆕 刷新配置
   */
  const REFRESH_DEBOUNCE_DELAY = 300 // ms

  /**
   * 🆕 批量更新防抖机制
   * 缓存待更新的排序，在下一个tick统一应用
   */
  let pendingUpdates = new Map<string, Map<string, number>>()
  let updateScheduled = false

  // ============================================================
  // GETTERS (Wires / Multiplexers) - 只读数据选择
  // ============================================================

  /**
   * 应用排序到任务列表 (Multiplexer)
   * @param tasks 原始任务列表（已经过滤好的）
   * @param viewKey 视图标识
   * @returns 排序后的任务列表
   *
   * 性能优化：
   * - 使用 Map 替代 indexOf，避免 O(n²) 复杂度
   * - 预先构建索引，排序时 O(1) 查找
   */
  function applySorting(tasks: TaskCard[], viewKey: string): TaskCard[] {
    const weights = sortWeights.value.get(viewKey)

    if (!weights || weights.size === 0) {
      // 如果没有排序信息，保持原顺序
      return tasks
    }

    // ✅ 性能优化：预先构建原顺序索引 Map（O(n)）
    const originalIndexMap = new Map<string, number>()
    tasks.forEach((task, index) => {
      originalIndexMap.set(task.id, index)
    })

    // ✅ 排序时使用 Map 查找（O(1)），而不是 indexOf（O(n)）
    const sorted = [...tasks].sort((a, b) => {
      const weightA = weights.get(a.id) ?? Infinity
      const weightB = weights.get(b.id) ?? Infinity

      if (weightA === weightB) {
        // O(1) 查找，而不是 O(n)
        const indexA = originalIndexMap.get(a.id) ?? 0
        const indexB = originalIndexMap.get(b.id) ?? 0
        return indexA - indexB
      }

      return weightA - weightB
    })

    return sorted
  }

  /**
   * 获取当前视图的排序ID列表（用于持久化）
   * @param viewKey 视图标识
   * @param tasks 当前任务列表
   * @returns 排序后的任务ID数组
   */
  function getSortedTaskIds(viewKey: string, tasks: TaskCard[]): string[] {
    const sorted = applySorting(tasks, viewKey)
    return sorted.map((t) => t.id)
  }

  // ============================================================
  // MUTATIONS (Register Write Operations) - 纯状态更新
  // ============================================================

  /**
   * 🔥 乐观更新排序（立即更新本地状态）
   * @param viewKey 视图标识
   * @param orderedTaskIds 新的任务ID顺序
   *
   * ⚠️ 此函数只更新本地状态，不调用 API
   * ⚠️ 应由 Command Handler 调用
   */
  function updateSortingOptimistic_mut(viewKey: string, orderedTaskIds: string[]): void {
    // 构建权重映射
    const weights = new Map<string, number>()
    orderedTaskIds.forEach((id, index) => {
      weights.set(id, index)
    })

    // 更新本地状态
    const newMap = new Map(sortWeights.value)
    newMap.set(viewKey, weights)
    sortWeights.value = newMap

    logger.debug(LogTags.STORE_VIEW, 'Optimistic sorting update applied', {
      viewKey,
      taskCount: orderedTaskIds.length,
    })
  }

  /**
   * ❌ 已废弃：旧的 updateSorting 方法
   * 请使用 pipeline.dispatch('viewpreference.update_sorting', ...) 代替
   *
   * @deprecated 使用 CPU Pipeline 代替直接调用
   */
  async function updateSorting(viewKey: string, orderedTaskIds: string[]): Promise<boolean> {
    logger.warn(
      LogTags.STORE_VIEW,
      '⚠️ DEPRECATED: Direct updateSorting call detected. Use pipeline.dispatch("viewpreference.update_sorting") instead',
      { viewKey }
    )

    // 为了向后兼容，临时保留实现
    // 🔥 TODO: 移除此方法，强制使用 Command Bus
    updateSortingOptimistic_mut(viewKey, orderedTaskIds)
    return true
  }

  /**
   * 加载排序配置（从后端加载时调用）
   * 🆕 使用防抖批量更新，避免多次触发响应式重新计算
   * @param viewKey 视图标识
   * @param orderedTaskIds 保存的任务ID顺序
   */
  function loadSorting(viewKey: string, orderedTaskIds: string[]) {
    const weights = new Map<string, number>()
    orderedTaskIds.forEach((id, index) => {
      weights.set(id, index)
    })

    // ✅ 缓存待更新的数据
    pendingUpdates.set(viewKey, weights)

    // ✅ 如果还没有调度更新，在下一个tick批量应用所有更新
    if (!updateScheduled) {
      updateScheduled = true
      nextTick(() => {
        // 一次性应用所有缓存的更新
        const newMap = new Map(sortWeights.value)
        pendingUpdates.forEach((weights, key) => {
          newMap.set(key, weights)
        })
        sortWeights.value = newMap

        // 清理
        pendingUpdates.clear()
        updateScheduled = false
      })
    }
  }

  /**
   * 🆕 批量加载多个视图的排序配置
   * @param viewKeys 视图标识数组
   * @returns 成功加载的数量
   */
  async function batchFetchViewPreferences(viewKeys: string[]): Promise<number> {
    const results = await Promise.all(viewKeys.map((key) => fetchViewPreference(key)))
    const successCount = results.filter((r) => r).length
    return successCount
  }

  /**
   * 从后端加载视图的排序配置
   * @param viewKey 视图标识（必须符合 VIEW_CONTEXT_KEY_SPEC 规范，如 'misc::staging', 'daily::2025-10-01'）
   */
  async function fetchViewPreference(viewKey: string): Promise<boolean> {
    try {
      const data = await apiGet<{
        context_key: string
        sorted_task_ids: string[]
        updated_at: string
      }>(`/view-preferences/${encodeURIComponent(viewKey)}`)

      // 加载排序配置
      loadSorting(viewKey, data.sorted_task_ids)

      return true
    } catch (err) {
      // 404 表示没有保存的配置，静默处理
      if (err instanceof Error && err.message.includes('404')) {
        return true
      }

      logger.error(
        LogTags.STORE_VIEW,
        'Failed to fetch preference',
        err instanceof Error ? err : new Error(String(err)),
        { viewKey }
      )
      return false
    }
  }

  /**
   * 清除指定视图的排序
   * @param viewKey 视图标识
   */
  function clearSorting(viewKey: string) {
    const newMap = new Map(sortWeights.value)
    newMap.delete(viewKey)
    sortWeights.value = newMap
    logger.debug(LogTags.STORE_VIEW, 'Cleared sorting for view', { viewKey })
  }

  /**
   * 清除所有排序
   */
  function clearAllSorting() {
    sortWeights.value = new Map()
    logger.debug(LogTags.STORE_VIEW, 'Cleared all sorting')
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
   * - 📊 详细日志：记录刷新过程和结果统计
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
        // 使用 refreshDailyTasks_DMA 进行替换式刷新
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

    // 等待所有刷新完成
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
      results: results.map((r) => ({
        date: r.date,
        success: r.success,
        duration: `${r.duration.toFixed(1)}ms`,
      })),
    })

    // 如果有失败的刷新，记录警告
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
   *
   * 使用场景：
   * - 用户手动触发的刷新操作
   * - 关键业务操作后需要立即看到结果
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
      // 简单的轮询等待，实际项目中可以用更优雅的方式
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
    // ============================================================
    // STATE (Registers) - 只读状态
    // ============================================================
    sortWeights,
    isLoading,
    error,
    isRefreshing,

    // ============================================================
    // GETTERS (Wires / Multiplexers) - 数据选择
    // ============================================================
    applySorting,
    getSortedTaskIds,

    // ============================================================
    // MUTATIONS (Register Write Operations) - 状态更新
    // ============================================================
    updateSortingOptimistic_mut, // 🔥 乐观更新（由 Command Handler 调用）
    clearSorting,
    clearAllSorting,
    loadSorting, // 从后端加载时调用（批量防抖）

    // ============================================================
    // DMA (Direct Memory Access) - 数据加载
    // ============================================================
    fetchViewPreference, // 从后端加载单个视图
    batchFetchViewPreferences, // 批量加载多个视图

    // ============================================================
    // DEPRECATED - 向后兼容
    // ============================================================
    updateSorting, // ❌ 已废弃，使用 pipeline.dispatch('viewpreference.update_sorting') 代替

    // ============================================================
    // Daily 视图注册与刷新
    // ============================================================
    registerDailyView,
    unregisterDailyView,
    refreshAllMountedDailyViews,
    refreshAllMountedDailyViewsImmediately,
  }
})

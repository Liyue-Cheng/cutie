import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { TaskCard } from '@/types/dtos'
import { waitForApiReady } from '@/composables/useApiConfig'

/**
 * View Store V4.0 - 纯排序系统
 *
 * 职责：只管理视图的排序信息
 * - 不存储任务数据（由 TaskStore 负责）
 * - 不存储任务ID列表（过滤由 TaskStore getter 负责）
 * - 只存储排序权重（持久化到后端）
 *
 * 架构原则：
 * - 过滤逻辑 → TaskStore 动态计算
 * - 排序信息 → ViewStore 持久化
 * - 完全分离关注点
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

  // ============================================================
  // ACTIONS - 排序管理
  // ============================================================

  /**
   * 应用排序到任务列表
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
    return [...tasks].sort((a, b) => {
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
  }

  /**
   * 更新排序（拖拽时调用）
   * @param viewKey 视图标识
   * @param orderedTaskIds 新的任务ID顺序
   */
  async function updateSorting(viewKey: string, orderedTaskIds: string[]): Promise<boolean> {
    try {
      // 构建权重映射
      const weights = new Map<string, number>()
      orderedTaskIds.forEach((id, index) => {
        weights.set(id, index)
      })

      // 更新本地状态
      const newMap = new Map(sortWeights.value)
      newMap.set(viewKey, weights)
      sortWeights.value = newMap

      // ✅ 持久化到后端
      const contextKey = `misc::${viewKey}` // 使用规范格式
      console.log('[ViewStore] 💾 Saving to backend:', {
        context_key: contextKey,
        task_count: orderedTaskIds.length,
      })

      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/view-preferences`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          context_key: contextKey,
          sorted_task_ids: orderedTaskIds,
        }),
      })

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }

      console.log('[ViewStore] ✅ Saved successfully')
      return true
    } catch (err) {
      console.error('[ViewStore] Failed to update sorting:', err)
      error.value = `Failed to update sorting: ${err}`
      return false
    }
  }

  /**
   * 加载排序配置（从后端加载时调用）
   * @param viewKey 视图标识
   * @param orderedTaskIds 保存的任务ID顺序
   */
  function loadSorting(viewKey: string, orderedTaskIds: string[]) {
    const weights = new Map<string, number>()
    orderedTaskIds.forEach((id, index) => {
      weights.set(id, index)
    })

    const newMap = new Map(sortWeights.value)
    newMap.set(viewKey, weights)
    sortWeights.value = newMap

    console.log(`[ViewStore] Loaded sorting for ${viewKey}:`, orderedTaskIds.length, 'tasks')
  }

  /**
   * 从后端加载视图的排序配置
   * @param viewKey 视图标识（如 'all', 'staging', 'planned'）
   */
  async function fetchViewPreference(viewKey: string): Promise<boolean> {
    try {
      const apiBaseUrl = await waitForApiReady()
      const contextKey = `misc::${viewKey}` // 使用规范格式

      console.log(`[ViewStore] 📥 Fetching preference for: ${contextKey}`)

      const response = await fetch(
        `${apiBaseUrl}/view-preferences/${encodeURIComponent(contextKey)}`
      )

      if (response.status === 404) {
        // 没有保存的配置，使用默认顺序
        console.log(`[ViewStore] No saved preference for ${contextKey}`)
        return true
      }

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }

      const result = await response.json()
      const data = result.data as {
        context_key: string
        sorted_task_ids: string[]
        updated_at: string
      }

      // 加载排序配置
      loadSorting(viewKey, data.sorted_task_ids)

      console.log(
        `[ViewStore] ✅ Loaded preference for ${contextKey}:`,
        data.sorted_task_ids.length,
        'tasks'
      )
      return true
    } catch (err) {
      console.error(`[ViewStore] Failed to fetch preference for ${viewKey}:`, err)
      return false
    }
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

  /**
   * 清除指定视图的排序
   * @param viewKey 视图标识
   */
  function clearSorting(viewKey: string) {
    const newMap = new Map(sortWeights.value)
    newMap.delete(viewKey)
    sortWeights.value = newMap
    console.log(`[ViewStore] Cleared sorting for ${viewKey}`)
  }

  /**
   * 清除所有排序
   */
  function clearAllSorting() {
    sortWeights.value = new Map()
    console.log('[ViewStore] Cleared all sorting')
  }

  return {
    // State
    sortWeights,
    isLoading,
    error,

    // Actions
    applySorting,
    updateSorting,
    loadSorting,
    fetchViewPreference,
    getSortedTaskIds,
    clearSorting,
    clearAllSorting,
  }
})

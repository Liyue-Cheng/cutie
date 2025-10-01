/**
 * 视图操作层
 *
 * 职责：
 * - 提供统一的视图加载API
 * - 协调 ViewStore 和 TaskStore 的数据同步
 *
 * 新架构说明：
 * - 只需加载任务数据到 TaskStore
 * - ViewStore 的排序配置单独加载（待后端 API 完成）
 */

import { useTaskStore } from '@/stores/task'
import { fetchView, type ViewContext } from '@/services/viewAdapter'

export function useViewOperations() {
  const taskStore = useTaskStore()

  /**
   * 加载视图数据
   * 统一的视图加载入口
   */
  async function loadView(context: ViewContext): Promise<boolean> {
    try {
      // 获取并缓存任务数据
      const tasks = await fetchView(context)
      taskStore.addOrUpdateTasks(tasks)

      console.log(`[ViewOperations] Loaded view:`, context, `- ${tasks.length} tasks`)

      // TODO: 加载该视图的排序配置
      // const contextKey = getContextKey(context)
      // const preference = await fetchViewPreference(contextKey)
      // if (preference) {
      //   const taskIds = JSON.parse(preference.sorted_task_ids)
      //   viewStore.loadSorting(contextKey, taskIds)
      // }

      return true
    } catch (error) {
      console.error('[ViewOperations] Error loading view:', context, error)
      return false
    }
  }

  /**
   * 便捷方法：加载所有任务
   */
  async function loadAllTasks() {
    return loadView({ type: 'all' })
  }

  /**
   * 便捷方法：加载所有未完成任务
   */
  async function loadAllIncompleteTasks() {
    return loadView({ type: 'all_incomplete' })
  }

  /**
   * 便捷方法：加载 Staging 区任务
   */
  async function loadStagingTasks() {
    return loadView({ type: 'staging' })
  }

  /**
   * 便捷方法：加载已排期任务
   */
  async function loadPlannedTasks() {
    return loadView({ type: 'planned' })
  }

  /**
   * 便捷方法：加载每日看板
   */
  async function loadDailyKanban(date: string) {
    return loadView({ type: 'daily_kanban', date })
  }

  return {
    loadView,
    loadAllTasks,
    loadAllIncompleteTasks,
    loadStagingTasks,
    loadPlannedTasks,
    loadDailyKanban,
  }
}

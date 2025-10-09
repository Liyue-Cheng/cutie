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
import { logger, LogTags } from '@/services/logger'
// import { fetchView, type ViewContext } from '@/services/viewAdapter'
// 注意：fetchView 函数不存在，暂时注释掉避免错误

export function useViewOperations() {
  const taskStore = useTaskStore()

  /**
   * 加载视图数据
   * 统一的视图加载入口
   */
  async function loadView(context: any): Promise<boolean> {
    try {
      // 🚧 临时实现：直接调用 taskStore.fetchAllTasks()
      // 因为 fetchView 函数不存在，我们使用现有的 API
      await taskStore.fetchAllTasks()

      logger.info(LogTags.STORE_VIEW, 'Loaded all tasks (temporary implementation)')

      return true
    } catch (error) {
      logger.error(
        LogTags.STORE_VIEW,
        'Error loading tasks',
        error instanceof Error ? error : new Error(String(error))
      )
      return false
    }
  }

  /**
   * 便捷方法：加载所有任务
   */
  async function loadAllTasks() {
    return loadView({})
  }

  /**
   * 便捷方法：加载所有未完成任务
   */
  async function loadAllIncompleteTasks() {
    return loadView({})
  }

  /**
   * 便捷方法：加载 Staging 区任务
   */
  async function loadStagingTasks() {
    return loadView({})
  }

  /**
   * 便捷方法：加载已排期任务
   */
  async function loadPlannedTasks() {
    return loadView({})
  }

  /**
   * 便捷方法：加载每日看板
   */
  async function loadDailyKanban(date: string) {
    return loadView({})
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

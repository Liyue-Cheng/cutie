import type { TaskCard } from '@/types/dtos'
import { apiGet } from '@/stores/shared'
import type { createTaskCore } from './core'
import { logger, LogTags } from '@/services/logger'

/**
 * Task Store 视图操作
 *
 * 职责：
 * - 获取各种视图的任务列表
 * - 处理视图相关的 API 调用
 * - 更新核心状态
 */

/**
 * 创建视图操作功能
 */
export function createViewOperations(core: ReturnType<typeof createTaskCore>) {
  const { addOrUpdateTasks, replaceTasksForDate, withLoading } = core

  /**
   * 获取所有任务（包括已完成）
   * API: GET /views/all
   */
  async function fetchAllTasks() {
    return withLoading(async () => {
      const tasks: TaskCard[] = await apiGet('/views/all')
      addOrUpdateTasks(tasks)
      logger.info(LogTags.STORE_TASKS, 'Fetched all tasks', { count: tasks.length })
      return tasks
    }, 'fetch all tasks')
  }

  /**
   * 获取所有未完成任务
   * API: GET /views/all-incomplete
   */
  async function fetchAllIncompleteTasks() {
    return withLoading(async () => {
      const tasks: TaskCard[] = await apiGet('/views/all-incomplete')
      addOrUpdateTasks(tasks)
      logger.info(LogTags.STORE_TASKS, 'Fetched incomplete tasks', { count: tasks.length })
      return tasks
    }, 'fetch incomplete tasks')
  }

  /**
   * 获取已排期任务
   * API: GET /views/planned
   */
  async function fetchPlannedTasks() {
    return withLoading(async () => {
      const tasks: TaskCard[] = await apiGet('/views/planned')
      addOrUpdateTasks(tasks)
      logger.info(LogTags.STORE_TASKS, 'Fetched planned tasks', { count: tasks.length })
      return tasks
    }, 'fetch planned tasks')
  }

  /**
   * 获取 Staging 区的任务
   * API: GET /views/staging
   */
  async function fetchStagingTasks() {
    return withLoading(async () => {
      const stagingTasks: TaskCard[] = await apiGet('/views/staging')
      addOrUpdateTasks(stagingTasks)
      logger.info(LogTags.STORE_TASKS, 'Fetched staging tasks', { count: stagingTasks.length })
      return stagingTasks
    }, 'fetch staging tasks')
  }

  /**
   * 获取指定日期的任务
   * API: GET /views/daily/:date
   */
  async function fetchDailyTasks(date: string): Promise<TaskCard[]> {
    const result = await withLoading(async () => {
      const response: { tasks: TaskCard[]; date: string; count: number } = await apiGet(
        `/views/daily/${date}`
      )
      addOrUpdateTasks(response.tasks)
      logger.info(LogTags.STORE_TASKS, 'Fetched tasks for date', {
        count: response.tasks.length,
        date,
      })
      return response.tasks
    }, `fetch tasks for ${date}`)
    return result ?? []
  }

  /**
   * 刷新指定日期的任务（替换式更新）
   * API: GET /views/daily/:date
   *
   * 与 fetchDailyTasks 的区别：
   * - fetchDailyTasks: 追加/更新任务（适合初次加载）
   * - refreshDailyTasks: 先清理该日期的旧任务，再添加新任务（适合刷新场景）
   */
  async function refreshDailyTasks(date: string): Promise<TaskCard[]> {
    const result = await withLoading(async () => {
      const response: { tasks: TaskCard[]; date: string; count: number } = await apiGet(
        `/views/daily/${date}`
      )

      // 🔥 替换式更新：先清理该日期的旧任务，再添加新任务
      replaceTasksForDate(date, response.tasks)

      logger.info(LogTags.STORE_TASKS, 'Refreshed and replaced tasks for date', {
        count: response.tasks.length,
        date,
      })
      return response.tasks
    }, `refresh tasks for ${date}`)
    return result ?? []
  }

  /**
   * 搜索任务
   * API: GET /tasks/search?q=...
   */
  async function searchTasks(query: string, limit?: number): Promise<TaskCard[]> {
    const result = await withLoading(async () => {
      // TODO: 实现 API 调用
      // const params = new URLSearchParams({ q: query })
      // if (limit) params.append('limit', limit.toString())
      // const results: TaskCard[] = await apiGet(`/tasks/search?${params}`)
      // addOrUpdateTasks(results)
      // return results

      logger.info(LogTags.STORE_TASKS, 'searchTasks - API not implemented yet', { query, limit })
      return []
    }, 'search tasks')

    return result ?? []
  }

  return {
    fetchAllTasks,
    fetchAllIncompleteTasks,
    fetchPlannedTasks,
    fetchStagingTasks,
    fetchDailyTasks,
    refreshDailyTasks,
    searchTasks,
  }
}

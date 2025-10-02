import type { TaskCard } from '@/types/dtos'
import { apiGet } from '@/stores/shared'
import type { createTaskCore } from './core'

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
  const { addOrUpdateTasks, withLoading } = core

  /**
   * 获取所有任务（包括已完成）
   * API: GET /views/all
   */
  async function fetchAllTasks() {
    return withLoading(async () => {
      const tasks: TaskCard[] = await apiGet('/views/all')
      addOrUpdateTasks(tasks)
      console.log('[TaskStore] Fetched', tasks.length, 'all tasks')
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
      console.log('[TaskStore] Fetched', tasks.length, 'incomplete tasks')
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
      console.log('[TaskStore] Fetched', tasks.length, 'planned tasks')
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
      console.log('[TaskStore] Fetched', stagingTasks.length, 'staging tasks')
      return stagingTasks
    }, 'fetch staging tasks')
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

      console.log('[TaskStore] searchTasks - API not implemented yet', { query, limit })
      return []
    }, 'search tasks')

    return result ?? []
  }

  return {
    fetchAllTasks,
    fetchAllIncompleteTasks,
    fetchPlannedTasks,
    fetchStagingTasks,
    searchTasks,
  }
}

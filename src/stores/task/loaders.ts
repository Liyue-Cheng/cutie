/**
 * Task Store DMA 控制器（Direct Memory Access）
 *
 * 职责：
 * - 从后端批量加载数据，直接写入 Store（绕过 Command Bus）
 * - ❌ 不执行修改操作（用 Command Bus）
 * - ❌ 不包含业务逻辑
 * - ✅ 只用于应用启动时的初始化加载
 *
 * RTL 命名规范：
 * - fetchXXX_DMA - DMA 传输方法（类比硬件的 DMA Controller）
 *
 * 类比：
 * - 就像 CPU 的 DMA 控制器，绕过 CPU 流水线，直接将数据写入内存
 * - 提高效率，减少 CPU（Command Bus）负担
 */

import type { TaskCard, TaskDetail } from '@/types/dtos'
import { apiGet } from '@/stores/shared'
import type { createTaskCore } from './core'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 创建数据加载功能
 */
export function createLoaders(core: ReturnType<typeof createTaskCore>) {
  const { addOrUpdateTasks, replaceTasksForDate, addOrUpdateTask, withLoading } = core

  /**
   * 🚨 fetchAllTasks_DMA 已删除！
   *
   * 原因：随着循环任务的实现，查询所有任务会导致性能问题和潜在的无限数据。
   *
   * 请使用以下替代方案：
   * - fetchAllIncompleteTasks_DMA() - 查询未完成任务
   * - fetchStagingTasks_DMA() - 查询暂存区任务
   * - fetchDailyTasks_DMA(date) - 查询特定日期任务
   * - fetchPlannedTasks_DMA() - 查询已排期任务
   */

  /**
   * DMA: 加载所有未完成任务
   * API: GET /views/all-incomplete
   */
  async function fetchAllIncompleteTasks_DMA() {
    return withLoading(async () => {
      const tasks: TaskCard[] = await apiGet('/views/all-incomplete')
      addOrUpdateTasks(tasks)
      logger.info(LogTags.STORE_TASKS, 'DMA: Loaded incomplete tasks', { count: tasks.length })
      return tasks
    }, 'fetch incomplete tasks')
  }

  /**
   * DMA: 加载已排期任务（planned 视图）
   * API: GET /views/planned
   */
  async function fetchPlannedTasks_DMA() {
    return withLoading(async () => {
      const tasks: TaskCard[] = await apiGet('/views/planned')
      addOrUpdateTasks(tasks)
      logger.info(LogTags.STORE_TASKS, 'DMA: Loaded planned tasks', { count: tasks.length })
      return tasks
    }, 'fetch planned tasks')
  }

  /**
   * DMA: 加载暂存区任务（staging 视图）
   * API: GET /views/staging
   */
  async function fetchStagingTasks_DMA() {
    return withLoading(async () => {
      const tasks: TaskCard[] = await apiGet('/views/staging')
      addOrUpdateTasks(tasks)
      logger.info(LogTags.STORE_TASKS, 'DMA: Loaded staging tasks', { count: tasks.length })
      return tasks
    }, 'fetch staging tasks')
  }

  /**
   * DMA: 加载指定日期的任务（每日看板）
   * API: GET /views/daily?date=YYYY-MM-DD
   */
  async function fetchDailyTasks_DMA(date: string) {
    return withLoading(async () => {
      const response: { tasks: TaskCard[]; date: string; count: number } = await apiGet(
        `/views/daily/${date}`
      )
      addOrUpdateTasks(response.tasks)
      logger.info(LogTags.STORE_TASKS, 'DMA: Loaded daily tasks', {
        date,
        count: response.tasks.length,
      })
      return response.tasks
    }, `fetch daily tasks for ${date}`)
  }

  /**
   * DMA: 刷新指定日期的任务
   * API: GET /views/daily?date=YYYY-MM-DD
   */
  async function refreshDailyTasks_DMA(date: string) {
    return withLoading(async () => {
      const response: { tasks: TaskCard[]; date: string; count: number } = await apiGet(
        `/views/daily/${date}`
      )
      // 🔥 替换式更新：先清理该日期的旧任务，再添加新任务
      replaceTasksForDate(date, response.tasks)
      logger.info(LogTags.STORE_TASKS, 'DMA: Refreshed daily tasks', {
        date,
        count: response.tasks.length,
      })
      return response.tasks
    }, `refresh daily tasks for ${date}`)
  }

  /**
   * DMA: 加载任务详情（编辑器使用）
   * API: GET /tasks/:id
   */
  async function fetchTaskDetail_DMA(id: string): Promise<TaskDetail | null> {
    return withLoading(async () => {
      const taskDetail: TaskDetail = await apiGet(`/tasks/${id}`)
      addOrUpdateTask(taskDetail)
      logger.info(LogTags.STORE_TASKS, 'DMA: Loaded task detail', {
        taskId: taskDetail.id,
        title: taskDetail.title,
      })
      return taskDetail
    }, `fetch task detail ${id}`)
  }

  /**
   * DMA: 搜索任务
   * API: GET /tasks/search?q=...
   */
  async function searchTasks_DMA(query: string, limit?: number): Promise<TaskCard[]> {
    const result = await withLoading(async () => {
      // TODO: 实现 API 调用
      logger.info(LogTags.STORE_TASKS, 'DMA: searchTasks - API not implemented yet', {
        query,
        limit,
      })
      return []
    }, 'search tasks')

    return result ?? []
  }

  return {
    // fetchAllTasks_DMA, // 🚨 已删除 - 危险操作，会导致无限数据
    fetchAllIncompleteTasks_DMA,
    fetchPlannedTasks_DMA,
    fetchStagingTasks_DMA,
    fetchDailyTasks_DMA,
    refreshDailyTasks_DMA,
    fetchTaskDetail_DMA,
    searchTasks_DMA,
  }
}

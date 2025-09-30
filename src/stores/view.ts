import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from './task'
// import { waitForApiReady } from '@/composables/useApiConfig'

/**
 * View Store
 *
 * 职责：管理视图和上下文
 * - 管理"哪天有哪些任务"
 * - 管理"这些任务的顺序"
 * - 协调 TaskStore 和视图层
 *
 * 架构原则：
 * - State: 存储视图相关的索引数据（日期 -> 任务ID列表）
 * - Actions: 负责调用视图相关的API、修改State
 * - Getters: 从索引数据和 TaskStore 中组合出完整的任务列表
 *
 * 注意：View Store 不存储任务的原始数据，只存储任务ID和顺序
 */

// --- Payload Types for API calls ---
export interface ScheduleTaskPayload {
  task_id: string
  scheduled_day: string // YYYY-MM-DD ISO 8601 UTC
}

export interface RescheduleTaskPayload {
  task_id: string
  from_day: string
  to_day: string
}

export interface ReorderTasksPayload {
  date: string
  task_ids: string[]
}

export const useViewStore = defineStore('view', () => {
  // ============================================================
  // STATE - 只存储视图索引数据
  // ============================================================

  /**
   * 每日看板的任务ID列表（已排序）
   * key: YYYY-MM-DD (日期字符串)
   * value: 该日期的任务 ID 列表（按 sort_order 排序）
   */
  const dailyTaskIds = ref(new Map<string, string[]>())

  /**
   * 加载状态
   */
  const isLoading = ref(false)

  /**
   * 错误信息
   */
  const error = ref<string | null>(null)

  // ============================================================
  // GETTERS - 从索引数据和 TaskStore 组合出完整数据
  // ============================================================

  /**
   * 获取指定日期的任务卡片列表（已排序）
   * 这个 getter 会从 TaskStore 获取实际的任务数据
   */
  const getDailyTasks = computed(() => {
    return (date: string): TaskCard[] => {
      const taskStore = useTaskStore()
      const taskIds = dailyTaskIds.value.get(date) || []

      return taskIds
        .map((id) => taskStore.getTaskById(id))
        .filter((task): task is TaskCard => task !== undefined)
    }
  })

  /**
   * 获取指定日期的任务 ID 列表
   */
  const getDailyTaskIds = computed(() => {
    return (date: string): string[] => {
      return dailyTaskIds.value.get(date) || []
    }
  })

  /**
   * 检查指定日期是否有任务
   */
  const hasTasks = computed(() => {
    return (date: string): boolean => {
      const ids = dailyTaskIds.value.get(date)
      return ids !== undefined && ids.length > 0
    }
  })

  /**
   * 获取已加载的所有日期
   */
  const loadedDates = computed(() => {
    return Array.from(dailyTaskIds.value.keys()).sort()
  })

  /**
   * 获取指定日期的任务统计
   */
  const getDailyStats = computed(() => {
    return (date: string) => {
      const tasks = getDailyTasks.value(date)
      return {
        total: tasks.length,
        completed: tasks.filter((t) => t.is_completed).length,
        remaining: tasks.filter((t) => !t.is_completed).length,
      }
    }
  })

  // ============================================================
  // ACTIONS - 负责执行操作、调用API、修改State
  // ============================================================

  /**
   * 设置指定日期的任务 ID 列表（内部使用）
   */
  function setDailyTaskIds(date: string, taskIds: string[]) {
    const newMap = new Map(dailyTaskIds.value)
    newMap.set(date, taskIds)
    dailyTaskIds.value = newMap
  }

  /**
   * 在指定日期添加任务 ID
   */
  function addTaskIdToDate(date: string, taskId: string, position?: number) {
    const currentIds = dailyTaskIds.value.get(date) || []
    const newIds = [...currentIds]

    if (position !== undefined && position >= 0 && position <= newIds.length) {
      newIds.splice(position, 0, taskId)
    } else {
      newIds.push(taskId)
    }

    setDailyTaskIds(date, newIds)
  }

  /**
   * 从指定日期移除任务 ID
   */
  function removeTaskIdFromDate(date: string, taskId: string) {
    const currentIds = dailyTaskIds.value.get(date) || []
    const newIds = currentIds.filter((id) => id !== taskId)
    setDailyTaskIds(date, newIds)
  }

  /**
   * 清空指定日期的任务
   */
  function clearDate(date: string) {
    const newMap = new Map(dailyTaskIds.value)
    newMap.delete(date)
    dailyTaskIds.value = newMap
  }

  /**
   * 清空所有日期的任务
   */
  function clearAll() {
    dailyTaskIds.value = new Map()
  }

  /**
   * 获取指定日期的每日看板任务
   * API: GET /views/daily-schedule?day=YYYY-MM-DD
   *
   * 这个方法会：
   * 1. 调用后端 API 获取任务列表
   * 2. 更新 TaskStore 中的任务数据
   * 3. 更新本地的任务ID索引
   */
  async function fetchDailyTasks(date: string): Promise<TaskCard[]> {
    isLoading.value = true
    error.value = null

    try {
      // TODO: 实现 API 调用
      // const taskStore = useTaskStore()
      // const apiBaseUrl = await waitForApiReady()
      // const response = await fetch(`${apiBaseUrl}/views/daily-schedule?day=${date}`)
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)
      // const tasks: TaskCard[] = await response.json()

      // // 1. 更新 TaskStore 中的任务数据
      // taskStore.addOrUpdateTasks(tasks)

      // // 2. 更新本地的任务ID索引（保持后端返回的顺序）
      // const taskIds = tasks.map((t) => t.id)
      // setDailyTaskIds(date, taskIds)

      // return tasks

      console.log('[ViewStore] fetchDailyTasks - API not implemented yet', { date })
      return []
    } catch (e) {
      error.value = `Failed to fetch daily tasks for ${date}: ${e}`
      console.error('[ViewStore] Error fetching daily tasks:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取日期范围内的任务
   * API: GET /views/daily-schedule?start_date=...&end_date=...
   */
  async function fetchTasksForRange(startDate: string, endDate: string): Promise<void> {
    isLoading.value = true
    error.value = null

    try {
      // TODO: 可以实现为批量 API 调用，或者逐日查询
      // 临时方案：逐日查询
      const start = new Date(startDate)
      const end = new Date(endDate)
      const current = new Date(start)

      while (current <= end) {
        const dateStr = current.toISOString().split('T')[0] as string
        await fetchDailyTasks(dateStr)
        current.setDate(current.getDate() + 1)
      }

      console.log('[ViewStore] fetchTasksForRange - completed')
    } catch (e) {
      error.value = `Failed to fetch tasks for range: ${e}`
      console.error('[ViewStore] Error fetching tasks for range:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 将任务安排到指定日期
   * API: POST /schedules
   */
  async function scheduleTask(payload: ScheduleTaskPayload): Promise<boolean> {
    isLoading.value = true
    error.value = null
    console.log('[ViewStore] Scheduling task:', payload)

    try {
      // TODO: 实现 API 调用
      // const apiBaseUrl = await waitForApiReady()
      // const response = await fetch(`${apiBaseUrl}/schedules`, {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify(payload)
      // })
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)

      // // 重新获取该日期的任务列表
      // await fetchDailyTasks(payload.scheduled_day)
      // return true

      console.log('[ViewStore] scheduleTask - API not implemented yet')

      // 乐观更新：临时添加任务ID到日期
      addTaskIdToDate(payload.scheduled_day, payload.task_id)
      return true
    } catch (e) {
      error.value = `Failed to schedule task: ${e}`
      console.error('[ViewStore] Error scheduling task:', e)
      return false
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 重新安排任务（从一个日期移动到另一个日期）
   * API: POST /schedules/reschedule
   */
  async function rescheduleTask(payload: RescheduleTaskPayload): Promise<boolean> {
    isLoading.value = true
    error.value = null
    console.log('[ViewStore] Rescheduling task:', payload)

    try {
      // TODO: 实现 API 调用
      // const apiBaseUrl = await waitForApiReady()
      // const response = await fetch(`${apiBaseUrl}/schedules/reschedule`, {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify(payload)
      // })
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)

      // // 重新获取两个日期的任务列表
      // await fetchDailyTasks(payload.from_day)
      // await fetchDailyTasks(payload.to_day)
      // return true

      console.log('[ViewStore] rescheduleTask - API not implemented yet')

      // 乐观更新
      removeTaskIdFromDate(payload.from_day, payload.task_id)
      addTaskIdToDate(payload.to_day, payload.task_id)
      return true
    } catch (e) {
      error.value = `Failed to reschedule task: ${e}`
      console.error('[ViewStore] Error rescheduling task:', e)
      return false
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 取消任务的日程安排（将任务移回 staging 区）
   * API: DELETE /schedules/:task_id
   */
  async function unscheduleTask(taskId: string, date?: string): Promise<boolean> {
    isLoading.value = true
    error.value = null
    console.log('[ViewStore] Unscheduling task:', taskId)

    try {
      // TODO: 实现 API 调用
      // const apiBaseUrl = await waitForApiReady()
      // const response = await fetch(`${apiBaseUrl}/schedules/${taskId}`, {
      //   method: 'DELETE'
      // })
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)

      // // 从所有日期中移除该任务（或只从指定日期移除）
      // if (date) {
      //   removeTaskIdFromDate(date, taskId)
      // } else {
      //   for (const d of dailyTaskIds.value.keys()) {
      //     removeTaskIdFromDate(d, taskId)
      //   }
      // }
      // return true

      console.log('[ViewStore] unscheduleTask - API not implemented yet')

      // 乐观更新
      if (date) {
        removeTaskIdFromDate(date, taskId)
      } else {
        // 从所有日期中移除
        for (const d of dailyTaskIds.value.keys()) {
          removeTaskIdFromDate(d, taskId)
        }
      }
      return true
    } catch (e) {
      error.value = `Failed to unschedule task: ${e}`
      console.error('[ViewStore] Error unscheduling task:', e)
      return false
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 更新指定日期任务的排序
   * API: PUT /schedules/reorder
   */
  async function reorderTasks(payload: ReorderTasksPayload): Promise<boolean> {
    isLoading.value = true
    error.value = null
    console.log('[ViewStore] Reordering tasks:', payload)

    try {
      // TODO: 实现 API 调用
      // const apiBaseUrl = await waitForApiReady()
      // const response = await fetch(`${apiBaseUrl}/schedules/reorder`, {
      //   method: 'PUT',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify(payload)
      // })
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)

      // // 更新本地 state
      // setDailyTaskIds(payload.date, payload.task_ids)
      // return true

      console.log('[ViewStore] reorderTasks - API not implemented yet')

      // 乐观更新
      setDailyTaskIds(payload.date, payload.task_ids)
      return true
    } catch (e) {
      error.value = `Failed to reorder tasks: ${e}`
      console.error('[ViewStore] Error reordering tasks:', e)
      return false
    } finally {
      isLoading.value = false
    }
  }

  return {
    // State
    dailyTaskIds,
    isLoading,
    error,

    // Getters
    getDailyTasks,
    getDailyTaskIds,
    hasTasks,
    loadedDates,
    getDailyStats,

    // Actions
    setDailyTaskIds,
    addTaskIdToDate,
    removeTaskIdFromDate,
    clearDate,
    clearAll,
    fetchDailyTasks,
    fetchTasksForRange,
    scheduleTask,
    rescheduleTask,
    unscheduleTask,
    reorderTasks,
  }
})

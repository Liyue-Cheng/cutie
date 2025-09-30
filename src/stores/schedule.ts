import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

// --- Type Definitions ---
type ID = string

export interface TaskSchedule {
  id: string
  task_id: string
  scheduled_day: string
  outcome: 'PLANNED' | 'PRESENCE_LOGGED' | 'COMPLETED_ON_DAY' | 'CARRIED_OVER'
  created_at: string
  updated_at: string
}

export interface ScheduleTaskPayload {
  task_id: string
  scheduled_day: string
}

// --- API Base URL ---
import { waitForApiReady } from '@/composables/useApiConfig'

export const useScheduleStore = defineStore('schedule', () => {
  // --- State ---
  const schedules = ref(new Map<ID, TaskSchedule>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // --- Getters ---

  /**
   * Returns all schedules as a sorted array.
   */
  const allSchedules = computed(() => {
    return Array.from(schedules.value.values()).sort(
      (a, b) => new Date(a.scheduled_day).getTime() - new Date(b.scheduled_day).getTime()
    )
  })

  /**
   * Returns schedules for a specific date.
   */
  const getSchedulesForDate = computed(() => {
    return (date: string) => {
      return Array.from(schedules.value.values()).filter(
        (schedule) => schedule.scheduled_day === date
      )
    }
  })

  /**
   * Returns schedules for a specific task.
   */
  const getSchedulesForTask = computed(() => {
    return (taskId: string) => {
      return Array.from(schedules.value.values()).filter((schedule) => schedule.task_id === taskId)
    }
  })

  /**
   * Returns a function to get a schedule by its ID.
   */
  function getScheduleById(id: ID): TaskSchedule | undefined {
    return schedules.value.get(id)
  }

  // --- Actions ---

  /**
   * Fetches schedules for a specific date.
   * 注意：后端没有直接查询schedules的API，这个函数暂时返回空数组
   * 请使用 views/daily-schedule 端点获取某天的任务
   */
  async function fetchSchedulesForDate(date: string) {
    console.warn('[ScheduleStore] fetchSchedulesForDate is deprecated, use views API instead')
    return []
  }

  /**
   * Fetches schedules for a date range.
   * 通过多次单日查询实现范围查询
   */
  async function fetchSchedulesForRange(startDate: string, endDate: string) {
    isLoading.value = true
    error.value = null
    try {
      const start = new Date(startDate)
      const end = new Date(endDate)

      const allSchedules: TaskSchedule[] = []
      const currentDate = new Date(start)

      // 遍历日期范围，逐日查询
      while (currentDate <= end) {
        const dateStr = currentDate.toISOString().split('T')[0] // YYYY-MM-DD
        const schedulesForDay = await fetchSchedulesForDate(dateStr)
        allSchedules.push(...schedulesForDay)

        // 移动到下一天
        currentDate.setDate(currentDate.getDate() + 1)
      }

      console.log(
        `[ScheduleStore] Fetched ${allSchedules.length} schedules for range ${startDate} - ${endDate}`
      )
      return allSchedules
    } catch (e) {
      error.value = `Failed to fetch schedules for range: ${e}`
      console.error('[ScheduleStore] Error fetching schedules:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Schedules a task to a specific day.
   */
  async function scheduleTask(payload: ScheduleTaskPayload) {
    isLoading.value = true
    error.value = null
    console.log(`[ScheduleStore] Attempting to schedule task with payload:`, payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/schedules`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const newSchedule: TaskSchedule = apiResponse.data

      schedules.value.set(newSchedule.id, newSchedule)
      console.log(`[ScheduleStore] Successfully scheduled task:`, newSchedule)
      return newSchedule
    } catch (e) {
      error.value = `Failed to schedule task: ${e}`
      console.error(`[ScheduleStore] Error scheduling task:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Deletes a specific schedule.
   */
  async function deleteSchedule(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/schedules/${id}`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      schedules.value.delete(id)
      console.log(`[ScheduleStore] Successfully deleted schedule ${id}`)
    } catch (e) {
      error.value = `Failed to delete schedule ${id}: ${e}`
      console.error('[ScheduleStore] Error deleting schedule:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Logs presence for a schedule (marks effort as logged).
   */
  async function logPresence(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/schedules/${id}/presence`, {
        method: 'POST',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const updatedSchedule: TaskSchedule = apiResponse.data

      schedules.value.set(id, updatedSchedule)
      console.log(`[ScheduleStore] Successfully logged presence for schedule ${id}`)
      return updatedSchedule
    } catch (e) {
      error.value = `Failed to log presence for schedule ${id}: ${e}`
      console.error('[ScheduleStore] Error logging presence:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Unschedules a task completely (removes all schedules).
   */
  async function unscheduleTaskCompletely(taskId: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${taskId}/schedules`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      // Remove all schedules for this task from the store
      const schedulesToRemove = Array.from(schedules.value.values()).filter(
        (schedule) => schedule.task_id === taskId
      )
      for (const schedule of schedulesToRemove) {
        schedules.value.delete(schedule.id)
      }

      console.log(`[ScheduleStore] Successfully unscheduled task ${taskId} completely`)
    } catch (e) {
      error.value = `Failed to unschedule task ${taskId}: ${e}`
      console.error('[ScheduleStore] Error unscheduling task:', e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    // State
    schedules,
    isLoading,
    error,
    // Getters
    allSchedules,
    getSchedulesForDate,
    getSchedulesForTask,
    getScheduleById,
    // Actions
    fetchSchedulesForDate,
    fetchSchedulesForRange,
    scheduleTask,
    deleteSchedule,
    logPresence,
    unscheduleTaskCompletely,
  }
})

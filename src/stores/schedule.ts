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
  target_day: string
  mode: 'link' | 'move'
  source_schedule_id?: string | null
}

// --- API Base URL ---
const API_BASE_URL = 'http://localhost:3030/api'

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
   */
  async function fetchSchedulesForDate(date: string) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/schedules?date=${encodeURIComponent(date)}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const scheduleList: TaskSchedule[] = apiResponse.data

      // Update schedules in the store
      for (const schedule of scheduleList) {
        schedules.value.set(schedule.id, schedule)
      }

      console.log(`[ScheduleStore] Fetched ${scheduleList.length} schedules for ${date}`)
      return scheduleList
    } catch (e) {
      error.value = `Failed to fetch schedules for ${date}: ${e}`
      console.error('[ScheduleStore] Error fetching schedules:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches schedules for a date range.
   */
  async function fetchSchedulesForRange(startDate: string, endDate: string) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${API_BASE_URL}/schedules?start_date=${encodeURIComponent(startDate)}&end_date=${encodeURIComponent(endDate)}`
      )
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const scheduleList: TaskSchedule[] = apiResponse.data

      // Update schedules in the store
      for (const schedule of scheduleList) {
        schedules.value.set(schedule.id, schedule)
      }

      console.log(
        `[ScheduleStore] Fetched ${scheduleList.length} schedules for range ${startDate} - ${endDate}`
      )
      return scheduleList
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
      const response = await fetch(`${API_BASE_URL}/schedules`, {
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
      const response = await fetch(`${API_BASE_URL}/schedules/${id}`, {
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
      const response = await fetch(`${API_BASE_URL}/schedules/${id}/presence`, {
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
      const response = await fetch(`${API_BASE_URL}/schedules/tasks/${taskId}`, {
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

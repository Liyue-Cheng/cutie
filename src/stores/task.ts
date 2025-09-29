import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { Task, Subtask } from '@/types/models'
import { waitForApiReady } from '@/composables/useApiConfig'

// --- Type Aliases from models.ts ---
type ID = string

// --- Payload Types for API calls ---
export interface CreateTaskPayload {
  title: string
  glance_note?: string | null
  detail_note?: string | null
  estimated_duration?: number | null
  subtasks?: Subtask[] | null
  area_id?: string | null
  due_date?: string | null
  due_date_type?: 'SOFT' | 'HARD' | null
  context: {
    context_type: 'MISC' | 'DAILY_KANBAN' | 'PROJECT_LIST' | 'AREA_FILTER'
    context_id: string
  }
}

export interface UpdateTaskPayload {
  title?: string
  glance_note?: string | null
  detail_note?: string | null
  estimated_duration?: number | null
  subtasks?: Subtask[] | null
  project_id?: string | null
  area_id?: string | null
  due_date?: string | null
  due_date_type?: 'SOFT' | 'HARD' | null
}

export interface SearchTasksParams {
  q?: string
  limit?: number
}

export const useTaskStore = defineStore('task', () => {
  // --- State ---
  const tasks = ref(new Map<ID, Task>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // --- Getters ---

  /**
   * Returns all tasks as a sorted array.
   */
  const allTasks = computed(() => {
    return Array.from(tasks.value.values()).sort(
      (a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
    )
  })

  /**
   * Returns all unscheduled tasks (staging area).
   */
  const unscheduledTasks = computed(() => {
    const allTasks = Array.from(tasks.value.values())
    const filtered = allTasks.filter((task) => !task.is_deleted && !task.completed_at)
    console.log(
      `[TaskStore] unscheduledTasks computed - Total tasks: ${allTasks.length}, Unscheduled: ${filtered.length}`
    )
    console.log(
      `[TaskStore] All tasks:`,
      allTasks.map((t) => ({
        id: t.id,
        title: t.title,
        completed_at: t.completed_at,
        is_deleted: t.is_deleted,
      }))
    )
    return filtered
  })

  /**
   * Returns a function to get a task by its ID.
   * @param id The ID of the task to retrieve.
   */
  function getTaskById(id: ID): Task | undefined {
    return tasks.value.get(id)
  }

  // --- Actions ---

  /**
   * Fetches all unscheduled tasks from the backend and updates the state.
   */
  async function fetchUnscheduledTasks() {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/unscheduled`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const taskList: Task[] = apiResponse.data

      const taskMap = new Map<ID, Task>()
      for (const task of taskList) {
        taskMap.set(task.id, task)
      }
      tasks.value = taskMap

      console.log(`[TaskStore] Fetched ${taskList.length} unscheduled tasks`)
    } catch (e) {
      error.value = `Failed to fetch unscheduled tasks: ${e}`
      console.error('[TaskStore] Error fetching unscheduled tasks:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Searches tasks based on keywords or returns unscheduled tasks if no query.
   */
  async function searchTasks(params: SearchTasksParams = {}) {
    isLoading.value = true
    error.value = null
    try {
      const searchParams = new URLSearchParams()
      if (params.q) searchParams.append('q', params.q)
      if (params.limit) searchParams.append('limit', params.limit.toString())

      const apiBaseUrl = await waitForApiReady()
      const url = `${apiBaseUrl}/tasks${searchParams.toString() ? '?' + searchParams.toString() : ''}`
      const response = await fetch(url)

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const taskList: Task[] = apiResponse.data

      // Update tasks in the store
      for (const task of taskList) {
        tasks.value.set(task.id, task)
      }

      console.log(`[TaskStore] Found ${taskList.length} tasks matching search`)
      return taskList
    } catch (e) {
      error.value = `Failed to search tasks: ${e}`
      console.error('[TaskStore] Error searching tasks:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches a single task by ID.
   */
  async function fetchTask(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const task: Task = apiResponse.data

      // 确保响应性更新
      const newTasks = new Map(tasks.value)
      newTasks.set(task.id, task)
      tasks.value = newTasks
      console.log(`[TaskStore] Fetched task ${id}`)
      return task
    } catch (e) {
      error.value = `Failed to fetch task ${id}: ${e}`
      console.error(`[TaskStore] Error fetching task ${id}:`, e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Creates a new task.
   * @param payload The data for the new task.
   */
  async function createTask(payload: CreateTaskPayload) {
    isLoading.value = true
    error.value = null
    console.log(`[TaskStore] Attempting to create task with payload:`, payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks`, {
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
      const newTask: Task = apiResponse.data

      // 确保响应性更新
      const newTasks = new Map(tasks.value)
      newTasks.set(newTask.id, newTask)
      tasks.value = newTasks
      console.log(`[TaskStore] Successfully created task:`, newTask)
    } catch (e) {
      error.value = `Failed to create task: ${e}`
      console.error(`[TaskStore] Error creating task:`, e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Updates an existing task.
   * @param id The ID of the task to update.
   * @param payload The new data for the task.
   */
  async function updateTask(id: ID, payload: UpdateTaskPayload) {
    isLoading.value = true
    error.value = null
    console.log(`[TaskStore] Attempting to update task ${id} with payload:`, payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}`, {
        method: 'PUT',
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
      const updatedTask: Task = apiResponse.data

      // 确保响应性更新
      const newTasks = new Map(tasks.value)
      newTasks.set(id, updatedTask)
      tasks.value = newTasks
      console.log(`[TaskStore] Successfully updated task ${id}:`, updatedTask)
    } catch (e) {
      error.value = `Failed to update task ${id}: ${e}`
      console.error(`[TaskStore] Error updating task ${id}:`, e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Deletes a task.
   * @param id The ID of the task to delete.
   */
  async function deleteTask(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      // 确保响应性更新
      const newTasks = new Map(tasks.value)
      newTasks.delete(id)
      tasks.value = newTasks
      console.log(`[TaskStore] Successfully deleted task ${id}`)
    } catch (e) {
      error.value = `Failed to delete task ${id}: ${e}`
      console.error('[TaskStore] Error deleting task:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Completes a task by calling the completion endpoint.
   * @param id The ID of the task to complete.
   */
  async function completeTask(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}/completion`, {
        method: 'POST',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const completedTask: Task = apiResponse.data

      console.log(`[TaskStore] API Response for complete task:`, apiResponse)
      console.log(`[TaskStore] Completed task data:`, completedTask)
      console.log(`[TaskStore] Task completed_at:`, completedTask.completed_at)

      // 确保响应性更新
      const newTasks = new Map(tasks.value)
      newTasks.set(id, completedTask)
      tasks.value = newTasks
      console.log(`[TaskStore] Successfully completed task ${id}`)
      console.log(
        `[TaskStore] Updated tasks map:`,
        Array.from(newTasks.values()).find((t) => t.id === id)
      )
    } catch (e) {
      error.value = `Failed to complete task ${id}: ${e}`
      console.error('[TaskStore] Error completing task:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Reopens a completed task.
   * @param id The ID of the task to reopen.
   */
  async function reopenTask(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}/reopen`, {
        method: 'POST',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const reopenedTask: Task = apiResponse.data

      // 确保响应性更新
      const newTasks = new Map(tasks.value)
      newTasks.set(id, reopenedTask)
      tasks.value = newTasks
      console.log(`[TaskStore] Successfully reopened task ${id}`)
    } catch (e) {
      error.value = `Failed to reopen task ${id}: ${e}`
      console.error('[TaskStore] Error reopening task:', e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    // State
    tasks,
    isLoading,
    error,
    // Getters
    allTasks,
    unscheduledTasks,
    getTaskById,
    // Actions
    fetchUnscheduledTasks,
    searchTasks,
    fetchTask,
    createTask,
    updateTask,
    deleteTask,
    completeTask,
    reopenTask,
  }
})

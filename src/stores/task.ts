import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { Task, TaskStatus } from '@/types/models'

// --- Type Aliases from models.ts ---
type ID = string

// --- API Base URL ---
// 使用固定端口3030，避免与8080端口冲突
const API_BASE_URL = 'http://localhost:3030/api'

// --- Payload Types for API calls ---
export interface CreateTaskPayload {
  title: string
  glance_note?: string | null
  detail_note?: string | null
  estimated_duration?: number | null
  subtasks?: Array<{
    id: string
    title: string
    is_completed: boolean
    sort_order: string
  }> | null
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
  subtasks?: Array<{
    id: string
    title: string
    is_completed: boolean
    sort_order: string
  }> | null
  project_id?: string | null
  area_id?: string | null
  due_date?: string | null
  due_date_type?: 'SOFT' | 'HARD' | null
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
    return Array.from(tasks.value.values()).filter((task) => !task.is_deleted && !task.completed_at)
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
  async function fetchTasks() {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/tasks/unscheduled`)
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
      error.value = `Failed to fetch tasks: ${e}`
      console.error('[TaskStore] Error fetching tasks:', e)
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
      const response = await fetch(`${API_BASE_URL}/tasks`, {
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

      tasks.value.set(newTask.id, newTask)
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
      const response = await fetch(`${API_BASE_URL}/tasks/${id}`, {
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

      tasks.value.set(id, updatedTask)
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
      const response = await fetch(`${API_BASE_URL}/tasks/${id}`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      tasks.value.delete(id)
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
      const response = await fetch(`${API_BASE_URL}/tasks/${id}/completion`, {
        method: 'POST',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const completedTask: Task = apiResponse.data

      tasks.value.set(id, completedTask)
      console.log(`[TaskStore] Successfully completed task ${id}`)
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
      const response = await fetch(`${API_BASE_URL}/tasks/${id}/reopen`, {
        method: 'POST',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const reopenedTask: Task = apiResponse.data

      tasks.value.set(id, reopenedTask)
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
    fetchTasks,
    createTask,
    updateTask,
    deleteTask,
    completeTask,
    reopenTask,
  }
})

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Task, TaskStatus } from '@/types/models'

// --- Type Aliases from models.ts ---
type ID = string

// --- Payload Types for API calls ---
export interface CreateTaskPayload {
  project_id?: ID | null
  title: string
  status?: TaskStatus
  due_date?: number | null
  sort_key: string
  metadata?: Record<string, any> | null
}

export interface UpdateTaskPayload {
  project_id?: ID | null
  title?: string
  status?: TaskStatus
  due_date?: number | null
  completed_at?: number | null
  sort_key?: string
  metadata?: Record<string, any> | null
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
    return Array.from(tasks.value.values()).sort((a, b) => a.sort_key.localeCompare(b.sort_key))
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
   * Fetches all tasks from the backend and updates the state.
   */
  async function fetchTasks() {
    isLoading.value = true
    error.value = null
    try {
      const taskList = await invoke<Task[]>('list_tasks')
      const taskMap = new Map<ID, Task>()
      for (const task of taskList) {
        taskMap.set(task.id, task)
      }
      tasks.value = taskMap
    } catch (e) {
      error.value = `Failed to fetch tasks: ${e}`
      console.error(e)
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
    try {
      const newTask = await invoke<Task>('create_task', { payload })
      tasks.value.set(newTask.id, newTask)
    } catch (e) {
      error.value = `Failed to create task: ${e}`
      console.error(e)
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
    try {
      const updatedTask = await invoke<Task>('update_task', { id, payload })
      tasks.value.set(id, updatedTask)
    } catch (e) {
      error.value = `Failed to update task ${id}: ${e}`
      console.error(e)
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
      await invoke('delete_task', { id })
      tasks.value.delete(id)
    } catch (e) {
      error.value = `Failed to delete task ${id}: ${e}`
      console.error(e)
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
    getTaskById,
    // Actions
    fetchTasks,
    createTask,
    updateTask,
    deleteTask,
  }
})

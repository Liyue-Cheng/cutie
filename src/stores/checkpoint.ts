import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Checkpoint } from '@/types/models'

type ID = string

export interface CreateCheckpointPayload {
  task_id: ID
  title: string
  sort_key: string
}

export interface UpdateCheckpointPayload {
  title?: string
  is_completed?: boolean
  sort_key?: string
}

export const useCheckpointStore = defineStore('checkpoint', () => {
  // Store checkpoints in a map where the key is the task ID
  const checkpointsByTask = ref(new Map<ID, Checkpoint[]>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const getCheckpointsForTask = computed(() => {
    return (taskId: ID) => {
      const taskCheckpoints = checkpointsByTask.value.get(taskId) || []
      // Return a sorted copy
      return [...taskCheckpoints].sort((a, b) => a.sort_key.localeCompare(b.sort_key))
    }
  })

  async function fetchCheckpointsForTask(taskId: ID) {
    isLoading.value = true
    error.value = null
    try {
      const checkpointList = await invoke<Checkpoint[]>('list_checkpoints_for_task', {
        taskId: taskId,
      })
      checkpointsByTask.value.set(taskId, checkpointList)
    } catch (e) {
      error.value = `Failed to fetch checkpoints for task ${taskId}: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function createCheckpoint(payload: CreateCheckpointPayload) {
    isLoading.value = true
    error.value = null
    try {
      const newCheckpoint = await invoke<Checkpoint>('create_checkpoint', { payload })
      const taskCheckpoints = checkpointsByTask.value.get(payload.task_id) || []
      taskCheckpoints.push(newCheckpoint)
      checkpointsByTask.value.set(payload.task_id, taskCheckpoints)
    } catch (e) {
      error.value = `Failed to create checkpoint: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function updateCheckpoint(id: ID, payload: UpdateCheckpointPayload, taskId: ID) {
    isLoading.value = true
    error.value = null
    try {
      const updatedCheckpoint = await invoke<Checkpoint>('update_checkpoint', { id, payload })
      const taskCheckpoints = checkpointsByTask.value.get(taskId)
      if (taskCheckpoints) {
        const index = taskCheckpoints.findIndex((c) => c.id === id)
        if (index !== -1) {
          taskCheckpoints[index] = updatedCheckpoint
          checkpointsByTask.value.set(taskId, [...taskCheckpoints])
        }
      }
    } catch (e) {
      error.value = `Failed to update checkpoint ${id}: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function deleteCheckpoint(id: ID, taskId: ID) {
    isLoading.value = true
    error.value = null
    try {
      await invoke('delete_checkpoint', { id })
      const taskCheckpoints = checkpointsByTask.value.get(taskId)
      if (taskCheckpoints) {
        const filteredCheckpoints = taskCheckpoints.filter((c) => c.id !== id)
        checkpointsByTask.value.set(taskId, filteredCheckpoints)
      }
    } catch (e) {
      error.value = `Failed to delete checkpoint ${id}: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    checkpointsByTask,
    isLoading,
    error,
    getCheckpointsForTask,
    fetchCheckpointsForTask,
    createCheckpoint,
    updateCheckpoint,
    deleteCheckpoint,
  }
})

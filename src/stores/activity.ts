import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Activity } from '@/types/models'

type ID = string

export interface CreateActivityPayload {
  title?: string | null
  start_time: string
  end_time: string
  timezone?: string | null
  is_all_day?: boolean
  color?: string | null
  metadata?: Record<string, any> | null
}

export interface UpdateActivityPayload {
  title?: string | null
  start_time?: string
  end_time?: string
  timezone?: string | null
  is_all_day?: boolean
  color?: string | null
  metadata?: Record<string, any> | null
}

export const useActivityStore = defineStore('activity', () => {
  const activities = ref(new Map<ID, Activity>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const allActivities = computed(() => {
    // Activities are typically sorted by start time.
    return Array.from(activities.value.values()).sort((a, b) => {
      const dateA = new Date(a.start_time).getTime()
      const dateB = new Date(b.start_time).getTime()
      return dateA - dateB
    })
  })

  function getActivityById(id: ID): Activity | undefined {
    return activities.value.get(id)
  }

  async function fetchActivities() {
    isLoading.value = true
    error.value = null
    try {
      const activityList = await invoke<Activity[]>('list_activities')
      const activityMap = new Map<ID, Activity>()
      for (const activity of activityList) {
        activityMap.set(activity.id, activity)
      }
      activities.value = activityMap
    } catch (e) {
      error.value = `Failed to fetch activities: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function createActivity(payload: CreateActivityPayload) {
    isLoading.value = true
    error.value = null
    try {
      const newActivity = await invoke<Activity>('create_activity', { payload })
      activities.value.set(newActivity.id, newActivity)
    } catch (e) {
      error.value = `Failed to create activity: ${e}`
      console.error(e)
      // Re-throw the error so the component can handle it
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function updateActivity(id: ID, payload: UpdateActivityPayload) {
    isLoading.value = true
    error.value = null
    try {
      const updatedActivity = await invoke<Activity>('update_activity', { id, payload })
      activities.value.set(id, updatedActivity)
    } catch (e) {
      error.value = `Failed to update activity ${id}: ${e}`
      console.error(e)
      // Re-throw the error so the component can handle it
      throw e
    } finally {
      isLoading.value = false
    }
  }

  async function deleteActivity(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      await invoke('delete_activity', { id })
      activities.value.delete(id)
    } catch (e) {
      error.value = `Failed to delete activity ${id}: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    activities,
    isLoading,
    error,
    allActivities,
    getActivityById,
    fetchActivities,
    createActivity,
    updateActivity,
    deleteActivity,
  }
})

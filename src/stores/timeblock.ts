import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

// --- Type Definitions ---
type ID = string

export interface TimeBlock {
  id: string
  title: string | null
  glance_note: string | null
  detail_note: string | null
  start_time: string
  end_time: string
  area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
}

export interface CreateTimeBlockPayload {
  title?: string | null
  glance_note?: string | null
  detail_note?: string | null
  start_time: string
  end_time: string
  area_id?: string | null
  task_ids: string[]
}

export interface UpdateTimeBlockPayload {
  title?: string | null
  glance_note?: string | null
  detail_note?: string | null
  start_time?: string
  end_time?: string
  area_id?: string | null
}

export interface FreeTimeSlot {
  start_time: string
  end_time: string
  duration_minutes: number
}

export interface ConflictCheckResult {
  has_conflict: boolean
  start_time: string
  end_time: string
}

// --- API Base URL ---
import { waitForApiReady } from '@/composables/useApiConfig'

export const useTimeBlockStore = defineStore('timeblock', () => {
  // --- State ---
  const timeBlocks = ref(new Map<ID, TimeBlock>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // --- Getters ---

  /**
   * Returns all time blocks as a sorted array.
   */
  const allTimeBlocks = computed(() => {
    return Array.from(timeBlocks.value.values())
      .filter((block) => !block.is_deleted)
      .sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime())
  })

  /**
   * Returns time blocks for a specific date.
   */
  const getTimeBlocksForDate = computed(() => {
    return (date: string) => {
      const targetDate = new Date(date).toDateString()
      return Array.from(timeBlocks.value.values())
        .filter(
          (block) => !block.is_deleted && new Date(block.start_time).toDateString() === targetDate
        )
        .sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime())
    }
  })

  /**
   * Returns time blocks for a specific area.
   */
  const getTimeBlocksForArea = computed(() => {
    return (areaId: string) => {
      return Array.from(timeBlocks.value.values())
        .filter((block) => !block.is_deleted && block.area_id === areaId)
        .sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime())
    }
  })

  /**
   * Returns a function to get a time block by its ID.
   */
  function getTimeBlockById(id: ID): TimeBlock | undefined {
    return timeBlocks.value.get(id)
  }

  // --- Actions ---

  /**
   * Fetches time blocks for a specific date.
   */
  async function fetchTimeBlocksForDate(date: string) {
    // 不设置 loading 状态，因为可能被 fetchTimeBlocksForRange 批量调用
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks?date=${encodeURIComponent(date)}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      // 后端直接返回数组，不是 {data: []} 格式
      const blockList: TimeBlock[] = await response.json()

      // Update time blocks in the store
      for (const block of blockList) {
        timeBlocks.value.set(block.id, block)
      }

      console.log(`[TimeBlockStore] Fetched ${blockList.length} time blocks for ${date}`)
      return blockList
    } catch (e) {
      error.value = `Failed to fetch time blocks for ${date}: ${e}`
      console.error(`[TimeBlockStore] Error fetching time blocks for date ${date}:`, e)
      return []
    }
  }

  /**
   * Fetches time blocks for a date range.
   * 通过多次单日查询实现范围查询
   */
  async function fetchTimeBlocksForRange(startDate: string, endDate: string) {
    isLoading.value = true
    error.value = null
    try {
      const start = new Date(startDate)
      const end = new Date(endDate)
      
      const allBlocks: TimeBlock[] = []
      const currentDate = new Date(start)

      // 遍历日期范围，逐日查询
      while (currentDate <= end) {
        const dateStr = currentDate.toISOString().split('T')[0] // YYYY-MM-DD
        const blocks = await fetchTimeBlocksForDate(dateStr)
        allBlocks.push(...blocks)
        
        // 移动到下一天
        currentDate.setDate(currentDate.getDate() + 1)
      }

      console.log(
        `[TimeBlockStore] Fetched ${allBlocks.length} time blocks for range ${startDate} - ${endDate}`
      )
      return allBlocks
    } catch (e) {
      error.value = `Failed to fetch time blocks for range: ${e}`
      console.error('[TimeBlockStore] Error fetching time blocks:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches time blocks for a specific task.
   */
  async function fetchTimeBlocksForTask(taskId: string) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(
        `${apiBaseUrl}/time-blocks?task_id=${encodeURIComponent(taskId)}`
      )
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const blockList: TimeBlock[] = apiResponse.data

      // Update time blocks in the store
      for (const block of blockList) {
        timeBlocks.value.set(block.id, block)
      }

      console.log(`[TimeBlockStore] Fetched ${blockList.length} time blocks for task ${taskId}`)
      return blockList
    } catch (e) {
      error.value = `Failed to fetch time blocks for task ${taskId}: ${e}`
      console.error('[TimeBlockStore] Error fetching time blocks:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches time blocks for a specific area.
   */
  async function fetchTimeBlocksForArea(areaId: string) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(
        `${apiBaseUrl}/time-blocks?area_id=${encodeURIComponent(areaId)}`
      )
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const blockList: TimeBlock[] = apiResponse.data

      // Update time blocks in the store
      for (const block of blockList) {
        timeBlocks.value.set(block.id, block)
      }

      console.log(`[TimeBlockStore] Fetched ${blockList.length} time blocks for area ${areaId}`)
      return blockList
    } catch (e) {
      error.value = `Failed to fetch time blocks for area ${areaId}: ${e}`
      console.error('[TimeBlockStore] Error fetching time blocks:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches a single time block by ID.
   */
  async function fetchTimeBlock(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks/${id}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const timeBlock: TimeBlock = apiResponse.data

      timeBlocks.value.set(timeBlock.id, timeBlock)
      console.log(`[TimeBlockStore] Fetched time block ${id}`)
      return timeBlock
    } catch (e) {
      error.value = `Failed to fetch time block ${id}: ${e}`
      console.error(`[TimeBlockStore] Error fetching time block ${id}:`, e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Creates a new time block.
   */
  async function createTimeBlock(payload: CreateTimeBlockPayload) {
    isLoading.value = true
    error.value = null
    console.log(`[TimeBlockStore] Attempting to create time block with payload:`, payload)
    console.log(`[TimeBlockStore] Payload JSON:`, JSON.stringify(payload, null, 2))
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks`, {
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
      const newTimeBlock: TimeBlock = apiResponse.data

      timeBlocks.value.set(newTimeBlock.id, newTimeBlock)
      console.log(`[TimeBlockStore] Successfully created time block:`, newTimeBlock)
      return newTimeBlock
    } catch (e) {
      error.value = `Failed to create time block: ${e}`
      console.error(`[TimeBlockStore] Error creating time block:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Updates an existing time block.
   */
  async function updateTimeBlock(id: ID, payload: UpdateTimeBlockPayload) {
    isLoading.value = true
    error.value = null
    console.log(`[TimeBlockStore] Attempting to update time block ${id} with payload:`, payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks/${id}`, {
        method: 'PATCH',
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
      const updatedTimeBlock: TimeBlock = apiResponse.data

      timeBlocks.value.set(id, updatedTimeBlock)
      console.log(`[TimeBlockStore] Successfully updated time block ${id}:`, updatedTimeBlock)
      return updatedTimeBlock
    } catch (e) {
      error.value = `Failed to update time block ${id}: ${e}`
      console.error(`[TimeBlockStore] Error updating time block ${id}:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Deletes a time block.
   */
  async function deleteTimeBlock(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks/${id}`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      timeBlocks.value.delete(id)
      console.log(`[TimeBlockStore] Successfully deleted time block ${id}`)
    } catch (e) {
      error.value = `Failed to delete time block ${id}: ${e}`
      console.error('[TimeBlockStore] Error deleting time block:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Links a task to a time block.
   */
  async function linkTaskToBlock(blockId: ID, taskId: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks/${blockId}/links`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          task_id: taskId,
        }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      console.log(`[TimeBlockStore] Successfully linked task ${taskId} to block ${blockId}`)
    } catch (e) {
      error.value = `Failed to link task to block: ${e}`
      console.error('[TimeBlockStore] Error linking task to block:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Unlinks a task from a time block.
   */
  async function unlinkTaskFromBlock(blockId: ID, taskId: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks/${blockId}/links`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          task_id: taskId,
        }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      console.log(`[TimeBlockStore] Successfully unlinked task ${taskId} from block ${blockId}`)
    } catch (e) {
      error.value = `Failed to unlink task from block: ${e}`
      console.error('[TimeBlockStore] Error unlinking task from block:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Checks for time conflicts.
   */
  async function checkTimeConflict(
    startTime: string,
    endTime: string,
    excludeId?: string
  ): Promise<ConflictCheckResult | null> {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const params = new URLSearchParams()
      params.append('start_time', startTime)
      params.append('end_time', endTime)
      if (excludeId) {
        params.append('exclude_id', excludeId)
      }

      const response = await fetch(`${apiBaseUrl}/time-blocks/conflicts?${params.toString()}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const result: ConflictCheckResult = apiResponse.data

      console.log(`[TimeBlockStore] Conflict check result:`, result)
      return result
    } catch (e) {
      error.value = `Failed to check time conflict: ${e}`
      console.error('[TimeBlockStore] Error checking time conflict:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Finds free time slots in a given time range.
   */
  async function findFreeSlots(
    startTime: string,
    endTime: string,
    minDurationMinutes: number
  ): Promise<FreeTimeSlot[]> {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const params = new URLSearchParams()
      params.append('start_time', startTime)
      params.append('end_time', endTime)
      params.append('min_duration_minutes', minDurationMinutes.toString())

      const response = await fetch(`${apiBaseUrl}/time-blocks/free-slots?${params.toString()}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const slots: FreeTimeSlot[] = apiResponse.data

      console.log(`[TimeBlockStore] Found ${slots.length} free slots`)
      return slots
    } catch (e) {
      error.value = `Failed to find free slots: ${e}`
      console.error('[TimeBlockStore] Error finding free slots:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  return {
    // State
    timeBlocks,
    isLoading,
    error,
    // Getters
    allTimeBlocks,
    getTimeBlocksForDate,
    getTimeBlocksForArea,
    getTimeBlockById,
    // Actions
    fetchTimeBlocksForDate,
    fetchTimeBlocksForRange,
    fetchTimeBlocksForTask,
    fetchTimeBlocksForArea,
    fetchTimeBlock,
    createTimeBlock,
    updateTimeBlock,
    deleteTimeBlock,
    linkTaskToBlock,
    unlinkTaskFromBlock,
    checkTimeConflict,
    findFreeSlots,
  }
})

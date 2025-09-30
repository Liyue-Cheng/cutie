import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

// --- Type Definitions ---
type ID = string

export interface Ordering {
  id: string
  context_type: 'DAILY_KANBAN' | 'PROJECT_LIST' | 'AREA_FILTER' | 'MISC'
  context_id: string
  task_id: string
  sort_order: string
  updated_at: string
}

export interface UpdateOrderPayload {
  task_id: string
  context_type: 'DAILY_KANBAN' | 'PROJECT_LIST' | 'AREA_FILTER' | 'MISC'
  context_id: string
  sort_order: string
}

export interface CalculateSortOrderParams {
  context_type: 'DAILY_KANBAN' | 'PROJECT_LIST' | 'AREA_FILTER' | 'MISC'
  context_id: string
  prev_sort_order?: string
  next_sort_order?: string
}

// --- API Base URL ---
import { waitForApiReady } from '@/composables/useApiConfig'

export const useOrderingStore = defineStore('ordering', () => {
  // --- State ---
  const orderings = ref(new Map<string, Ordering>()) // key: `${context_type}:${context_id}:${task_id}`
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // --- Getters ---

  /**
   * Returns orderings for a specific context.
   */
  const getOrderingsForContext = computed(() => {
    return (contextType: string, contextId: string) => {
      return Array.from(orderings.value.values())
        .filter(
          (ordering) => ordering.context_type === contextType && ordering.context_id === contextId
        )
        .sort((a, b) => a.sort_order.localeCompare(b.sort_order))
    }
  })

  /**
   * Returns orderings for a specific task.
   */
  const getOrderingsForTask = computed(() => {
    return (taskId: string) => {
      return Array.from(orderings.value.values())
        .filter((ordering) => ordering.task_id === taskId)
        .sort((a, b) => a.sort_order.localeCompare(b.sort_order))
    }
  })

  /**
   * Returns a specific ordering by context and task.
   */
  function getOrdering(
    contextType: string,
    contextId: string,
    taskId: string
  ): Ordering | undefined {
    const key = `${contextType}:${contextId}:${taskId}`
    return orderings.value.get(key)
  }

  /**
   * Generates a key for storing orderings.
   */
  function generateKey(contextType: string, contextId: string, taskId: string): string {
    return `${contextType}:${contextId}:${taskId}`
  }

  // --- Actions ---

  /**
   * Fetches orderings for a specific context.
   */
  async function fetchOrderingsForContext(contextType: string, contextId: string) {
    isLoading.value = true
    error.value = null
    try {
      const params = new URLSearchParams()
      params.append('context_type', contextType)
      params.append('context_id', contextId)

      const response = await fetch(`${API_BASE_URL}/ordering?${params.toString()}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const orderingList: Ordering[] = apiResponse.data

      // Update orderings in the store
      for (const ordering of orderingList) {
        const key = generateKey(ordering.context_type, ordering.context_id, ordering.task_id)
        orderings.value.set(key, ordering)
      }

      console.log(
        `[OrderingStore] Fetched ${orderingList.length} orderings for context ${contextType}:${contextId}`
      )
      return orderingList
    } catch (e) {
      error.value = `Failed to fetch orderings for context: ${e}`
      console.error('[OrderingStore] Error fetching orderings:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Updates the order of a task in a specific context.
   */
  async function updateOrder(payload: UpdateOrderPayload) {
    isLoading.value = true
    error.value = null
    console.log(`[OrderingStore] Attempting to update order with payload:`, payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/ordering`, {
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

      // Update the ordering in the store optimistically
      const key = generateKey(payload.context_type, payload.context_id, payload.task_id)
      const existingOrdering = orderings.value.get(key)

      if (existingOrdering) {
        const updatedOrdering: Ordering = {
          ...existingOrdering,
          sort_order: payload.sort_order,
          updated_at: new Date().toISOString(),
        }
        orderings.value.set(key, updatedOrdering)
      } else {
        // Create a new ordering entry
        const newOrdering: Ordering = {
          id: `temp-${Date.now()}`, // Temporary ID
          context_type: payload.context_type,
          context_id: payload.context_id,
          task_id: payload.task_id,
          sort_order: payload.sort_order,
          updated_at: new Date().toISOString(),
        }
        orderings.value.set(key, newOrdering)
      }

      console.log(`[OrderingStore] Successfully updated order for task ${payload.task_id}`)
    } catch (e) {
      error.value = `Failed to update order: ${e}`
      console.error(`[OrderingStore] Error updating order:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Batch updates multiple orderings.
   */
  async function batchUpdateOrdering(orderingList: Ordering[]) {
    isLoading.value = true
    error.value = null
    console.log(`[OrderingStore] Attempting to batch update ${orderingList.length} orderings`)
    try {
      const response = await fetch(`${API_BASE_URL}/ordering/batch`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(orderingList),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      // Update orderings in the store
      for (const ordering of orderingList) {
        const key = generateKey(ordering.context_type, ordering.context_id, ordering.task_id)
        orderings.value.set(key, ordering)
      }

      console.log(`[OrderingStore] Successfully batch updated ${orderingList.length} orderings`)
    } catch (e) {
      error.value = `Failed to batch update orderings: ${e}`
      console.error(`[OrderingStore] Error batch updating orderings:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Calculates a sort order position between two existing positions.
   */
  async function calculateSortOrder(params: CalculateSortOrderParams): Promise<string | null> {
    isLoading.value = true
    error.value = null
    try {
      const searchParams = new URLSearchParams()
      searchParams.append('context_type', params.context_type)
      searchParams.append('context_id', params.context_id)
      if (params.prev_sort_order) {
        searchParams.append('prev_sort_order', params.prev_sort_order)
      }
      if (params.next_sort_order) {
        searchParams.append('next_sort_order', params.next_sort_order)
      }

      const response = await fetch(`${API_BASE_URL}/ordering/calculate?${searchParams.toString()}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const result = apiResponse.data

      console.log(`[OrderingStore] Calculated sort order:`, result.sort_order)
      return result.sort_order
    } catch (e) {
      error.value = `Failed to calculate sort order: ${e}`
      console.error('[OrderingStore] Error calculating sort order:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Clears all orderings for a specific context.
   */
  async function clearContextOrdering(contextType: string, contextId: string) {
    isLoading.value = true
    error.value = null
    try {
      const params = new URLSearchParams()
      params.append('context_type', contextType)
      params.append('context_id', contextId)

      const response = await fetch(`${API_BASE_URL}/ordering?${params.toString()}`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      // Remove orderings from the store
      const keysToRemove: string[] = []
      for (const [key, ordering] of orderings.value.entries()) {
        if (ordering.context_type === contextType && ordering.context_id === contextId) {
          keysToRemove.push(key)
        }
      }

      for (const key of keysToRemove) {
        orderings.value.delete(key)
      }

      console.log(
        `[OrderingStore] Successfully cleared orderings for context ${contextType}:${contextId}`
      )
    } catch (e) {
      error.value = `Failed to clear context ordering: ${e}`
      console.error('[OrderingStore] Error clearing context ordering:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Removes ordering for a specific task from a context.
   */
  function removeTaskFromContext(contextType: string, contextId: string, taskId: string) {
    const key = generateKey(contextType, contextId, taskId)
    orderings.value.delete(key)
    console.log(`[OrderingStore] Removed task ${taskId} from context ${contextType}:${contextId}`)
  }

  /**
   * Removes all orderings for a specific task.
   */
  function removeAllOrderingsForTask(taskId: string) {
    const keysToRemove: string[] = []
    for (const [key, ordering] of orderings.value.entries()) {
      if (ordering.task_id === taskId) {
        keysToRemove.push(key)
      }
    }

    for (const key of keysToRemove) {
      orderings.value.delete(key)
    }

    console.log(`[OrderingStore] Removed all orderings for task ${taskId}`)
  }

  return {
    // State
    orderings,
    isLoading,
    error,
    // Getters
    getOrderingsForContext,
    getOrderingsForTask,
    getOrdering,
    // Actions
    fetchOrderingsForContext,
    updateOrder,
    batchUpdateOrdering,
    calculateSortOrder,
    clearContextOrdering,
    removeTaskFromContext,
    removeAllOrderingsForTask,
  }
})

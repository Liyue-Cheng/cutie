import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

// --- Type Definitions ---
type ID = string

export interface Area {
  id: string
  name: string
  color: string
  parent_area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
}

export interface CreateAreaPayload {
  name: string
  color: string
  parent_area_id?: string | null
}

export interface UpdateAreaPayload {
  name?: string
  color?: string
  parent_area_id?: string | null
}

// --- API Base URL ---
import { waitForApiReady } from '@/composables/useApiConfig'

export const useAreaStore = defineStore('area', () => {
  // --- State ---
  const areas = ref(new Map<ID, Area>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // --- Getters ---

  /**
   * Returns all areas as a sorted array.
   */
  const allAreas = computed(() => {
    return Array.from(areas.value.values())
      .filter((area) => !area.is_deleted)
      .sort((a, b) => a.name.localeCompare(b.name))
  })

  /**
   * Returns root areas (areas with no parent).
   */
  const rootAreas = computed(() => {
    return Array.from(areas.value.values())
      .filter((area) => !area.is_deleted && !area.parent_area_id)
      .sort((a, b) => a.name.localeCompare(b.name))
  })

  /**
   * Returns child areas for a given parent area.
   */
  const getChildAreas = computed(() => {
    return (parentId: string) => {
      return Array.from(areas.value.values())
        .filter((area) => !area.is_deleted && area.parent_area_id === parentId)
        .sort((a, b) => a.name.localeCompare(b.name))
    }
  })

  /**
   * Returns a function to get an area by its ID.
   */
  function getAreaById(id: ID): Area | undefined {
    return areas.value.get(id)
  }

  /**
   * Returns the full path from root to the specified area.
   */
  function getAreaPath(areaId: ID): Area[] {
    const path: Area[] = []
    let currentArea = areas.value.get(areaId)

    while (currentArea) {
      path.unshift(currentArea)
      currentArea = currentArea.parent_area_id
        ? areas.value.get(currentArea.parent_area_id)
        : undefined
    }

    return path
  }

  // --- Actions ---

  /**
   * Fetches all areas.
   */
  async function fetchAreas() {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/areas`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const areaList: Area[] = apiResponse.data

      const areaMap = new Map<ID, Area>()
      for (const area of areaList) {
        areaMap.set(area.id, area)
      }
      areas.value = areaMap

      console.log(`[AreaStore] Fetched ${areaList.length} areas`)
    } catch (e) {
      error.value = `Failed to fetch areas: ${e}`
      console.error('[AreaStore] Error fetching areas:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches root areas only.
   */
  async function fetchRootAreas() {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/areas?roots_only=true`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const areaList: Area[] = apiResponse.data

      // Update areas in the store
      for (const area of areaList) {
        areas.value.set(area.id, area)
      }

      console.log(`[AreaStore] Fetched ${areaList.length} root areas`)
      return areaList
    } catch (e) {
      error.value = `Failed to fetch root areas: ${e}`
      console.error('[AreaStore] Error fetching root areas:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches child areas for a parent area.
   */
  async function fetchChildAreas(parentId: ID, includeDescendants: boolean = false) {
    isLoading.value = true
    error.value = null
    try {
      const params = new URLSearchParams()
      params.append('parent_id', parentId)
      if (includeDescendants) {
        params.append('include_descendants', 'true')
      }

      const response = await fetch(`${API_BASE_URL}/areas?${params.toString()}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const areaList: Area[] = apiResponse.data

      // Update areas in the store
      for (const area of areaList) {
        areas.value.set(area.id, area)
      }

      console.log(`[AreaStore] Fetched ${areaList.length} child areas for ${parentId}`)
      return areaList
    } catch (e) {
      error.value = `Failed to fetch child areas: ${e}`
      console.error('[AreaStore] Error fetching child areas:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches a single area by ID.
   */
  async function fetchArea(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/areas/${id}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const area: Area = apiResponse.data

      areas.value.set(area.id, area)
      console.log(`[AreaStore] Fetched area ${id}`)
      return area
    } catch (e) {
      error.value = `Failed to fetch area ${id}: ${e}`
      console.error(`[AreaStore] Error fetching area ${id}:`, e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Creates a new area.
   */
  async function createArea(payload: CreateAreaPayload) {
    isLoading.value = true
    error.value = null
    console.log(`[AreaStore] Attempting to create area with payload:`, payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/areas`, {
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
      const newArea: Area = apiResponse.data

      areas.value.set(newArea.id, newArea)
      console.log(`[AreaStore] Successfully created area:`, newArea)
      return newArea
    } catch (e) {
      error.value = `Failed to create area: ${e}`
      console.error(`[AreaStore] Error creating area:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Updates an existing area.
   */
  async function updateArea(id: ID, payload: UpdateAreaPayload) {
    isLoading.value = true
    error.value = null
    console.log(`[AreaStore] Attempting to update area ${id} with payload:`, payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/areas/${id}`, {
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
      const updatedArea: Area = apiResponse.data

      areas.value.set(id, updatedArea)
      console.log(`[AreaStore] Successfully updated area ${id}:`, updatedArea)
      return updatedArea
    } catch (e) {
      error.value = `Failed to update area ${id}: ${e}`
      console.error(`[AreaStore] Error updating area ${id}:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Deletes an area.
   */
  async function deleteArea(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/areas/${id}`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      areas.value.delete(id)
      console.log(`[AreaStore] Successfully deleted area ${id}`)
    } catch (e) {
      error.value = `Failed to delete area ${id}: ${e}`
      console.error('[AreaStore] Error deleting area:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Moves an area to a new parent.
   */
  async function moveArea(id: ID, newParentId: ID | null) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/areas/${id}/move`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          new_parent_id: newParentId,
        }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const movedArea: Area = apiResponse.data

      areas.value.set(id, movedArea)
      console.log(`[AreaStore] Successfully moved area ${id} to parent ${newParentId}`)
      return movedArea
    } catch (e) {
      error.value = `Failed to move area ${id}: ${e}`
      console.error('[AreaStore] Error moving area:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Checks if an area can be deleted.
   */
  async function checkCanDelete(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/areas/${id}/can-delete`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const result = apiResponse.data

      console.log(`[AreaStore] Can delete check for area ${id}:`, result)
      return result.can_delete
    } catch (e) {
      error.value = `Failed to check if area ${id} can be deleted: ${e}`
      console.error('[AreaStore] Error checking can delete:', e)
      return false
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches the full path for an area from the backend.
   */
  async function fetchAreaPath(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/areas/${id}/path`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const path: Area[] = apiResponse.data

      // Update areas in the store
      for (const area of path) {
        areas.value.set(area.id, area)
      }

      console.log(`[AreaStore] Fetched path for area ${id}:`, path)
      return path
    } catch (e) {
      error.value = `Failed to fetch path for area ${id}: ${e}`
      console.error('[AreaStore] Error fetching area path:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  return {
    // State
    areas,
    isLoading,
    error,
    // Getters
    allAreas,
    rootAreas,
    getChildAreas,
    getAreaById,
    getAreaPath,
    // Actions
    fetchAreas,
    fetchRootAreas,
    fetchChildAreas,
    fetchArea,
    createArea,
    updateArea,
    deleteArea,
    moveArea,
    checkCanDelete,
    fetchAreaPath,
  }
})

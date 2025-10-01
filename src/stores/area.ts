import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { waitForApiReady } from '@/composables/useApiConfig'

/**
 * Area Store
 *
 * 职责：管理区域（标签分类）数据
 *
 * 架构原则：
 * - State: 只存储最原始、最规范化的数据
 * - Actions: 负责执行操作、调用API、修改State
 * - Getters: 只负责从State中读取和计算数据，不修改State
 */

export interface Area {
  id: string
  name: string
  color: string
  parent_area_id: string | null
  created_at: string
  updated_at: string
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

export const useAreaStore = defineStore('area', () => {
  // ============================================================
  // STATE
  // ============================================================

  const areas = ref(new Map<string, Area>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // ============================================================
  // GETTERS
  // ============================================================

  const allAreas = computed(() => {
    return Array.from(areas.value.values()).sort((a, b) => a.name.localeCompare(b.name))
  })

  const rootAreas = computed(() => {
    return allAreas.value.filter((area) => !area.parent_area_id)
  })

  const getChildAreas = computed(() => {
    return (parentId: string) => {
      return allAreas.value.filter((area) => area.parent_area_id === parentId)
    }
  })

  function getAreaById(id: string): Area | undefined {
    return areas.value.get(id)
  }

  // ============================================================
  // ACTIONS
  // ============================================================

  function addOrUpdateAreas(newAreas: Area[]) {
    const newMap = new Map(areas.value)
    for (const area of newAreas) {
      newMap.set(area.id, area)
    }
    areas.value = newMap
  }

  function addOrUpdateArea(area: Area) {
    const newMap = new Map(areas.value)
    newMap.set(area.id, area)
    areas.value = newMap
  }

  function removeArea(id: string) {
    const newMap = new Map(areas.value)
    newMap.delete(id)
    areas.value = newMap
  }

  async function fetchAreas() {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/areas`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const areaList: Area[] = result.data
      addOrUpdateAreas(areaList)
      console.log('[AreaStore] Fetched', areaList.length, 'areas')
    } catch (e) {
      error.value = `Failed to fetch areas: ${e}`
      console.error('[AreaStore] Error fetching areas:', e)
    } finally {
      isLoading.value = false
    }
  }

  async function createArea(payload: CreateAreaPayload): Promise<Area | null> {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/areas`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const newArea: Area = result.data
      addOrUpdateArea(newArea)
      console.log('[AreaStore] Created area:', newArea)
      return newArea
    } catch (e) {
      error.value = `Failed to create area: ${e}`
      console.error('[AreaStore] Error creating area:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  async function updateArea(id: string, payload: UpdateAreaPayload): Promise<Area | null> {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/areas/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const updatedArea: Area = result.data
      addOrUpdateArea(updatedArea)
      console.log('[AreaStore] Updated area:', updatedArea)
      return updatedArea
    } catch (e) {
      error.value = `Failed to update area: ${e}`
      console.error('[AreaStore] Error updating area:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  async function deleteArea(id: string): Promise<boolean> {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/areas/${id}`, {
        method: 'DELETE',
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      removeArea(id)
      console.log('[AreaStore] Deleted area:', id)
      return true
    } catch (e) {
      error.value = `Failed to delete area: ${e}`
      console.error('[AreaStore] Error deleting area:', e)
      return false
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

    // Actions
    addOrUpdateAreas,
    addOrUpdateArea,
    removeArea,
    fetchAreas,
    createArea,
    updateArea,
    deleteArea,
  }
})

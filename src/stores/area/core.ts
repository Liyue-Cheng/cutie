/**
 * Area Store - Core (RTL 架构)
 *
 * 职责：管理 Area 数据的核心状态和 mutations
 *
 * RTL 架构：
 * - Register (寄存器): 纯响应式状态，只读
 * - Transmission (传输线): Getters，计算属性
 * - Logic (逻辑门): Mutations，纯数据操作（_mut 后缀）
 */

import { ref, computed } from 'vue'

export interface Area {
  id: string
  name: string
  color: string
  parent_area_id: string | null
  created_at: string
  updated_at: string
}

// ============================================================
// STATE (寄存器)
// ============================================================

export const areas = ref(new Map<string, Area>())

// ============================================================
// GETTERS (传输线)
// ============================================================

export const allAreas = computed(() => {
  return Array.from(areas.value.values()).sort((a, b) => a.name.localeCompare(b.name))
})

export const rootAreas = computed(() => {
  return allAreas.value.filter((area) => !area.parent_area_id)
})

export const getChildAreas = computed(() => {
  return (parentId: string) => {
    return allAreas.value.filter((area) => area.parent_area_id === parentId)
  }
})

export const getAreaById = computed(() => {
  return (id: string): Area | undefined => {
    return areas.value.get(id)
  }
})

// ============================================================
// MUTATIONS (逻辑门)
// ============================================================

/**
 * 添加或更新单个 Area
 */
export function addOrUpdate_mut(area: Area) {
  const newMap = new Map(areas.value)
  newMap.set(area.id, area)
  areas.value = newMap
}

/**
 * 批量添加或更新 Areas
 */
export function addOrUpdateBatch_mut(newAreas: Area[]) {
  const newMap = new Map(areas.value)
  for (const area of newAreas) {
    newMap.set(area.id, area)
  }
  areas.value = newMap
}

/**
 * 替换所有 Areas（用于初始加载）
 */
export function replaceAll_mut(newAreas: Area[]) {
  const newMap = new Map<string, Area>()
  for (const area of newAreas) {
    newMap.set(area.id, area)
  }
  areas.value = newMap
}

/**
 * 删除单个 Area
 */
export function remove_mut(id: string) {
  const newMap = new Map(areas.value)
  newMap.delete(id)
  areas.value = newMap
}

/**
 * 清空所有 Areas
 */
export function clear_mut() {
  areas.value = new Map()
}

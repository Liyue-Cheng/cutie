/**
 * Trash Store - Core State & Getters
 */
import { ref, computed } from 'vue'
import type { TaskCard } from '@/types/dtos'

// ==================== State ====================

export const trashedTasks = ref(new Map<string, TaskCard>())

// ==================== Getters ====================

export const allTrashedTasks = computed(() => {
  return Array.from(trashedTasks.value.values()).sort((a, b) => {
    // 按删除时间倒序排列（最近删除的在前）
    if (!a.deleted_at || !b.deleted_at) return 0
    return new Date(b.deleted_at).getTime() - new Date(a.deleted_at).getTime()
  })
})

export const trashedTaskCount = computed(() => trashedTasks.value.size)

export const getTrashedTaskById = computed(() => (id: string) => trashedTasks.value.get(id))

// ==================== Mutations ====================

export function addOrUpdateTrashedTask(task: TaskCard) {
  const newMap = new Map(trashedTasks.value)
  newMap.set(task.id, task)
  trashedTasks.value = newMap
}

export function removeTrashedTask(id: string) {
  const newMap = new Map(trashedTasks.value)
  newMap.delete(id)
  trashedTasks.value = newMap
}

export function clearAllTrashedTasks() {
  trashedTasks.value = new Map()
}

export function setTrashedTasks(tasks: TaskCard[]) {
  trashedTasks.value = new Map(tasks.map((task) => [task.id, task]))
}

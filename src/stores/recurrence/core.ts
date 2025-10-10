import { ref, computed } from 'vue'
import type { TaskRecurrence } from '@/types/dtos'

// ==================== State ====================
export const recurrences = ref(new Map<string, TaskRecurrence>())

// ==================== Getters ====================
export const allRecurrences = computed(() => Array.from(recurrences.value.values()))

export const getRecurrenceById = computed(() => (id: string) => recurrences.value.get(id))

// 按模板ID过滤
export const getRecurrencesByTemplateId = computed(
  () => (templateId: string) =>
    Array.from(recurrences.value.values()).filter((r) => r.template_id === templateId)
)

// 只获取激活的循环规则
export const activeRecurrences = computed(() =>
  Array.from(recurrences.value.values()).filter((r) => r.is_active)
)

// ==================== Mutations ====================
export function addOrUpdateRecurrence(recurrence: TaskRecurrence) {
  const newMap = new Map(recurrences.value)
  newMap.set(recurrence.id, recurrence)
  recurrences.value = newMap
}

export function removeRecurrence(id: string) {
  const newMap = new Map(recurrences.value)
  newMap.delete(id)
  recurrences.value = newMap
}

export function clearAll() {
  recurrences.value = new Map()
}

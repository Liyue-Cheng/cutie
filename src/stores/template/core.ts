import { ref, computed } from 'vue'
import type { Template } from '@/types/dtos'

// ==================== State ====================
export const templates = ref(new Map<string, Template>())

// ==================== Getters ====================
export const allTemplates = computed(() => Array.from(templates.value.values()))

export const getTemplateById = computed(() => (id: string) => templates.value.get(id))

export const generalTemplates = computed(() =>
  Array.from(templates.value.values()).filter((t) => t.category === 'GENERAL')
)

export const recurrenceTemplates = computed(() =>
  Array.from(templates.value.values()).filter((t) => t.category === 'RECURRENCE')
)

// ==================== Mutations ====================
export function addOrUpdateTemplate(template: Template) {
  const newMap = new Map(templates.value)
  newMap.set(template.id, template)
  templates.value = newMap
}

export function removeTemplate(id: string) {
  const newMap = new Map(templates.value)
  newMap.delete(id)
  templates.value = newMap
}

export function clearAll() {
  templates.value = new Map()
}

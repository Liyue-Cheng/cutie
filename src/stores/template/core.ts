import { ref, computed } from 'vue'
import type { Template } from '@/types/dtos'

// ==================== State ====================
export const templates = ref(new Map<string, Template>())

// ==================== Getters ====================
export const allTemplates = computed(() => Array.from(templates.value.values()))

export const getTemplateById = computed(() => (id: string) => templates.value.get(id))

function sortByLexoRank(items: Template[]): Template[] {
  return [...items].sort((a, b) => {
    const rankA = a.sort_rank || ''
    const rankB = b.sort_rank || ''
    if (rankA && rankB) return rankA.localeCompare(rankB)
    if (rankA) return -1
    if (rankB) return 1
    return new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
  })
}

export const generalTemplates = computed(() =>
  sortByLexoRank(Array.from(templates.value.values()).filter((t) => t.category === 'GENERAL'))
)

export const recurrenceTemplates = computed(() =>
  sortByLexoRank(Array.from(templates.value.values()).filter((t) => t.category === 'RECURRENCE'))
)

// ==================== Mutations ====================
export function addOrUpdateTemplate_mut(template: Template) {
  const newMap = new Map(templates.value)
  newMap.set(template.id, template)
  templates.value = newMap
}

export function removeTemplate_mut(id: string) {
  const newMap = new Map(templates.value)
  newMap.delete(id)
  templates.value = newMap
}

export function clearAllTemplates_mut() {
  templates.value = new Map()
}

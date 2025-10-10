import { apiBaseUrl } from '@/composables/useApiConfig'
import type { TaskRecurrence } from '@/types/dtos'
import { addOrUpdateRecurrence, clearAll } from './core'

export async function fetchAllRecurrences(): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/recurrences`)

  if (!response.ok) {
    throw new Error('Failed to fetch recurrences')
  }

  const responseData = await response.json()
  const recurrences: TaskRecurrence[] = responseData.data || responseData

  clearAll()
  recurrences.forEach(addOrUpdateRecurrence)
}

export async function fetchRecurrencesByTemplateId(templateId: string): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/recurrences?template_id=${templateId}`)

  if (!response.ok) {
    throw new Error('Failed to fetch recurrences for template')
  }

  const responseData = await response.json()
  const recurrences: TaskRecurrence[] = responseData.data || responseData

  // 不清空全部，只更新相关的
  recurrences.forEach(addOrUpdateRecurrence)
}

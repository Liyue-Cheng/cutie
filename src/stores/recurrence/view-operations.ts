import { apiGet } from '@/stores/shared'
import type { TaskRecurrence } from '@/types/dtos'
import { addOrUpdateRecurrence, clearAll } from './core'

export async function fetchAllRecurrences(): Promise<void> {
  const recurrences: TaskRecurrence[] = await apiGet('/recurrences')
  clearAll()
  recurrences.forEach(addOrUpdateRecurrence)
}

export async function fetchRecurrencesByTemplateId(templateId: string): Promise<void> {
  const recurrences: TaskRecurrence[] = await apiGet(`/recurrences?template_id=${templateId}`)
  // 不清空全部，只更新相关的
  recurrences.forEach(addOrUpdateRecurrence)
}

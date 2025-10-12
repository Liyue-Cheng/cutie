import { apiGet } from '@/stores/shared'
import type { Template } from '@/types/dtos'
import { addOrUpdateTemplate, clearAll } from './core'

export async function fetchAllTemplates(): Promise<void> {
  const templates: Template[] = await apiGet('/templates')
  clearAll()
  templates.forEach(addOrUpdateTemplate)
}

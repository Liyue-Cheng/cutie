import { apiGet } from '@/stores/shared'
import type { Template } from '@/types/dtos'
import { addOrUpdateTemplate_mut, clearAllTemplates_mut } from './core'

export async function fetchAllTemplates(): Promise<void> {
  const templates: Template[] = await apiGet('/templates')
  clearAllTemplates_mut()
  templates.forEach(addOrUpdateTemplate_mut)
}

import { apiBaseUrl } from '@/composables/useApiConfig'
import type { Template } from '@/types/dtos'
import { addOrUpdateTemplate, clearAll } from './core'

export async function fetchAllTemplates(): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/templates`)

  if (!response.ok) {
    throw new Error('Failed to fetch templates')
  }

  const templates: Template[] = await response.json()

  clearAll()
  templates.forEach(addOrUpdateTemplate)
}

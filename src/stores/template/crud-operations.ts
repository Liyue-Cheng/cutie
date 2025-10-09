import { apiBaseUrl } from '@/composables/useApiConfig'
import type { Template, TemplateCategory, TaskCard } from '@/types/dtos'
import { addOrUpdateTemplate, removeTemplate } from './core'

export interface CreateTemplatePayload {
  title: string
  glance_note_template?: string
  detail_note_template?: string
  estimated_duration_template?: number
  subtasks_template?: Array<{
    id: string
    title: string
    is_completed: boolean
    sort_order: string
  }>
  area_id?: string
  category?: TemplateCategory
}

export interface UpdateTemplatePayload {
  title?: string
  glance_note_template?: string
  detail_note_template?: string
  estimated_duration_template?: number
  subtasks_template?: Array<{
    id: string
    title: string
    is_completed: boolean
    sort_order: string
  }>
  area_id?: string
  category?: TemplateCategory
}

export async function createTemplate(payload: CreateTemplatePayload): Promise<Template> {
  const response = await fetch(`${apiBaseUrl.value}/templates`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    throw new Error('Failed to create template')
  }

  const template: Template = await response.json()
  addOrUpdateTemplate(template)
  return template
}

export async function updateTemplate(
  id: string,
  payload: UpdateTemplatePayload
): Promise<Template> {
  const response = await fetch(`${apiBaseUrl.value}/templates/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    throw new Error('Failed to update template')
  }

  const template: Template = await response.json()
  addOrUpdateTemplate(template)
  return template
}

export async function deleteTemplate(id: string): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/templates/${id}`, {
    method: 'DELETE',
  })

  if (!response.ok) {
    throw new Error('Failed to delete template')
  }

  removeTemplate(id)
}

export async function createTaskFromTemplate(
  templateId: string,
  variables: Record<string, string> = {}
): Promise<TaskCard> {
  console.log('[Template Store] Creating task from template', {
    templateId,
    variables,
    url: `${apiBaseUrl.value}/templates/${templateId}/create-task`,
  })

  const response = await fetch(`${apiBaseUrl.value}/templates/${templateId}/create-task`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(variables),
  })

  console.log('[Template Store] Response status', {
    ok: response.ok,
    status: response.status,
    statusText: response.statusText,
  })

  if (!response.ok) {
    const errorText = await response.text()
    console.error('[Template Store] Failed to create task', {
      status: response.status,
      errorText,
    })
    throw new Error('Failed to create task from template')
  }

  const responseData = await response.json()
  console.log('[Template Store] Received response data', {
    responseData,
    hasData: !!responseData?.data,
    dataKeys: responseData?.data ? Object.keys(responseData.data) : [],
  })

  // 提取 data 字段（后端使用 ApiResponse 包装）
  const taskCard: TaskCard = responseData.data
  console.log('[Template Store] Extracted task card', {
    taskCard,
    hasId: !!taskCard?.id,
    id: taskCard?.id,
    title: taskCard?.title,
  })

  return taskCard
}

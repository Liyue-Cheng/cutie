import { apiPost, apiPatch, apiDelete } from '@/stores/shared'
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
  const template: Template = await apiPost('/templates', payload)
  addOrUpdateTemplate(template)
  return template
}

export async function updateTemplate(
  id: string,
  payload: UpdateTemplatePayload
): Promise<Template> {
  const template: Template = await apiPatch(`/templates/${id}`, payload)
  addOrUpdateTemplate(template)
  return template
}

export async function deleteTemplate(id: string): Promise<void> {
  await apiDelete(`/templates/${id}`)
  removeTemplate(id)
}

export async function createTaskFromTemplate(
  templateId: string,
  variables: Record<string, string> = {}
): Promise<TaskCard> {
  console.log('[Template Store] Creating task from template', {
    templateId,
    variables,
    url: `/templates/${templateId}/create-task`,
  })

  const taskCard: TaskCard = await apiPost(`/templates/${templateId}/create-task`, variables)

  console.log('[Template Store] Extracted task card', {
    taskCard,
    hasId: !!taskCard?.id,
    id: taskCard?.id,
    title: taskCard?.title,
  })

  return taskCard
}

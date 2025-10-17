/**
 * 模板Store类型定义
 */

import type { TemplateCategory } from '@/types/dtos'

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

export interface CreateTaskFromTemplatePayload {
  template_id: string
  variables?: Record<string, string>
}

export interface CreateTemplateFromTaskPayload {
  task_id: string
  title?: string
  category?: TemplateCategory
}

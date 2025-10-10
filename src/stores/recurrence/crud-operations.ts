import { apiBaseUrl } from '@/composables/useApiConfig'
import type {
  CreateTaskRecurrencePayload,
  TaskRecurrence,
  UpdateTaskRecurrencePayload,
} from '@/types/dtos'
import { addOrUpdateRecurrence, removeRecurrence } from './core'

export async function createRecurrence(
  payload: CreateTaskRecurrencePayload
): Promise<TaskRecurrence> {
  const response = await fetch(`${apiBaseUrl.value}/recurrences`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    throw new Error('Failed to create recurrence')
  }

  // ✅ 正确：提取 .data 字段
  const responseData = await response.json()
  const recurrence: TaskRecurrence = responseData.data
  addOrUpdateRecurrence(recurrence)
  return recurrence
}

export async function updateRecurrence(
  id: string,
  payload: UpdateTaskRecurrencePayload
): Promise<TaskRecurrence> {
  const response = await fetch(`${apiBaseUrl.value}/recurrences/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })

  if (!response.ok) {
    throw new Error('Failed to update recurrence')
  }

  // ✅ 正确：提取 .data 字段
  const responseData = await response.json()
  const recurrence: TaskRecurrence = responseData.data
  addOrUpdateRecurrence(recurrence)
  return recurrence
}

export async function deleteRecurrence(id: string): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/recurrences/${id}`, {
    method: 'DELETE',
  })

  if (!response.ok) {
    throw new Error('Failed to delete recurrence')
  }

  removeRecurrence(id)
}

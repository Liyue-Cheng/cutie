import { apiPost, apiPatch, apiDelete } from '@/stores/shared'
import type {
  CreateTaskRecurrencePayload,
  TaskRecurrence,
  UpdateTaskRecurrencePayload,
} from '@/types/dtos'
import { addOrUpdateRecurrence, removeRecurrence } from './core'

export async function createRecurrence(
  payload: CreateTaskRecurrencePayload
): Promise<TaskRecurrence> {
  const recurrence: TaskRecurrence = await apiPost('/recurrences', payload)
  addOrUpdateRecurrence(recurrence)
  return recurrence
}

export async function updateRecurrence(
  id: string,
  payload: UpdateTaskRecurrencePayload
): Promise<TaskRecurrence> {
  const recurrence: TaskRecurrence = await apiPatch(`/recurrences/${id}`, payload)
  addOrUpdateRecurrence(recurrence)
  return recurrence
}

export async function deleteRecurrence(id: string): Promise<void> {
  await apiDelete(`/recurrences/${id}`)
  removeRecurrence(id)
}

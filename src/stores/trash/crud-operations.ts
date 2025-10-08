/**
 * Trash Store - CRUD Operations
 */
import { apiBaseUrl } from '@/composables/useApiConfig'
import { removeTrashedTask } from './core'
import type { TaskCard } from '@/types/dtos'

/**
 * 恢复任务（从回收站）
 */
export async function restoreTask(taskId: string): Promise<TaskCard> {
  const response = await fetch(`${apiBaseUrl.value}/tasks/${taskId}/restore`, {
    method: 'PATCH',
  })
  if (!response.ok) throw new Error('Failed to restore task')

  const result: { data: TaskCard } = await response.json()
  removeTrashedTask(taskId) // 从回收站移除
  return result.data
}

/**
 * 彻底删除任务
 */
export async function permanentlyDeleteTask(taskId: string): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/tasks/${taskId}/permanently`, {
    method: 'DELETE',
  })
  if (!response.ok) throw new Error('Failed to permanently delete task')

  // 后端返回 { data: { success: boolean } }，但我们不需要使用返回值
  removeTrashedTask(taskId) // 从回收站移除
}

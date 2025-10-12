/**
 * Trash Store - CRUD Operations
 */
import { apiPatch, apiDelete } from '@/stores/shared'
import { removeTrashedTask } from './core'
import type { TaskCard } from '@/types/dtos'

/**
 * 恢复任务（从回收站）
 */
export async function restoreTask(taskId: string): Promise<TaskCard> {
  const restoredTask: TaskCard = await apiPatch(`/tasks/${taskId}/restore`)
  removeTrashedTask(taskId) // 从回收站移除
  return restoredTask
}

/**
 * 彻底删除任务
 */
export async function permanentlyDeleteTask(taskId: string): Promise<void> {
  await apiDelete(`/tasks/${taskId}/permanently`)
  removeTrashedTask(taskId) // 从回收站移除
}

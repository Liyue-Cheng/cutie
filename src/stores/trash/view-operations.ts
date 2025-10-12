/**
 * Trash Store - View Operations
 */
import { apiGet, apiPost } from '@/stores/shared'
import { setTrashedTasks } from './core'
import type { TaskCard } from '@/types/dtos'

/**
 * 获取回收站列表
 */
export async function fetchTrash(options?: { limit?: number; offset?: number }): Promise<void> {
  const limit = options?.limit || 50
  const offset = options?.offset || 0

  const result: { tasks: TaskCard[]; total: number } = await apiGet(
    `/trash?limit=${limit}&offset=${offset}`
  )
  setTrashedTasks(result.tasks)
}

/**
 * 清空回收站
 */
export async function emptyTrash(options?: {
  olderThanDays?: number
  limit?: number
}): Promise<number> {
  const result: { deleted_count: number } = await apiPost('/trash/empty', {
    older_than_days: options?.olderThanDays ?? 30,
    limit: options?.limit ?? 100,
  })
  return result.deleted_count
}

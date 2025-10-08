/**
 * Trash Store - View Operations
 */
import { apiBaseUrl } from '@/composables/useApiConfig'
import { setTrashedTasks } from './core'
import type { TaskCard } from '@/types/dtos'

/**
 * 获取回收站列表
 */
export async function fetchTrash(options?: { limit?: number; offset?: number }): Promise<void> {
  const limit = options?.limit || 50
  const offset = options?.offset || 0

  const response = await fetch(`${apiBaseUrl.value}/trash?limit=${limit}&offset=${offset}`)
  if (!response.ok) throw new Error('Failed to fetch trash')

  const result: { data: { tasks: TaskCard[]; total: number } } = await response.json()
  setTrashedTasks(result.data.tasks)
}

/**
 * 清空回收站
 */
export async function emptyTrash(options?: {
  olderThanDays?: number
  limit?: number
}): Promise<number> {
  const response = await fetch(`${apiBaseUrl.value}/trash/empty`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      older_than_days: options?.olderThanDays ?? 30, // 使用 ?? 而不是 ||，因为 0 是有效值
      limit: options?.limit ?? 100,
    }),
  })
  if (!response.ok) throw new Error('Failed to empty trash')

  const result: { data: { deleted_count: number } } = await response.json()
  return result.data.deleted_count
}

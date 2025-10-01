/**
 * 视图API适配层
 *
 * 职责：仅仅解决后端分散的视图端点与前端统一调用的适配问题
 *
 * 后端现状：
 * - GET /views/all
 * - GET /views/staging
 * - GET /views/planned
 * - GET /views/all-incomplete
 * - GET /views/daily-schedule?day=YYYY-MM-DD
 *
 * 前端期望：
 * - fetchView({ type: 'staging' }) → GET /views/staging
 * - fetchView({ type: 'daily_kanban', date: '2024-10-01' }) → GET /views/daily-schedule?day=2024-10-01
 */

import { waitForApiReady } from '@/composables/useApiConfig'
import type { TaskCard } from '@/types/dtos'

export type ViewContext =
  | { type: 'all' }
  | { type: 'all_incomplete' }
  | { type: 'staging' }
  | { type: 'planned' }
  | { type: 'daily_kanban'; date: string }
  | { type: 'project_list'; projectId: string }
  | { type: 'area_filter'; areaId: string }

/**
 * 统一的视图获取函数
 * 根据上下文类型，适配到对应的后端端点
 */
export async function fetchView(context: ViewContext): Promise<TaskCard[]> {
  const apiBaseUrl = await waitForApiReady()
  let endpoint: string

  switch (context.type) {
    case 'all':
      endpoint = '/views/all'
      break
    case 'all_incomplete':
      endpoint = '/views/all-incomplete'
      break
    case 'staging':
      endpoint = '/views/staging'
      break
    case 'planned':
      endpoint = '/views/planned'
      break
    case 'daily_kanban':
      endpoint = `/views/daily-schedule?day=${context.date}`
      break
    case 'project_list':
      endpoint = `/views/project/${context.projectId}`
      break
    case 'area_filter':
      endpoint = `/views/area/${context.areaId}`
      break
    default:
      throw new Error(`Unknown view context type: ${(context as any).type}`)
  }

  const response = await fetch(`${apiBaseUrl}${endpoint}`)
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${endpoint}`)
  }

  const result = await response.json()
  return result.data
}

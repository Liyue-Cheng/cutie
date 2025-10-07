/**
 * Task Store 类型定义
 */

// --- Payload Types for API calls ---
export interface CreateTaskPayload {
  title: string
  glance_note?: string | null
  detail_note?: string | null
  area_id?: string | null
  due_date?: string | null
  due_date_type?: 'soft' | 'hard' | null
  project_id?: string | null
  subtasks?: Array<{
    title: string
    is_completed: boolean
  }> | null
}

export interface UpdateTaskPayload {
  title?: string
  glance_note?: string | null
  detail_note?: string | null
  area_id?: string | null
  due_date?: string | null
  due_date_type?: 'soft' | 'hard' | null
  project_id?: string | null
  estimated_duration?: number | null
  subtasks?: Array<{
    id?: string
    title: string
    is_completed: boolean
  }> | null
}

/**
 * 完成任务的响应数据
 */
export interface CompleteTaskResponse {
  task: import('@/types/dtos').TaskCard
  // 注意：副作用（deleted/truncated time blocks）已通过 SSE 推送
}

/**
 * 删除任务的响应数据（副作用通过SSE）
 */
export interface DeleteTaskResponse {
  success: boolean
}

/**
 * 重新打开任务的响应数据
 */
export interface ReopenTaskResponse {
  task: import('@/types/dtos').TaskCard
}

/**
 * 归档任务的响应数据
 */
export interface ArchiveTaskResponse {
  task: import('@/types/dtos').TaskCard
}

/**
 * 取消归档任务的响应数据
 */
export interface UnarchiveTaskResponse {
  task: import('@/types/dtos').TaskCard
}

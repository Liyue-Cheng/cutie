/**
 * API 请求和响应的类型定义
 *
 * 职责：集中管理所有与后端 API 交互的类型
 * - Request Payloads: 发送给后端的数据结构
 * - Response Types: 后端返回的数据结构
 *
 * 命名规范：
 * - Payload: 请求体类型（如 CreateTaskPayload）
 * - Response: 响应体类型（如 CompleteTaskResponse）
 */

import type { TaskCard, TimeBlockView } from './dtos'

// ==================== 通用响应类型 ====================

/**
 * 标准 API 响应包装
 */
export interface ApiResponse<T> {
  data: T
  timestamp?: string
}

// ==================== Task API ====================

export namespace TaskAPI {
  /**
   * 创建任务的请求体
   */
  export interface CreatePayload {
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

  /**
   * 更新任务的请求体
   */
  export interface UpdatePayload {
    title?: string
    glance_note?: string | null
    detail_note?: string | null
    area_id?: string | null
    due_date?: string | null
    due_date_type?: 'soft' | 'hard' | null
    project_id?: string | null
    subtasks?: Array<{
      id?: string
      title: string
      is_completed: boolean
    }> | null
  }

  /**
   * 完成任务的响应数据
   * 注意：副作用（deleted/truncated time blocks）通过 SSE 事件推送
   */
  export interface CompleteResponse {
    task: TaskCard
  }

  /**
   * 删除任务的响应数据
   * 注意：副作用（deleted orphan time blocks）通过 SSE 事件推送
   */
  export interface DeleteResponse {
    success: boolean
  }

  /**
   * 重新打开任务的响应数据
   */
  export interface ReopenResponse {
    task: TaskCard
  }
}

// ==================== TimeBlock API ====================

export namespace TimeBlockAPI {
  /**
   * 创建空时间块的请求体
   */
  export interface CreatePayload {
    title?: string | null
    glance_note?: string | null
    detail_note?: string | null
    start_time: string // ISO 8601 UTC
    end_time: string // ISO 8601 UTC
    /** 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
    start_time_local?: string | null
    /** 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
    end_time_local?: string | null
    /** 时间类型，默认为FLOATING */
    time_type?: import('@/types/dtos').TimeType
    /** 创建时的时区（占位字段） */
    creation_timezone?: string | null
    area_id?: string | null
  }

  /**
   * 从任务创建时间块的请求体
   */
  export interface CreateFromTaskPayload {
    task_id: string
    start_time: string // ISO 8601 UTC
    end_time: string // ISO 8601 UTC
    title?: string | null // 可选，默认使用任务标题
  }

  /**
   * 从任务创建时间块的响应数据
   */
  export interface CreateFromTaskResponse {
    time_block: TimeBlockView
    updated_task: TaskCard
  }

  /**
   * 更新时间块的请求体
   */
  export interface UpdatePayload {
    title?: string | null
    glance_note?: string | null
    detail_note?: string | null
    start_time?: string
    end_time?: string
    /** 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
    start_time_local?: string | null
    /** 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
    end_time_local?: string | null
    /** 时间类型 */
    time_type?: import('@/types/dtos').TimeType
    /** 创建时的时区（占位字段） */
    creation_timezone?: string | null
    area_id?: string | null
  }

  /**
   * 链接任务到时间块的请求体
   */
  export interface LinkTaskPayload {
    task_id: string
  }
}

// ==================== Area API ====================

export namespace AreaAPI {
  /**
   * 创建区域的请求体
   */
  export interface CreatePayload {
    name: string
    color: string
    parent_area_id?: string | null
  }

  /**
   * 更新区域的请求体
   */
  export interface UpdatePayload {
    name?: string
    color?: string
    parent_area_id?: string | null
  }
}

// ==================== Schedule/View API ====================

export namespace ScheduleAPI {
  /**
   * 安排任务到指定日期的请求体
   */
  export interface ScheduleTaskPayload {
    task_id: string
    scheduled_day: string // YYYY-MM-DD ISO 8601 UTC
  }

  /**
   * 重新安排任务的请求体
   */
  export interface RescheduleTaskPayload {
    task_id: string
    from_day: string
    to_day: string
  }

  /**
   * 重新排序任务的请求体
   */
  export interface ReorderTasksPayload {
    date: string
    task_ids: string[]
  }
}

// ==================== 导出便捷类型别名 ====================

// Task
export type CreateTaskPayload = TaskAPI.CreatePayload
export type UpdateTaskPayload = TaskAPI.UpdatePayload
export type CompleteTaskResponse = TaskAPI.CompleteResponse
export type DeleteTaskResponse = TaskAPI.DeleteResponse
export type ReopenTaskResponse = TaskAPI.ReopenResponse

// TimeBlock
export type CreateTimeBlockPayload = TimeBlockAPI.CreatePayload
export type CreateTimeBlockFromTaskPayload = TimeBlockAPI.CreateFromTaskPayload
export type CreateTimeBlockFromTaskResponse = TimeBlockAPI.CreateFromTaskResponse
export type UpdateTimeBlockPayload = TimeBlockAPI.UpdatePayload

// Area
export type CreateAreaPayload = AreaAPI.CreatePayload
export type UpdateAreaPayload = AreaAPI.UpdatePayload

// Schedule
export type ScheduleTaskPayload = ScheduleAPI.ScheduleTaskPayload
export type RescheduleTaskPayload = ScheduleAPI.RescheduleTaskPayload
export type ReorderTasksPayload = ScheduleAPI.ReorderTasksPayload

/**
 * 后端数据库实体类型定义
 *
 * 注意：这些是后端数据库的原始实体类型，主要用于理解数据结构。
 * 前端应该使用 dtos.ts 中定义的视图模型（TaskCard, TaskDetail, TimeBlockView 等）。
 *
 * 这个文件保留用于：
 * 1. 理解后端数据结构
 * 2. 某些需要直接操作原始数据的场景（少数）
 */

// --- Base Types ---

export type ID = string

export type ContextType = 'DAILY_KANBAN' | 'PROJECT_LIST' | 'AREA_FILTER' | 'MISC'

export type DueDateType = 'SOFT' | 'HARD'

export type Outcome = 'PLANNED' | 'PRESENCE_LOGGED' | 'COMPLETED_ON_DAY' | 'CARRIED_OVER'

// --- Core Entities (保留用于理解后端结构) ---

/**
 * Area 实体 - 用户定义的分类标签
 */
export interface Area {
  id: string
  name: string
  color: string
  parent_area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
}

/**
 * Project 实体（V1.0 暂不实现 API）
 */
export interface Project {
  id: string
  title: string
  description: string | null
  icon: string | null
  color: string | null
  status: 'active' | 'on_hold' | 'archived'
  metadata: Record<string, any> | null
  created_at: string
  updated_at: string
  deleted_at: string | null
}

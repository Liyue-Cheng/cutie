/**
 * This file contains the core TypeScript types that mirror the backend database schema.
 * These types are the "source of truth" for the application's data structures.
 *
 * 所有类型定义都与后端 Rust 实体保持完全一致。
 */

// --- Base Types ---

/**
 * A unique identifier, typically a UUID string.
 */
export type ID = string

/**
 * Context types for task organization and filtering.
 */
export type ContextType = 'DAILY_KANBAN' | 'PROJECT_LIST' | 'AREA_FILTER' | 'MISC'

/**
 * Due date types for tasks.
 */
export type DueDateType = 'SOFT' | 'HARD'

/**
 * Task schedule outcome types.
 */
export type Outcome = 'PLANNED' | 'PRESENCE_LOGGED' | 'COMPLETED_ON_DAY' | 'CARRIED_OVER'

// --- Core Entities ---

/**
 * Represents a Subtask within a task.
 * 对应后端: entities::task::Subtask
 */
export interface Subtask {
  id: string
  title: string
  is_completed: boolean
  sort_order: string
}

/**
 * Represents a Task: a concrete, actionable item.
 * 对应后端: entities::task::Task
 */
export interface Task {
  id: string
  title: string
  glance_note: string | null
  detail_note: string | null
  estimated_duration: number | null
  subtasks: Subtask[] | null
  project_id: string | null
  area_id: string | null
  due_date: string | null
  due_date_type: DueDateType | null
  completed_at: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
  source_info: Record<string, any> | null
  external_source_id: string | null
  external_source_provider: string | null
  external_source_metadata: Record<string, any> | null
  recurrence_rule: string | null
  recurrence_parent_id: string | null
  recurrence_original_date: string | null
  recurrence_exclusions: string[] | null
}

/**
 * Represents a TaskSchedule: a link between a task and a specific day.
 * 对应后端: entities::schedule::TaskSchedule
 */
export interface TaskSchedule {
  id: string
  task_id: string
  scheduled_day: string
  outcome: Outcome
  created_at: string
  updated_at: string
}

/**
 * Represents a TimeBlock: a block of time on the calendar.
 * 对应后端: entities::time_block::TimeBlock
 */
export interface TimeBlock {
  id: string
  title: string | null
  glance_note: string | null
  detail_note: string | null
  start_time: string
  end_time: string
  area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
  source_info: Record<string, any> | null
  external_source_id: string | null
  external_source_provider: string | null
  external_source_metadata: Record<string, any> | null
  recurrence_rule: string | null
  recurrence_parent_id: string | null
  recurrence_original_date: string | null
  recurrence_exclusions: string[] | null
}

/**
 * Represents an Ordering: defines the sort position of a task in a context.
 * 对应后端: entities::ordering::Ordering
 */
export interface Ordering {
  id: string
  context_type: ContextType
  context_id: string
  task_id: string
  sort_order: string
  updated_at: string
}

/**
 * Represents a Template: a preset configuration for creating tasks.
 * 对应后端: entities::template::Template
 */
export interface Template {
  id: string
  name: string
  title_template: string
  glance_note_template: string | null
  detail_note_template: string | null
  estimated_duration_template: number | null
  subtasks_template: Subtask[] | null
  area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
}

/**
 * Represents an Area: a user-defined categorization label with color.
 * 对应后端: entities::area::Area
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

// --- Legacy Types (保留用于兼容旧代码，逐步迁移) ---

/**
 * Represents the status of a project.
 */
export type ProjectStatus = 'active' | 'on_hold' | 'archived'

/**
 * Represents a Project (V1.0 暂不实现 API，仅建表).
 */
export interface Project {
  id: ID
  title: string
  description: string | null
  icon: string | null
  color: string | null
  status: ProjectStatus
  metadata: Record<string, any> | null
  created_at: string
  updated_at: string
  deleted_at: string | null
  remote_updated_at: string | null
}

/**
 * Represents a Checkpoint (V1.0 暂不实现).
 */
export interface Checkpoint {
  id: ID
  task_id: ID
  title: string
  is_completed: boolean
  sort_key: string
  created_at: string
  updated_at: string
  deleted_at: string | null
  remote_updated_at: string | null
}

/**
 * Represents an Activity (V1.0 暂不实现).
 */
export interface Activity {
  id: ID
  title: string | null
  start_time: string
  end_time: string
  timezone: string | null
  is_all_day: boolean
  color: string | null
  metadata: Record<string, any> | null
  origin_id: string | null
  connector_id: string | null
  created_at: string
  updated_at: string
  deleted_at: string | null
  remote_updated_at: string | null
}

/**
 * Represents a Tag (V1.0 暂不实现).
 */
export interface Tag {
  id: ID
  title: string
  color: string | null
  sort_key: string | null
  created_at: string
  updated_at: string
  deleted_at: string | null
  remote_updated_at: string | null
}

/**
 * Represents a link between a Task and an Activity (V1.0 暂不实现).
 */
export interface TaskActivityLink {
  id: ID
  task_id: ID
  activity_id: ID
}

/**
 * Represents the join table for projects and tags (V1.0 暂不实现).
 */
export interface ProjectTag {
  project_id: ID
  tag_id: ID
}

/**
 * Represents the join table for tasks and tags (V1.0 暂不实现).
 */
export interface TaskTag {
  task_id: ID
  tag_id: ID
}

/**
 * Represents a key-value setting in the application (V1.0 暂不实现).
 */
export interface Setting {
  key: string
  value: any
}

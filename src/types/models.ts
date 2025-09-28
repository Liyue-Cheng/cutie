/**
 * This file contains the core TypeScript types that mirror the database schema.
 * These types are the "source of truth" for the application's data structures.
 */

// --- Base Timestamps and Identifiers ---

/**
 * A unique identifier, typically a UUID.
 */
type ID = string

/**
 * Standard timestamps for all entities.
 * Timestamps are stored as numbers (Unix epoch in milliseconds).
 */
interface Timestamps {
  created_at: string
  updated_at: string
  deleted_at: string | null
  remote_updated_at: string | null
}

// --- Core Entities ---

/**
 * Represents the status of a project.
 * 'active': The project is ongoing.
 * 'on_hold': The project is paused.
 * 'archived': The project is completed and archived.
 */
export type ProjectStatus = 'active' | 'on_hold' | 'archived'

/**
 * Represents a Project: a large goal or an area of exploration.
 */
export interface Project extends Timestamps {
  id: ID
  title: string
  description: string | null
  icon: string | null
  color: string | null
  status: ProjectStatus
  metadata: Record<string, any> | null
}

/**
 * Represents the status of a task.
 * Based on completed_at field: null = not completed, string = completed
 */
export type TaskStatus = 'todo' | 'done'

/**
 * Represents a Task: a concrete, actionable item.
 * Updated to match the new backend API structure.
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
  due_date_type: 'SOFT' | 'HARD' | null
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
 * Represents a subtask within a task.
 */
export interface Subtask {
  id: string
  title: string
  is_completed: boolean
  sort_order: string
}

/**
 * Represents a Checkpoint: a small, guiding step within a task.
 */
export interface Checkpoint extends Timestamps {
  id: ID
  task_id: ID
  title: string
  is_completed: boolean
  sort_key: string
}

/**
 * Represents an Activity: a pure block of time on the calendar.
 */
export interface Activity extends Timestamps {
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
}

/**
 * Represents a Tag: a cross-cutting label for organizing entities.
 */
export interface Tag extends Timestamps {
  id: ID
  title: string
  color: string | null
  sort_key: string | null
}

// --- Link/Junction Types ---

/**
 * Represents a link between a Task and an Activity.
 */
export interface TaskActivityLink {
  id: ID
  task_id: ID
  activity_id: ID
}

/**
 * Represents the join table for projects and tags.
 */
export interface ProjectTag {
  project_id: ID
  tag_id: ID
}

/**
 * Represents the join table for tasks and tags.
 */
export interface TaskTag {
  task_id: ID
  tag_id: ID
}

// --- System Types ---

/**
 * Represents a key-value setting in the application.
 */
export interface Setting {
  key: string
  value: any
}

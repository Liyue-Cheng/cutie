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
  created_at: number
  updated_at: number
  deleted_at: number | null
  remote_updated_at: number | null
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
 * 'todo': The task has not been started.
 * 'in_progress': The task is currently being worked on.
 * 'done': The task is completed.
 * 'canceled': The task has been canceled.
 */
export type TaskStatus = 'todo' | 'in_progress' | 'done' | 'canceled'

/**
 * Represents a Task: a concrete, actionable item.
 */
export interface Task extends Timestamps {
  id: ID
  project_id: ID | null
  title: string
  status: TaskStatus
  due_date: number | null
  completed_at: number | null
  sort_key: string
  metadata: Record<string, any> | null
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
  start_time: number
  end_time: number
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

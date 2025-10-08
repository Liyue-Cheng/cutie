/**
 * Trash Store - Composition
 */
import { defineStore } from 'pinia'
import * as core from './core'
import * as crud from './crud-operations'
import * as view from './view-operations'
import * as events from './event-handlers'

export const useTrashStore = defineStore('trash', () => {
  return {
    // State & Getters
    trashedTasks: core.trashedTasks,
    allTrashedTasks: core.allTrashedTasks,
    trashedTaskCount: core.trashedTaskCount,
    getTrashedTaskById: core.getTrashedTaskById,

    // CRUD Operations
    restoreTask: crud.restoreTask,
    permanentlyDeleteTask: crud.permanentlyDeleteTask,

    // View Operations
    fetchTrash: view.fetchTrash,
    emptyTrash: view.emptyTrash,

    // Event Subscriptions
    initEventSubscriptions: events.initEventSubscriptions,
  }
})

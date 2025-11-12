import { defineStore } from 'pinia'
import * as core from './core'
import * as crud from './crud-operations'
import * as view from './view-operations'
import * as events from './event-handlers'

export const useRecurrenceStore = defineStore('recurrence', () => {
  return {
    // State & Getters
    recurrences: core.recurrences,
    allRecurrences: core.allRecurrences,
    activeRecurrences: core.activeRecurrences,
    getRecurrenceById: core.getRecurrenceById,
    getRecurrencesByTemplateId: core.getRecurrencesByTemplateId,

    // CRUD Actions
    createRecurrence: crud.createRecurrence,
    updateRecurrence: crud.updateRecurrence,
    deleteRecurrence: crud.deleteRecurrence,

    // View Actions
    fetchAllRecurrences: view.fetchAllRecurrences,
    fetchRecurrencesByTemplateId: view.fetchRecurrencesByTemplateId,

    // Event Handling (SSE 事件处理)
    initEventSubscriptions: events.initEventSubscriptions,
  }
})

import { defineStore } from 'pinia'
import * as core from './core'
import * as crud from './crud-operations'
import * as view from './view-operations'

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
  }
})

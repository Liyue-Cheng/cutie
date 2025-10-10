import { defineStore } from 'pinia'
import * as core from './core'
import * as crud from './crud-operations'
import * as view from './view-operations'
import * as events from './event-handlers'

export const useTemplateStore = defineStore('template', () => {
  return {
    // State & Getters
    templates: core.templates,
    allTemplates: core.allTemplates,
    getTemplateById: core.getTemplateById,
    generalTemplates: core.generalTemplates,
    recurrenceTemplates: core.recurrenceTemplates,

    // CRUD Actions
    createTemplate: crud.createTemplate,
    updateTemplate: crud.updateTemplate,
    deleteTemplate: crud.deleteTemplate,
    createTaskFromTemplate: crud.createTaskFromTemplate,

    // View Actions
    fetchAllTemplates: view.fetchAllTemplates,

    // Event Handlers
    initEventSubscriptions: events.initEventSubscriptions,
  }
})

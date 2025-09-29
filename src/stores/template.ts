import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { Task, Subtask } from '@/types/models'

// --- Type Definitions ---
type ID = string

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

export interface CreateTemplatePayload {
  name: string
  title_template: string
  glance_note_template?: string | null
  detail_note_template?: string | null
  estimated_duration_template?: number | null
  subtasks_template?: Subtask[] | null
  area_id?: string | null
}

export interface UpdateTemplatePayload {
  name?: string
  title_template?: string
  glance_note_template?: string | null
  detail_note_template?: string | null
  estimated_duration_template?: number | null
  subtasks_template?: Subtask[] | null
  area_id?: string | null
}

export interface CreationContext {
  context_type: 'MISC' | 'DAILY_KANBAN' | 'PROJECT_LIST' | 'AREA_FILTER'
  context_id: string
}

// --- API Base URL ---
const API_BASE_URL = 'http://localhost:3030/api'

export const useTemplateStore = defineStore('template', () => {
  // --- State ---
  const templates = ref(new Map<ID, Template>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // --- Getters ---

  /**
   * Returns all templates as a sorted array.
   */
  const allTemplates = computed(() => {
    return Array.from(templates.value.values())
      .filter((template) => !template.is_deleted)
      .sort((a, b) => a.name.localeCompare(b.name))
  })

  /**
   * Returns templates for a specific area.
   */
  const getTemplatesForArea = computed(() => {
    return (areaId: string) => {
      return Array.from(templates.value.values())
        .filter((template) => !template.is_deleted && template.area_id === areaId)
        .sort((a, b) => a.name.localeCompare(b.name))
    }
  })

  /**
   * Returns a function to get a template by its ID.
   */
  function getTemplateById(id: ID): Template | undefined {
    return templates.value.get(id)
  }

  // --- Actions ---

  /**
   * Fetches all templates.
   */
  async function fetchTemplates() {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/templates`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const templateList: Template[] = apiResponse.data

      const templateMap = new Map<ID, Template>()
      for (const template of templateList) {
        templateMap.set(template.id, template)
      }
      templates.value = templateMap

      console.log(`[TemplateStore] Fetched ${templateList.length} templates`)
    } catch (e) {
      error.value = `Failed to fetch templates: ${e}`
      console.error('[TemplateStore] Error fetching templates:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Searches templates by name.
   */
  async function searchTemplates(query: string) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/templates?q=${encodeURIComponent(query)}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const templateList: Template[] = apiResponse.data

      // Update templates in the store
      for (const template of templateList) {
        templates.value.set(template.id, template)
      }

      console.log(`[TemplateStore] Found ${templateList.length} templates matching "${query}"`)
      return templateList
    } catch (e) {
      error.value = `Failed to search templates: ${e}`
      console.error('[TemplateStore] Error searching templates:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches templates for a specific area.
   */
  async function fetchTemplatesForArea(areaId: string) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${API_BASE_URL}/templates?area_id=${encodeURIComponent(areaId)}`
      )
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const templateList: Template[] = apiResponse.data

      // Update templates in the store
      for (const template of templateList) {
        templates.value.set(template.id, template)
      }

      console.log(`[TemplateStore] Fetched ${templateList.length} templates for area ${areaId}`)
      return templateList
    } catch (e) {
      error.value = `Failed to fetch templates for area ${areaId}: ${e}`
      console.error('[TemplateStore] Error fetching templates for area:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches templates that contain a specific variable.
   */
  async function fetchTemplatesWithVariable(variable: string) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${API_BASE_URL}/templates?variable=${encodeURIComponent(variable)}`
      )
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const templateList: Template[] = apiResponse.data

      // Update templates in the store
      for (const template of templateList) {
        templates.value.set(template.id, template)
      }

      console.log(
        `[TemplateStore] Fetched ${templateList.length} templates with variable "${variable}"`
      )
      return templateList
    } catch (e) {
      error.value = `Failed to fetch templates with variable ${variable}: ${e}`
      console.error('[TemplateStore] Error fetching templates with variable:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Fetches a single template by ID.
   */
  async function fetchTemplate(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/templates/${id}`)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const template: Template = apiResponse.data

      templates.value.set(template.id, template)
      console.log(`[TemplateStore] Fetched template ${id}`)
      return template
    } catch (e) {
      error.value = `Failed to fetch template ${id}: ${e}`
      console.error(`[TemplateStore] Error fetching template ${id}:`, e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Creates a new template.
   */
  async function createTemplate(payload: CreateTemplatePayload) {
    isLoading.value = true
    error.value = null
    console.log(`[TemplateStore] Attempting to create template with payload:`, payload)
    try {
      const response = await fetch(`${API_BASE_URL}/templates`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const newTemplate: Template = apiResponse.data

      templates.value.set(newTemplate.id, newTemplate)
      console.log(`[TemplateStore] Successfully created template:`, newTemplate)
      return newTemplate
    } catch (e) {
      error.value = `Failed to create template: ${e}`
      console.error(`[TemplateStore] Error creating template:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Updates an existing template.
   */
  async function updateTemplate(id: ID, payload: UpdateTemplatePayload) {
    isLoading.value = true
    error.value = null
    console.log(`[TemplateStore] Attempting to update template ${id} with payload:`, payload)
    try {
      const response = await fetch(`${API_BASE_URL}/templates/${id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const updatedTemplate: Template = apiResponse.data

      templates.value.set(id, updatedTemplate)
      console.log(`[TemplateStore] Successfully updated template ${id}:`, updatedTemplate)
      return updatedTemplate
    } catch (e) {
      error.value = `Failed to update template ${id}: ${e}`
      console.error(`[TemplateStore] Error updating template ${id}:`, e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Deletes a template.
   */
  async function deleteTemplate(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/templates/${id}`, {
        method: 'DELETE',
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      templates.value.delete(id)
      console.log(`[TemplateStore] Successfully deleted template ${id}`)
    } catch (e) {
      error.value = `Failed to delete template ${id}: ${e}`
      console.error('[TemplateStore] Error deleting template:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Clones a template with a new name.
   */
  async function cloneTemplate(id: ID, newName: string) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/templates/${id}/clone`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          new_name: newName,
        }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const clonedTemplate: Template = apiResponse.data

      templates.value.set(clonedTemplate.id, clonedTemplate)
      console.log(`[TemplateStore] Successfully cloned template ${id} as "${newName}"`)
      return clonedTemplate
    } catch (e) {
      error.value = `Failed to clone template ${id}: ${e}`
      console.error('[TemplateStore] Error cloning template:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Creates a task from a template.
   */
  async function createTaskFromTemplate(id: ID, context: CreationContext) {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetch(`${API_BASE_URL}/templates/${id}/tasks`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          context,
        }),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`)
      }

      const apiResponse = await response.json()
      const newTask: Task = apiResponse.data

      console.log(`[TemplateStore] Successfully created task from template ${id}:`, newTask)
      return newTask
    } catch (e) {
      error.value = `Failed to create task from template ${id}: ${e}`
      console.error('[TemplateStore] Error creating task from template:', e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  return {
    // State
    templates,
    isLoading,
    error,
    // Getters
    allTemplates,
    getTemplatesForArea,
    getTemplateById,
    // Actions
    fetchTemplates,
    searchTemplates,
    fetchTemplatesForArea,
    fetchTemplatesWithVariable,
    fetchTemplate,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    cloneTemplate,
    createTaskFromTemplate,
  }
})

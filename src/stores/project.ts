import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Project, ProjectStatus } from '@/types/models'

type ID = string

export interface CreateProjectPayload {
  title: string
  description?: string | null
  icon?: string | null
  color?: string | null
  status?: ProjectStatus
  metadata?: Record<string, any> | null
}

export interface UpdateProjectPayload {
  title?: string
  description?: string | null
  icon?: string | null
  color?: string | null
  status?: ProjectStatus
  metadata?: Record<string, any> | null
}

export const useProjectStore = defineStore('project', () => {
  const projects = ref(new Map<ID, Project>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const allProjects = computed(() => {
    // Projects are typically sorted by creation time or a specific order key.
    // Here we sort by creation_at descending as a default.
    return Array.from(projects.value.values()).sort((a, b) => b.created_at - a.created_at)
  })

  function getProjectById(id: ID): Project | undefined {
    return projects.value.get(id)
  }

  async function fetchProjects() {
    isLoading.value = true
    error.value = null
    try {
      const projectList = await invoke<Project[]>('list_projects')
      const projectMap = new Map<ID, Project>()
      for (const project of projectList) {
        projectMap.set(project.id, project)
      }
      projects.value = projectMap
    } catch (e) {
      error.value = `Failed to fetch projects: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function createProject(payload: CreateProjectPayload) {
    isLoading.value = true
    error.value = null
    try {
      const newProject = await invoke<Project>('create_project', { payload })
      projects.value.set(newProject.id, newProject)
    } catch (e) {
      error.value = `Failed to create project: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function updateProject(id: ID, payload: UpdateProjectPayload) {
    isLoading.value = true
    error.value = null
    try {
      const updatedProject = await invoke<Project>('update_project', { id, payload })
      projects.value.set(id, updatedProject)
    } catch (e) {
      error.value = `Failed to update project ${id}: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  async function deleteProject(id: ID) {
    isLoading.value = true
    error.value = null
    try {
      await invoke('delete_project', { id })
      projects.value.delete(id)
    } catch (e) {
      error.value = `Failed to delete project ${id}: ${e}`
      console.error(e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    projects,
    isLoading,
    error,
    allProjects,
    getProjectById,
    fetchProjects,
    createProject,
    updateProject,
    deleteProject,
  }
})

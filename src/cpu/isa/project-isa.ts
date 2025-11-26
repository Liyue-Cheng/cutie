/**
 * Project 指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 统一的 commit 逻辑
 * 3. 完整的 CRUD 操作
 * 4. 支持 ProjectSection 管理
 */

import type { ISADefinition } from '@cutie/cpu-pipeline'
import type { ProjectCard, ProjectSection } from '@/types/dtos'
import { useProjectStore } from '@/stores/project'

export const ProjectISA: ISADefinition = {
  // ==================== Project 指令 ====================

  'project.fetch_all': {
    meta: {
      description: '获取所有 Projects',
      category: 'project',
      resourceIdentifier: () => ['projects:all'],
      priority: 5,
      timeout: 5000,
    },

    validate: async () => true,

    request: {
      method: 'GET',
      url: '/projects',
    },

    commit: async (result: ProjectCard[]) => {
      const projectStore = useProjectStore()
      projectStore.replaceAllProjects_mut(result)
    },
  },

  'project.get': {
    meta: {
      description: '获取单个 Project',
      category: 'project',
      resourceIdentifier: (payload) => [`project:${payload.id}`],
      priority: 5,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ Project ID 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'GET',
      url: (payload) => `/projects/${payload.id}`,
    },

    commit: async (result: ProjectCard) => {
      const projectStore = useProjectStore()
      projectStore.addOrUpdateProject_mut(result)
    },
  },

  'project.create': {
    meta: {
      description: '创建 Project',
      category: 'project',
      resourceIdentifier: () => [],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.name?.trim()) {
        console.warn('❌ Project 名称不能为空')
        return false
      }
      if (payload.name.length > 200) {
        console.warn('❌ Project 名称长度不能超过200字符')
        return false
      }
      if (payload.description && payload.description.length > 2000) {
        console.warn('❌ Project 描述长度不能超过2000字符')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: '/projects',
      body: (payload) => payload,
    },

    commit: async (result: ProjectCard) => {
      const projectStore = useProjectStore()
      projectStore.addOrUpdateProject_mut(result)
    },
  },

  'project.update': {
    meta: {
      description: '更新 Project',
      category: 'project',
      resourceIdentifier: (payload) => [`project:${payload.id}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ Project ID 不能为空')
        return false
      }
      if (payload.name !== undefined && !payload.name?.trim()) {
        console.warn('❌ Project 名称不能为空')
        return false
      }
      if (payload.name && payload.name.length > 200) {
        console.warn('❌ Project 名称长度不能超过200字符')
        return false
      }
      if (payload.description && payload.description.length > 2000) {
        console.warn('❌ Project 描述长度不能超过2000字符')
        return false
      }
      return true
    },

    request: {
      method: 'PATCH',
      url: (payload) => `/projects/${payload.id}`,
      body: (payload) => {
        const body: Record<string, any> = {}
        if (payload.name !== undefined) body.name = payload.name
        if (payload.description !== undefined) body.description = payload.description
        if (payload.status !== undefined) body.status = payload.status
        if (payload.due_date !== undefined) body.due_date = payload.due_date
        if (payload.area_id !== undefined) body.area_id = payload.area_id
        return body
      },
    },

    commit: async (result: ProjectCard) => {
      const projectStore = useProjectStore()
      projectStore.addOrUpdateProject_mut(result)
    },
  },

  'project.delete': {
    meta: {
      description: '删除 Project',
      category: 'project',
      resourceIdentifier: (payload) => [`project:${payload.id}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ Project ID 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'DELETE',
      url: (payload) => `/projects/${payload.id}`,
    },

    commit: async (_result, payload) => {
      const projectStore = useProjectStore()
      projectStore.removeProject_mut(payload.id)
    },
  },

  // ==================== ProjectSection 指令 ====================

  'project_section.fetch_all': {
    meta: {
      description: '获取项目的所有 Sections',
      category: 'project',
      resourceIdentifier: (payload) => [`project:${payload.project_id}:sections`],
      priority: 5,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.project_id?.trim()) {
        console.warn('❌ Project ID 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'GET',
      url: (payload) => `/projects/${payload.project_id}/sections`,
    },

    commit: async (result: ProjectSection[]) => {
      const projectStore = useProjectStore()
      projectStore.replaceProjectSections_mut(result)
    },
  },

  'project_section.create': {
    meta: {
      description: '创建 ProjectSection',
      category: 'project',
      resourceIdentifier: (payload) => [`project:${payload.project_id}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.project_id?.trim()) {
        console.warn('❌ Project ID 不能为空')
        return false
      }
      if (!payload.title?.trim()) {
        console.warn('❌ Section 标题不能为空')
        return false
      }
      if (payload.title.length > 200) {
        console.warn('❌ Section 标题长度不能超过200字符')
        return false
      }
      if (payload.description && payload.description.length > 2000) {
        console.warn('❌ Section 描述长度不能超过2000字符')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: (payload) => `/projects/${payload.project_id}/sections`,
      body: (payload) => ({
        title: payload.title,
        description: payload.description,
        sort_order: payload.sort_order,
      }),
    },

    commit: async (result: ProjectSection) => {
      const projectStore = useProjectStore()
      projectStore.addOrUpdateSection_mut(result)
    },
  },

  'project_section.update': {
    meta: {
      description: '更新 ProjectSection',
      category: 'project',
      resourceIdentifier: (payload) => [`project_section:${payload.id}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.project_id?.trim()) {
        console.warn('❌ Project ID 不能为空')
        return false
      }
      if (!payload.id?.trim()) {
        console.warn('❌ Section ID 不能为空')
        return false
      }
      if (payload.title !== undefined && !payload.title?.trim()) {
        console.warn('❌ Section 标题不能为空')
        return false
      }
      if (payload.title && payload.title.length > 200) {
        console.warn('❌ Section 标题长度不能超过200字符')
        return false
      }
      if (payload.description && payload.description.length > 2000) {
        console.warn('❌ Section 描述长度不能超过2000字符')
        return false
      }
      return true
    },

    request: {
      method: 'PATCH',
      url: (payload) => `/projects/${payload.project_id}/sections/${payload.id}`,
      body: (payload) => {
        const body: Record<string, any> = {}
        if (payload.title !== undefined) body.title = payload.title
        if (payload.description !== undefined) body.description = payload.description
        if (payload.sort_order !== undefined) body.sort_order = payload.sort_order
        return body
      },
    },

    commit: async (result: ProjectSection) => {
      const projectStore = useProjectStore()
      projectStore.addOrUpdateSection_mut(result)
    },
  },

  'project_section.delete': {
    meta: {
      description: '删除 ProjectSection',
      category: 'project',
      resourceIdentifier: (payload) => [`project_section:${payload.id}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.project_id?.trim()) {
        console.warn('❌ Project ID 不能为空')
        return false
      }
      if (!payload.id?.trim()) {
        console.warn('❌ Section ID 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'DELETE',
      url: (payload) => `/projects/${payload.project_id}/sections/${payload.id}`,
    },

    commit: async (_result, payload) => {
      const projectStore = useProjectStore()
      projectStore.removeSection_mut(payload.id)
    },
  },

  'project_section.reorder': {
    meta: {
      description: '重排序 ProjectSection',
      category: 'project',
      resourceIdentifier: (payload) => [`project_section:${payload.section_id}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.project_id?.trim()) {
        console.warn('❌ Project ID 不能为空')
        return false
      }
      if (!payload.section_id?.trim()) {
        console.warn('❌ Section ID 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: (payload) => `/projects/${payload.project_id}/sections/${payload.section_id}/reorder`,
      body: (payload) => ({
        prev_section_id: payload.prev_section_id ?? null,
        next_section_id: payload.next_section_id ?? null,
      }),
    },

    commit: async (result: ProjectSection) => {
      const projectStore = useProjectStore()
      projectStore.addOrUpdateSection_mut(result)
    },
  },
}

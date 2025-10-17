/**
 * 模板指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 自动处理 correlation-id
 * 3. 统一的 commit 逻辑
 */

import type { ISADefinition } from './types'
import type { Template, TaskCard } from '@/types/dtos'
import { useTemplateStore } from '@/stores/template'
import { useTaskStore } from '@/stores/task'

export const TemplateISA: ISADefinition = {
  'template.create': {
    meta: {
      description: '创建模板',
      category: 'system',
      resourceIdentifier: () => [],
      priority: 5,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.title?.trim()) {
        console.warn('❌ 模板标题不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: '/templates',
      body: (payload) => payload,
    },

    commit: async (result: Template) => {
      const templateStore = useTemplateStore()
      templateStore.addOrUpdateTemplate_mut(result)
    },
  },

  'template.update': {
    meta: {
      description: '更新模板',
      category: 'system',
      resourceIdentifier: (payload) => [`template:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      const templateStore = useTemplateStore()
      const template = templateStore.getTemplateById(payload.id)
      if (!template) {
        console.warn('❌ 模板不存在:', payload.id)
        return false
      }
      return true
    },

    // 🔥 声明式请求配置（动态 URL）
    request: {
      method: 'PATCH',
      url: (payload) => `/templates/${payload.id}`,
      body: (payload) => {
        const { id, ...updates } = payload
        return updates
      },
    },

    commit: async (result: Template) => {
      const templateStore = useTemplateStore()
      templateStore.addOrUpdateTemplate_mut(result)
    },
  },

  'template.delete': {
    meta: {
      description: '删除模板',
      category: 'system',
      resourceIdentifier: (payload) => [`template:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      const templateStore = useTemplateStore()
      const template = templateStore.getTemplateById(payload.id)
      if (!template) {
        console.warn('❌ 模板不存在:', payload.id)
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'DELETE',
      url: (payload) => `/templates/${payload.id}`,
    },

    commit: async (_result, payload) => {
      const templateStore = useTemplateStore()
      templateStore.removeTemplate_mut(payload.id)
    },
  },

  'template.create_task': {
    meta: {
      description: '从模板创建任务',
      category: 'task',
      resourceIdentifier: (payload) => [`template:${payload.template_id}`],
      priority: 5,
      timeout: 10000,
    },

    validate: async (payload) => {
      const templateStore = useTemplateStore()
      const template = templateStore.getTemplateById(payload.template_id)
      if (!template) {
        console.warn('❌ 模板不存在:', payload.template_id)
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: (payload) => `/templates/${payload.template_id}/create-task`,
      body: (payload) => payload.variables || {},
    },

    commit: async (result: TaskCard) => {
      const taskStore = useTaskStore()
      taskStore.addOrUpdateTask_mut(result)
    },
  },

  'template.from_task': {
    meta: {
      description: '从任务创建模板',
      category: 'system',
      resourceIdentifier: (payload) => [`task:${payload.task_id}`],
      priority: 5,
      timeout: 10000,
    },

    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.task_id)
      if (!task) {
        console.warn('❌ 任务不存在:', payload.task_id)
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: (payload) => `/tasks/${payload.task_id}/to-template`,
      body: (payload) => ({
        title: payload.title,
        category: payload.category,
      }),
    },

    commit: async (result: Template) => {
      const templateStore = useTemplateStore()
      templateStore.addOrUpdateTemplate_mut(result)
    },
  },
}

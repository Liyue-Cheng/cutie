/**
 * 模板指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 自动处理 correlation-id
 * 3. 统一的 commit 逻辑
 */

import type { ISADefinition } from 'front-cpu'
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
      description: '从模板创建任务（支持原子操作：创建+日程+排序）',
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
    // 支持可选参数：scheduled_day, sort_position
    request: {
      method: 'POST',
      url: (payload) => `/templates/${payload.template_id}/create-task`,
      body: (payload) => ({
        variables: payload.variables || {},
        scheduled_day: payload.scheduled_day,
        sort_position: payload.sort_position,
      }),
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

  'template.update_sort_rank': {
    meta: {
      description: '更新模板排序位置',
      category: 'template',
      resourceIdentifier: (payload) => [`template:${payload.template_id}`],
      priority: 6,
      timeout: 5000,
    },
    validate: async (payload) => {
      const templateStore = useTemplateStore()
      return Boolean(templateStore.getTemplateById(payload.template_id))
    },
    request: {
      method: 'PATCH',
      url: (payload) => `/templates/${payload.template_id}/sort-rank`,
      body: (payload) => ({
        prev_template_id: payload.prev_template_id ?? null,
        next_template_id: payload.next_template_id ?? null,
      }),
    },
    commit: async (result: { template_id: string; new_rank: string }) => {
      const templateStore = useTemplateStore()
      const template = templateStore.getTemplateById(result.template_id)
      if (!template) return
      templateStore.addOrUpdateTemplate_mut({
        ...template,
        sort_rank: result.new_rank,
      })
    },
  },

  'template.batch_init_ranks': {
    meta: {
      description: '批量初始化模板排序',
      category: 'template',
      resourceIdentifier: (payload) =>
        (payload.template_ids || []).map((id: string) => `template:${id}`),
      priority: 4,
      timeout: 10000,
    },
    request: {
      method: 'POST',
      url: '/templates/batch-init-ranks',
      body: (payload) => ({
        template_ids: payload.template_ids,
      }),
    },
    commit: async (result: { assigned: Array<{ template_id: string; new_rank: string }> }) => {
      const templateStore = useTemplateStore()
      result.assigned.forEach(({ template_id, new_rank }) => {
        const template = templateStore.getTemplateById(template_id)
        if (!template) return
        templateStore.addOrUpdateTemplate_mut({
          ...template,
          sort_rank: new_rank,
        })
      })
    },
  },
}

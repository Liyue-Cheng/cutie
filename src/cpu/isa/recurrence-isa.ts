/**
 * 循环规则指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 自动处理 correlation-id
 * 3. 统一的 commit 逻辑
 */

import type { ISADefinition } from '@cutie/cpu-pipeline'
import type { TaskRecurrence } from '@/types/dtos'
import { useRecurrenceStore } from '@/stores/recurrence'
import * as recurrenceCore from '@/stores/recurrence/core'

export const RecurrenceISA: ISADefinition = {
  'recurrence.create': {
    meta: {
      description: '创建循环规则',
      category: 'system',
      resourceIdentifier: () => [],
      priority: 5,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.template_id?.trim()) {
        console.warn('❌ 模板ID不能为空')
        return false
      }
      if (!payload.rule?.trim()) {
        console.warn('❌ 循环规则不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: '/recurrences',
      body: (payload) => payload,
    },

    commit: async (result: TaskRecurrence) => {
      recurrenceCore.addOrUpdateRecurrence(result)
    },
  },

  'recurrence.update': {
    meta: {
      description: '更新循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      const recurrenceStore = useRecurrenceStore()
      const recurrence = recurrenceStore.getRecurrenceById(payload.id)
      if (!recurrence) {
        console.warn('❌ 循环规则不存在:', payload.id)
        return false
      }
      return true
    },

    // 🔥 声明式请求配置（动态 URL）
    request: {
      method: 'PATCH',
      url: (payload) => `/recurrences/${payload.id}`,
      body: (payload) => {
        const { id, ...updates } = payload
        return updates
      },
    },

    commit: async (result: TaskRecurrence) => {
      recurrenceCore.addOrUpdateRecurrence(result)
    },
  },

  'recurrence.delete': {
    meta: {
      description: '删除循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`recurrence:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      const recurrenceStore = useRecurrenceStore()
      const recurrence = recurrenceStore.getRecurrenceById(payload.id)
      if (!recurrence) {
        console.warn('❌ 循环规则不存在:', payload.id)
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'DELETE',
      url: (payload) => `/recurrences/${payload.id}`,
    },

    commit: async (_result, payload) => {
      recurrenceCore.removeRecurrence(payload.id)
    },
  },

  'recurrence.fetch_all': {
    meta: {
      description: '获取所有循环规则',
      category: 'system',
      resourceIdentifier: () => [],
      priority: 3,
      timeout: 10000,
    },

    // 🔥 声明式请求配置
    request: {
      method: 'GET',
      url: '/recurrences',
    },

    commit: async (result: TaskRecurrence[]) => {
      recurrenceCore.clearAll()
      result.forEach((recurrence) => {
        recurrenceCore.addOrUpdateRecurrence(recurrence)
      })
    },
  },

  'recurrence.fetch_by_template': {
    meta: {
      description: '按模板ID获取循环规则',
      category: 'system',
      resourceIdentifier: (payload) => [`template:${payload.template_id}`],
      priority: 3,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.template_id?.trim()) {
        console.warn('❌ 模板ID不能为空')
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'GET',
      url: (payload) => `/recurrences?template_id=${payload.template_id}`,
    },

    commit: async (result: TaskRecurrence[]) => {
      // 不清空全部，只更新相关的
      result.forEach((recurrence) => {
        recurrenceCore.addOrUpdateRecurrence(recurrence)
      })
    },
  },

  'recurrence.update_template_and_instances': {
    meta: {
      description: '批量更新模板和所有未完成实例',
      category: 'system',
      resourceIdentifier: (payload) => [`recurrence:${payload.recurrence_id}`],
      priority: 7,
      timeout: 30000, // 批量操作可能耗时较长
    },

    validate: async (payload) => {
      const recurrenceStore = useRecurrenceStore()
      const recurrence = recurrenceStore.getRecurrenceById(payload.recurrence_id)
      if (!recurrence) {
        console.warn('❌ 循环规则不存在:', payload.recurrence_id)
        return false
      }
      return true
    },

    // 🔥 声明式请求配置
    request: {
      method: 'PATCH',
      url: (payload) => `/recurrences/${payload.recurrence_id}/template-and-instances`,
      body: (payload) => {
        const { recurrence_id, ...updates } = payload
        return updates
      },
    },

    commit: async (result) => {
      // 批量操作的结果通常包含统计信息，但不需要更新本地store
      // 因为具体的任务更新会通过SSE事件处理
      console.info('✅ 模板和实例批量更新完成:', result)
    },
  },
}

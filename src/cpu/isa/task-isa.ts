/**
 * 任务指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 自动处理 correlation-id
 * 3. 统一的 commit 逻辑
 */

import type { ISADefinition } from './types'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
import {
  transactionProcessor,
  type TaskTransactionResult,
} from '@/infra/transaction/transactionProcessor'

export const TaskISA: ISADefinition = {
  'task.create': {
    meta: {
      description: '创建任务',
      category: 'task',
      resourceIdentifier: () => [], // 创建操作无固定资源
      priority: 5,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      if (!payload.title?.trim()) {
        console.warn('❌ 任务标题不能为空')
        return false
      }
      return true
    },
    
    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: '/tasks',
    },
    
    commit: async (result: TaskCard) => {
      const taskStore = useTaskStore()
      taskStore.addOrUpdateTask_mut(result)
    },
  },

  'task.create_with_schedule': {
    meta: {
      description: '创建任务并添加日程',
      category: 'task',
      resourceIdentifier: () => [],
      priority: 5,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      if (!payload.title?.trim()) {
        console.warn('❌ 任务标题不能为空')
        return false
      }
      if (!payload.scheduled_day) {
        console.warn('❌ 日程日期不能为空')
        return false
      }
      return true
    },
    
    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: '/tasks/with-schedule',
    },
    
    commit: async (result: TaskCard) => {
      const taskStore = useTaskStore()
      taskStore.addOrUpdateTask_mut(result)
    },
  },

  'task.update': {
    meta: {
      description: '更新任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)
      if (!task) {
        console.warn('❌ 任务不存在:', payload.id)
        return false
      }
      return true
    },
    
    // 🔥 声明式请求配置（动态 URL）
    request: {
      method: 'PATCH',
      url: (payload) => `/tasks/${payload.id}`,
      body: (payload) => payload.updates, // 只发送 updates 部分
    },
    
    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.complete': {
    meta: {
      description: '完成任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 7,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)

      if (!task) {
        console.warn('❌ 任务不存在:', payload.id)
        return false
      }

      if (task.is_completed) {
        console.warn('⚠️ 任务已完成:', payload.id)
        return false
      }

      return true
    },
    
    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: (payload) => `/tasks/${payload.id}/completion`,
      body: () => ({}), // 空 body
    },
    
    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.reopen': {
    meta: {
      description: '重新打开任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 7,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)

      if (!task) {
        console.warn('❌ 任务不存在:', payload.id)
        return false
      }

      if (!task.is_completed) {
        console.warn('⚠️ 任务未完成，无需重新打开:', payload.id)
        return false
      }

      return true
    },
    
    // 🔥 声明式请求配置
    request: {
      method: 'DELETE',
      url: (payload) => `/tasks/${payload.id}/completion`,
    },
    
    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.delete': {
    meta: {
      description: '删除任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 5,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)

      if (!task) {
        console.warn('❌ 任务不存在:', payload.id)
        return false
      }

      return true
    },
    
    // 🔥 声明式请求配置
    request: {
      method: 'DELETE',
      url: (payload) => `/tasks/${payload.id}`,
    },
    
    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.archive': {
    meta: {
      description: '归档任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)

      if (!task) {
        console.warn('❌ 任务不存在:', payload.id)
        return false
      }

      if (task.is_archived) {
        console.warn('⚠️ 任务已归档:', payload.id)
        return false
      }

      return true
    },
    
    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: (payload) => `/tasks/${payload.id}/archive`,
      body: () => ({}), // 空 body
    },
    
    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.unarchive': {
    meta: {
      description: '取消归档任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)

      if (!task) {
        console.warn('❌ 任务不存在:', payload.id)
        return false
      }

      if (!task.is_archived) {
        console.warn('⚠️ 任务未归档，无需取消归档:', payload.id)
        return false
      }

      return true
    },
    
    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: (payload) => `/tasks/${payload.id}/unarchive`,
      body: () => ({}), // 空 body
    },
    
    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'task.return_to_staging': {
    meta: {
      description: '返回暂存区',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    
    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)

      if (!task) {
        console.warn('❌ 任务不存在:', payload.id)
        return false
      }

      return true
    },
    
    // 🔥 声明式请求配置
    request: {
      method: 'POST',
      url: (payload) => `/tasks/${payload.id}/return-to-staging`,
      body: () => ({}), // 空 body
    },
    
    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}

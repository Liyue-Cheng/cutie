/**
 * 任务指令集
 *
 * 将任务相关的 CommandBus handlers 迁移到 CPU Pipeline
 */

import type { ISADefinition } from './types'
import type { TaskCard } from '@/types/dtos'
import { apiPost, apiDelete, apiPatch } from '@/stores/shared'
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
    execute: async (payload, context) => {
      return await apiPost('/tasks', payload, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
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
    execute: async (payload, context) => {
      return await apiPost('/tasks/with-schedule', payload, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
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
    execute: async (payload, context) => {
      return await apiPatch(`/tasks/${payload.id}`, payload.updates, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
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
    execute: async (payload, context) => {
      return await apiPost(
        `/tasks/${payload.id}/completion`,
        {},
        {
          headers: { 'X-Correlation-ID': context.correlationId },
        }
      )
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
    execute: async (payload, context) => {
      return await apiDelete(`/tasks/${payload.id}/completion`, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
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
    execute: async (payload, context) => {
      return await apiDelete(`/tasks/${payload.id}`, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })
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
    execute: async (payload, context) => {
      return await apiPost(
        `/tasks/${payload.id}/archive`,
        {},
        {
          headers: { 'X-Correlation-ID': context.correlationId },
        }
      )
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
    execute: async (payload, context) => {
      return await apiPost(
        `/tasks/${payload.id}/unarchive`,
        {},
        {
          headers: { 'X-Correlation-ID': context.correlationId },
        }
      )
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
    execute: async (payload, context) => {
      return await apiPost(
        `/tasks/${payload.id}/return-to-staging`,
        {},
        {
          headers: { 'X-Correlation-ID': context.correlationId },
        }
      )
    },
    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}

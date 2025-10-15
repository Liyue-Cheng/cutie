/**
 * 日程指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 自动处理 correlation-id
 * 3. 统一的 commit 逻辑
 */

import type { ISADefinition } from './types'
import {
  transactionProcessor,
  type TaskTransactionResult,
} from '@/infra/transaction/transactionProcessor'
import { useTaskStore } from '@/stores/task'

export const ScheduleISA: ISADefinition = {
  'schedule.create': {
    meta: {
      description: '创建日程',
      category: 'schedule',
      resourceIdentifier: (payload) => [`task:${payload.task_id}`],
      priority: 6,
      timeout: 10000,
    },

    // 🔥 声明式请求配置（动态 URL）
    request: {
      method: 'POST',
      url: (payload) => `/tasks/${payload.task_id}/schedules`,
      body: (payload) => ({ scheduled_day: payload.scheduled_day }),
    },

    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'schedule.update': {
    meta: {
      description: '更新日程',
      category: 'schedule',
      resourceIdentifier: (payload) => [
        `task:${payload.task_id}`,
        `schedule:${payload.task_id}:${payload.scheduled_day}`,
      ],
      priority: 6,
      timeout: 10000,
    },

    // 🔥 乐观更新配置
    optimistic: {
      enabled: true,
      apply: (payload) => {
        const taskStore = useTaskStore()
        const task = taskStore.getTaskById_Mux(payload.task_id)

        if (!task || !task.schedules) {
          return { task_id: payload.task_id, had_task: false }
        }

        // 保存原始 schedules 数组（用于回滚）
        const snapshot = {
          task_id: payload.task_id,
          had_task: true,
          original_schedules: JSON.parse(JSON.stringify(task.schedules)), // 深拷贝
        }

        // 🔥 立即更新 schedules 数组
        // 找到对应日期的 schedule 并修改其 scheduled_day
        const newSchedules = task.schedules.map((schedule) => {
          if (schedule.scheduled_day === payload.scheduled_day) {
            return {
              ...schedule,
              scheduled_day: payload.updates.new_date,
            }
          }
          return schedule
        })

        // 立即更新任务
        taskStore.addOrUpdateTask_mut({
          ...task,
          schedules: newSchedules,
        })

        return snapshot
      },
      rollback: (snapshot) => {
        if (!snapshot.had_task) return

        const taskStore = useTaskStore()
        const task = taskStore.getTaskById_Mux(snapshot.task_id)

        if (task) {
          // 🔥 恢复原始 schedules 数组
          taskStore.addOrUpdateTask_mut({
            ...task,
            schedules: snapshot.original_schedules,
          })
        }
      },
    },

    // 🔥 声明式请求配置（动态 URL + body 映射）
    request: {
      method: 'PATCH',
      url: (payload) => `/tasks/${payload.task_id}/schedules/${payload.scheduled_day}`,
      body: (payload) => payload.updates,
    },

    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'schedule.delete': {
    meta: {
      description: '删除日程',
      category: 'schedule',
      resourceIdentifier: (payload) => [
        `task:${payload.task_id}`,
        `schedule:${payload.task_id}:${payload.scheduled_day}`,
      ],
      priority: 5,
      timeout: 10000,
    },

    // 🔥 声明式请求配置（动态 URL）
    request: {
      method: 'DELETE',
      url: (payload) => `/tasks/${payload.task_id}/schedules/${payload.scheduled_day}`,
    },

    commit: async (result: TaskTransactionResult, _payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}

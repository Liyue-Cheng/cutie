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

    // 🔥 乐观更新配置
    optimistic: {
      enabled: true,
      apply: (payload) => {
        const taskStore = useTaskStore()
        const task = taskStore.getTaskById_Mux(payload.task_id)

        if (!task) {
          return { task_id: payload.task_id, had_task: false }
        }

        // 保存原始状态（用于回滚）
        const snapshot = {
          task_id: payload.task_id,
          had_task: true,
          original_schedules: task.schedules ? JSON.parse(JSON.stringify(task.schedules)) : null,
          original_schedule_status: task.schedule_status,
        }

        // 🔥 立即添加新日程到 schedules 数组
        const newSchedule = {
          scheduled_day: payload.scheduled_day,
          outcome: 'planned' as const, // 新创建的日程默认为 planned
          time_blocks: [], // 暂时为空
        }

        const newSchedules = task.schedules ? [...task.schedules, newSchedule] : [newSchedule]

        // 立即更新任务：添加日程并设为 scheduled 状态
        taskStore.addOrUpdateTask_mut({
          ...task,
          schedules: newSchedules,
          schedule_status: 'scheduled' as const,
        })

        return snapshot
      },
      rollback: (snapshot) => {
        if (!snapshot.had_task) return

        const taskStore = useTaskStore()
        const task = taskStore.getTaskById_Mux(snapshot.task_id)

        if (task) {
          // 🔥 恢复原始状态
          taskStore.addOrUpdateTask_mut({
            ...task,
            schedules: snapshot.original_schedules,
            schedule_status: snapshot.original_schedule_status,
          })
        }
      },
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
        // 找到对应日期的 schedule 并修改对应字段
        const newSchedules = task.schedules.map((schedule) => {
          if (schedule.scheduled_day === payload.scheduled_day) {
            const updatedSchedule = { ...schedule }

            // 🔥 只有提供了 new_date 才更新 scheduled_day（改期操作）
            if (payload.updates.new_date !== undefined) {
              const isCrossDateUpdate = payload.updates.new_date !== payload.scheduled_day
              updatedSchedule.scheduled_day = payload.updates.new_date
              // 🔥 跨日期改期时，清空时间片（因为原日期的时间片会被后端删除）
              if (isCrossDateUpdate) {
                updatedSchedule.time_blocks = []
              }
            }

            // 🔥 只有提供了 outcome 才更新 outcome（状态切换操作）
            if (payload.updates.outcome !== undefined) {
              updatedSchedule.outcome = payload.updates.outcome.toLowerCase()
            }

            return updatedSchedule
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

    // 🔥 乐观更新配置
    optimistic: {
      enabled: true,
      apply: (payload) => {
        const taskStore = useTaskStore()
        const task = taskStore.getTaskById_Mux(payload.task_id)

        if (!task || !task.schedules) {
          return { task_id: payload.task_id, had_task: false }
        }

        // 保存原始状态（用于回滚）
        const snapshot = {
          task_id: payload.task_id,
          had_task: true,
          original_schedules: JSON.parse(JSON.stringify(task.schedules)),
          original_schedule_status: task.schedule_status,
        }

        // 🔥 立即删除指定日期的日程
        const newSchedules = task.schedules.filter(
          (schedule) => schedule.scheduled_day !== payload.scheduled_day
        )

        // 🔥 智能更新 schedule_status
        // 如果删除后没有未来日程，设为 staging；否则保持 scheduled
        const today = new Date().toISOString().split('T')[0]
        const hasFutureSchedule = newSchedules.some(
          (schedule) => schedule.scheduled_day >= today
        )

        // 立即更新任务
        taskStore.addOrUpdateTask_mut({
          ...task,
          schedules: newSchedules.length > 0 ? newSchedules : null,
          schedule_status: hasFutureSchedule ? 'scheduled' : 'staging',
        })

        return snapshot
      },
      rollback: (snapshot) => {
        if (!snapshot.had_task) return

        const taskStore = useTaskStore()
        const task = taskStore.getTaskById_Mux(snapshot.task_id)

        if (task) {
          // 🔥 恢复原始状态
          taskStore.addOrUpdateTask_mut({
            ...task,
            schedules: snapshot.original_schedules,
            schedule_status: snapshot.original_schedule_status,
          })
        }
      },
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

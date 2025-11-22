/**
 * 任务指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 自动处理 correlation-id
 * 3. 统一的 commit 逻辑
 */

import type { ISADefinition } from '@cutie/cpu-pipeline'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
import {
  transactionProcessor,
  type TaskTransactionResult,
} from '@/infra/transaction/transactionProcessor'

interface UpdateSortPositionResponse {
  task_id: string
  view_context: string
  new_rank: string
  updated_at: string
}

interface BatchInitRanksResponse {
  view_context: string
  assigned: Array<{
    task_id: string
    new_rank: string
  }>
  updated_at: string
}

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
      body: (payload) => ({
        // ✅ 客户端时间已通过 X-Client-Time 请求头统一发送
        view_context: payload.view_context || 'misc::staging', // 视图上下文
      }),
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

    // 🔥 乐观更新配置
    optimistic: {
      enabled: true,
      apply: (payload) => {
        const taskStore = useTaskStore()
        const task = taskStore.getTaskById_Mux(payload.id)

        if (!task) {
          return { task_id: payload.id, had_task: false }
        }

        // 保存原始状态（用于回滚）
        const snapshot = {
          task_id: payload.id,
          had_task: true,
          original_schedules: task.schedules ? JSON.parse(JSON.stringify(task.schedules)) : null,
          original_is_completed: task.is_completed,
          original_completed_at: task.completed_at,
        }

        // 🔥 立即清除所有当前和未来的日程
        // 返回暂存区操作会删除所有 >= today 的日程，只保留过去的
        const today = new Date().toISOString().split('T')[0]
        const pastSchedules =
          task.schedules?.filter((schedule) => schedule.scheduled_day < today) || []

        // 🔥 立即更新任务状态
        // - 清除当前和未来日程（schedule_status 由 store 实时计算）
        // - 如果已完成，重新打开
        taskStore.addOrUpdateTask_mut({
          ...task,
          schedules: pastSchedules.length > 0 ? pastSchedules : null,
          is_completed: false, // 后端会自动重新打开
          completed_at: null,
        })

        return snapshot
      },
      rollback: (snapshot) => {
        if (!snapshot.had_task) return

        const taskStore = useTaskStore()
        const task = taskStore.getTaskById_Mux(snapshot.task_id)

        if (task) {
          // 🔥 恢复原始状态（schedule_status 由 store 实时计算）
          taskStore.addOrUpdateTask_mut({
            ...task,
            schedules: snapshot.original_schedules,
            is_completed: snapshot.original_is_completed,
            completed_at: snapshot.original_completed_at,
          })
        }
      },
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

  'task.update_sort_position': {
    meta: {
      description: '更新任务在视图中的排序位置',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.task_id}`],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload?.task_id || !payload?.view_context) {
        console.warn('❌ 缺少必要参数 task_id 或 view_context')
        return false
      }
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.task_id)
      if (!task) {
        console.warn('❌ 任务不存在:', payload.task_id)
        return false
      }
      return true
    },

    request: {
      method: 'PATCH',
      url: (payload) => `/tasks/${payload.task_id}/sort-position`,
      body: (payload) => ({
        view_context: payload.view_context,
        prev_task_id: payload.prev_task_id ?? null,
        next_task_id: payload.next_task_id ?? null,
      }),
    },

    commit: async (result: UpdateSortPositionResponse) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(result.task_id)
      if (!task) {
        return
      }
      const updatedSortPositions = {
        ...(task.sort_positions ?? {}),
        [result.view_context]: result.new_rank,
      }
      taskStore.addOrUpdateTask_mut({
        ...task,
        sort_positions: updatedSortPositions,
      } as TaskCard)
    },
  },

  'task.batch_init_ranks': {
    meta: {
      description: '批量初始化任务排序位置',
      category: 'task',
      resourceIdentifier: (payload) => payload.task_ids?.map((id: string) => `task:${id}`) ?? [],
      priority: 4,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (
        !payload?.view_context ||
        !Array.isArray(payload.task_ids) ||
        payload.task_ids.length === 0
      ) {
        console.warn('❌ 批量初始化排序缺少参数或任务列表为空')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: () => `/tasks/batch-init-ranks`,
      body: (payload) => payload,
    },

    commit: async (result: BatchInitRanksResponse) => {
      const taskStore = useTaskStore()
      for (const assignment of result.assigned) {
        const task = taskStore.getTaskById_Mux(assignment.task_id)
        if (!task) {
          continue
        }
        const updatedSortPositions = {
          ...(task.sort_positions ?? {}),
          [result.view_context]: assignment.new_rank,
        }
        taskStore.addOrUpdateTask_mut({
          ...task,
          sort_positions: updatedSortPositions,
        } as TaskCard)
      }
    },
  },
}

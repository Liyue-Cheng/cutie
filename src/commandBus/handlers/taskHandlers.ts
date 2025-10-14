/**
 * 任务命令处理器（重构版 v2.0）
 *
 * 职责：
 * - 接收任务相关命令
 * - ✅ 直接调用 API
 * - ✅ 调用 Store 的 mutation 方法更新状态
 * - ✅ 编排业务逻辑
 * - 统一的错误处理和日志
 *
 * 架构：
 * Handler → API Client → Store.mutation (纯数据操作)
 *
 * 设计原则：
 * - Handler 负责业务逻辑编排
 * - API Client 负责网络请求
 * - Store 只负责数据存储（不调用API）
 */

import { useTaskStore } from '@/stores/task'
import { apiPost, apiPatch, apiDelete } from '@/stores/shared'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import type { CommandHandlerMap } from '../types'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'
import { generateCorrelationId } from '@/infra/correlation/correlationId'
import type { TaskTransactionResult } from '@/infra/transaction/transactionProcessor'

/**
 * 创建任务
 */
const handleCreateTask: CommandHandlerMap['task.create'] = async (payload) => {
  // 1. 调用 API
  const task: TaskCard = await apiPost('/tasks', payload)

  // 2. 更新 Store
  const taskStore = useTaskStore()
  taskStore.addOrUpdateTask_mut(task)
}

/**
 * 创建任务并添加日程
 */
const handleCreateTaskWithSchedule: CommandHandlerMap['task.create_with_schedule'] = async (
  payload
) => {
  // 1. 调用 API
  const task: TaskCard = await apiPost('/tasks/with-schedule', payload)

  // 2. 更新 Store
  const taskStore = useTaskStore()
  taskStore.addOrUpdateTask_mut(task)
}

/**
 * 更新任务
 */
const handleUpdateTask: CommandHandlerMap['task.update'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiPatch(`/tasks/${payload.id}`, payload.updates, {
    headers: { 'X-Correlation-ID': correlationId },
  })

  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 完成任务
 */
const handleCompleteTask: CommandHandlerMap['task.complete'] = async (payload) => {
  // 1. 生成 correlation ID
  const correlationId = generateCorrelationId()

  // 2. 调用 API（带 correlation ID）
  const result: TaskTransactionResult = await apiPost(
    `/tasks/${payload.id}/completion`,
    {},
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  // 3. 使用 transactionProcessor 处理结果（自动去重、应用副作用）
  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 重新打开任务
 */
const handleReopenTask: CommandHandlerMap['task.reopen'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiDelete(
    `/tasks/${payload.id}/completion`,
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 删除任务
 */
const handleDeleteTask: CommandHandlerMap['task.delete'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiDelete(`/tasks/${payload.id}`, {
    headers: { 'X-Correlation-ID': correlationId },
  })

  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 归档任务
 */
const handleArchiveTask: CommandHandlerMap['task.archive'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiPost(
    `/tasks/${payload.id}/archive`,
    {},
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 取消归档任务
 */
const handleUnarchiveTask: CommandHandlerMap['task.unarchive'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiPost(
    `/tasks/${payload.id}/unarchive`,
    {},
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 返回暂存区
 */
const handleReturnToStaging: CommandHandlerMap['task.return_to_staging'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiPost(
    `/tasks/${payload.id}/return-to-staging`,
    {},
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 导出所有任务处理器
 */
export const taskHandlers: Partial<CommandHandlerMap> = {
  'task.create': handleCreateTask,
  'task.create_with_schedule': handleCreateTaskWithSchedule,
  'task.update': handleUpdateTask,
  'task.complete': handleCompleteTask,
  'task.reopen': handleReopenTask,
  'task.delete': handleDeleteTask,
  'task.archive': handleArchiveTask,
  'task.unarchive': handleUnarchiveTask,
  'task.return_to_staging': handleReturnToStaging,
}

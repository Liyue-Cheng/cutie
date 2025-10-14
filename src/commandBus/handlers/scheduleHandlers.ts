/**
 * 日程命令处理器（重构版 v3.0）
 *
 * 职责：
 * - 接收日程相关命令
 * - 直接调用 API
 * - 使用 transactionProcessor 处理统一的事务结果
 */

import { logger, LogTags } from '@/infra/logging/logger'
import type { CommandHandlerMap } from '../types'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'
import { generateCorrelationId } from '@/infra/correlation/correlationId'
import type { TaskTransactionResult } from '@/infra/transaction/transactionProcessor'
import { apiPost, apiPatch, apiDelete } from '@/stores/shared'

/**
 * 创建日程
 */
const handleCreateSchedule: CommandHandlerMap['schedule.create'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiPost(
    `/tasks/${payload.task_id}/schedules`,
    { scheduled_day: payload.scheduled_day },
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
 * 更新日程
 */
const handleUpdateSchedule: CommandHandlerMap['schedule.update'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiPatch(
    `/tasks/${payload.task_id}/schedules/${payload.scheduled_day}`,
    payload.updates,
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
 * 删除日程
 */
const handleDeleteSchedule: CommandHandlerMap['schedule.delete'] = async (payload) => {
  const correlationId = generateCorrelationId()

  const result: TaskTransactionResult = await apiDelete(
    `/tasks/${payload.task_id}/schedules/${payload.scheduled_day}`,
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
 * 导出所有日程处理器
 */
export const scheduleHandlers: Partial<CommandHandlerMap> = {
  'schedule.create': handleCreateSchedule,
  'schedule.update': handleUpdateSchedule,
  'schedule.delete': handleDeleteSchedule,
}

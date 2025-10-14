/**
 * 时间块命令处理器（重构版 v2.0）
 *
 * 职责：
 * - 接收时间块相关命令
 * - ✅ 直接调用 API
 * - ✅ 使用 transactionProcessor 统一处理响应和副作用
 * - ✅ 编排业务逻辑
 * - 统一的错误处理和日志
 *
 * 架构：
 * Handler → API Client → transactionProcessor → Store.mutations
 *
 * 设计原则：
 * - Handler 负责业务逻辑编排
 * - API Client 负责网络请求
 * - transactionProcessor 负责统一的副作用处理
 * - Store 只负责数据存储（不调用API）
 */

import { apiPost, apiPatch, apiDelete } from '@/stores/shared'
import { logger, LogTags } from '@/infra/logging/logger'
import type { CommandHandlerMap } from '../types'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'
import { generateCorrelationId } from '@/infra/correlation/correlationId'
import type {
  TimeBlockTransactionResult,
  DeleteTimeBlockResponse
} from '@/infra/transaction/transactionProcessor'

/**
 * 创建时间块
 */
const handleCreateTimeBlock: CommandHandlerMap['time_block.create'] = async (payload) => {
  // 1. 生成 correlation ID
  const correlationId = generateCorrelationId()

  // 2. 调用 API（带 correlation ID）
  const result: TimeBlockTransactionResult = await apiPost(
    '/time-blocks/from-task',
    {
      task_id: payload.task_id,
      start_time: payload.start_time,
      end_time: payload.end_time
    },
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  // 3. 使用 transactionProcessor 处理结果（自动去重、应用副作用）
  await transactionProcessor.applyTimeBlockTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 更新时间块
 */
const handleUpdateTimeBlock: CommandHandlerMap['time_block.update'] = async (payload) => {
  // 1. 生成 correlation ID
  const correlationId = generateCorrelationId()

  // 2. 调用 API（带 correlation ID）
  const result: TimeBlockTransactionResult = await apiPatch(
    `/time-blocks/${payload.id}`,
    {
      start_time: payload.updates.start_time,
      end_time: payload.updates.end_time,
    },
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  // 3. 使用 transactionProcessor 处理结果（自动去重、应用副作用）
  await transactionProcessor.applyTimeBlockTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 删除时间块
 */
const handleDeleteTimeBlock: CommandHandlerMap['time_block.delete'] = async (payload) => {
  // 1. 生成 correlation ID
  const correlationId = generateCorrelationId()

  // 2. 调用 API（带 correlation ID）
  const response: DeleteTimeBlockResponse = await apiDelete(
    `/time-blocks/${payload.id}`,
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )

  // 3. 使用 transactionProcessor 处理删除响应（自动去重、应用副作用）
  await transactionProcessor.applyDeleteTimeBlock(response, {
    correlation_id: correlationId,
    source: 'http',
  })
}

/**
 * 导出所有时间块处理器
 */
export const timeBlockHandlers: Partial<CommandHandlerMap> = {
  'time_block.create': handleCreateTimeBlock,
  'time_block.update': handleUpdateTimeBlock,
  'time_block.delete': handleDeleteTimeBlock,
}

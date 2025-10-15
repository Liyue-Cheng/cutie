/**
 * TimeBlock ISA - 时间块指令集
 *
 * 包含指令：
 * - timeblock.create_from_task: 从任务创建时间块
 * - timeblock.create: 创建空时间块
 * - timeblock.update: 更新时间块
 * - timeblock.delete: 删除时间块
 *
 * 特点：
 * - 无乐观更新（时间块操作不需要即时反馈）
 * - 使用 transactionProcessor 统一处理结果
 */

import {
  transactionProcessor,
  type TimeBlockTransactionResult,
  type DeleteTimeBlockResponse,
} from '@/infra/transaction/transactionProcessor'
import type { ISADefinition } from './types'

export const TimeBlockISA: ISADefinition = {
  'timeblock.create_from_task': {
    meta: {
      description: '从任务创建时间块',
      category: 'system',
      resourceIdentifier: (payload) => [`task:${payload.task_id}`, `timeblock:create`],
      priority: 6,
      timeout: 10000,
    },
    request: {
      method: 'POST',
      url: '/time-blocks/from-task',
      body: (payload) => ({
        task_id: payload.task_id,
        start_time: payload.start_time,
        end_time: payload.end_time,
        start_time_local: payload.start_time_local,
        end_time_local: payload.end_time_local,
        time_type: payload.time_type,
        creation_timezone: payload.creation_timezone,
        is_all_day: payload.is_all_day,
      }),
    },
    commit: async (result: TimeBlockTransactionResult, _payload, context) => {
      await transactionProcessor.applyTimeBlockTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'timeblock.create': {
    meta: {
      description: '创建空时间块',
      category: 'system',
      resourceIdentifier: () => ['timeblock:create'],
      priority: 6,
      timeout: 10000,
    },
    request: {
      method: 'POST',
      url: '/time-blocks',
      body: (payload) => ({
        title: payload.title,
        start_time: payload.start_time,
        end_time: payload.end_time,
        start_time_local: payload.start_time_local,
        end_time_local: payload.end_time_local,
        time_type: payload.time_type,
        creation_timezone: payload.creation_timezone,
        is_all_day: payload.is_all_day,
      }),
    },
    commit: async (result: TimeBlockTransactionResult, _payload, context) => {
      await transactionProcessor.applyTimeBlockTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'timeblock.update': {
    meta: {
      description: '更新时间块',
      category: 'system',
      resourceIdentifier: (payload) => [`timeblock:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    request: {
      method: 'PATCH',
      url: (payload) => `/time-blocks/${payload.id}`,
      body: (payload) => ({
        title: payload.updates.title,
        start_time: payload.updates.start_time,
        end_time: payload.updates.end_time,
        start_time_local: payload.updates.start_time_local,
        end_time_local: payload.updates.end_time_local,
        time_type: payload.updates.time_type,
        is_all_day: payload.updates.is_all_day,
      }),
    },
    commit: async (result: TimeBlockTransactionResult, _payload, context) => {
      await transactionProcessor.applyTimeBlockTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'timeblock.delete': {
    meta: {
      description: '删除时间块',
      category: 'system',
      resourceIdentifier: (payload) => [`timeblock:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    request: {
      method: 'DELETE',
      url: (payload) => `/time-blocks/${payload.id}`,
    },
    commit: async (result: DeleteTimeBlockResponse, _payload, context) => {
      await transactionProcessor.applyDeleteTimeBlock(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}

/**
 * TimeBlock ISA - 时间块指令集
 *
 * 包含指令：
 * - time_block.create_from_task: 从任务创建时间块（带乐观更新）
 * - time_block.create: 创建空时间块（带乐观更新）
 * - time_block.update: 更新时间块
 * - time_block.delete: 删除时间块
 *
 * 特点：
 * - create 操作使用乐观更新防止 UI 闪烁
 * - 使用 transactionProcessor 统一处理结果
 */

import {
  transactionProcessor,
  type TimeBlockTransactionResult,
  type DeleteTimeBlockResponse,
} from '@/infra/transaction/transactionProcessor'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useTaskStore } from '@/stores/task'
import type { TimeBlockView } from '@/types/dtos'
import type { ISADefinition } from './types'

export const TimeBlockISA: ISADefinition = {
  'time_block.create_from_task': {
    meta: {
      description: '从任务创建时间块',
      category: 'system',
      resourceIdentifier: (payload) => [`task:${payload.task_id}`, `time_block:create`],
      priority: 6,
      timeout: 10000,
    },
    optimistic: {
      enabled: true,
      apply: (payload) => {
        const timeBlockStore = useTimeBlockStore()
        const taskStore = useTaskStore()

        // 生成临时 ID
        const tempId = `temp_timeblock_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

        // 获取关联的任务信息
        const task = taskStore.getTaskById_Mux(payload.task_id)

        // 🔥 创建临时时间块对象
        const tempTimeBlock: TimeBlockView = {
          id: tempId,
          start_time: payload.start_time,
          end_time: payload.end_time,
          start_time_local: payload.start_time_local || null,
          end_time_local: payload.end_time_local || null,
          time_type: payload.time_type,
          creation_timezone: payload.creation_timezone || null,
          is_all_day: payload.is_all_day,
          title: task?.title || null,
          glance_note: task?.glance_note || null,
          detail_note: null,
          area_id: task?.area_id || null,
          linked_tasks: task
            ? [
                {
                  id: task.id,
                  title: task.title,
                  is_completed: task.is_completed,
                },
              ]
            : [],
          is_recurring: false,
        }

        // 🔥 立即添加到 store
        timeBlockStore.addOrUpdateTimeBlock_mut(tempTimeBlock)

        // 返回快照（用于回滚）
        return { tempId }
      },
      rollback: (snapshot) => {
        const timeBlockStore = useTimeBlockStore()
        // 🔥 移除临时时间块
        timeBlockStore.removeTimeBlock_mut(snapshot.tempId)
      },
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
    commit: async (result: TimeBlockTransactionResult, _payload, context, optimisticSnapshot) => {
      const timeBlockStore = useTimeBlockStore()

      // 🔥 先删除临时时间片（如果存在）
      if (optimisticSnapshot?.tempId) {
        timeBlockStore.removeTimeBlock_mut(optimisticSnapshot.tempId)
      }

      // 🔥 然后添加真实的时间块
      await transactionProcessor.applyTimeBlockTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'time_block.create': {
    meta: {
      description: '创建空时间块',
      category: 'system',
      resourceIdentifier: () => ['time_block:create'],
      priority: 6,
      timeout: 10000,
    },
    optimistic: {
      enabled: false, //和拖放系统的兼容性正在排查中
      apply: (payload) => {
        const timeBlockStore = useTimeBlockStore()

        // 生成临时 ID
        const tempId = `temp_timeblock_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

        // 🔥 创建临时时间块对象
        const tempTimeBlock: TimeBlockView = {
          id: tempId,
          start_time: payload.start_time,
          end_time: payload.end_time,
          start_time_local: payload.start_time_local || null,
          end_time_local: payload.end_time_local || null,
          time_type: payload.time_type,
          creation_timezone: payload.creation_timezone || null,
          is_all_day: payload.is_all_day,
          title: payload.title || null,
          glance_note: null,
          detail_note: null,
          area_id: null,
          linked_tasks: [],
          is_recurring: false,
        }

        // 🔥 立即添加到 store
        timeBlockStore.addOrUpdateTimeBlock_mut(tempTimeBlock)

        // 返回快照（用于回滚）
        return { tempId }
      },
      rollback: (snapshot) => {
        const timeBlockStore = useTimeBlockStore()
        // 🔥 移除临时时间块
        timeBlockStore.removeTimeBlock_mut(snapshot.tempId)
      },
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
    commit: async (result: TimeBlockTransactionResult, _payload, context, optimisticSnapshot) => {
      const timeBlockStore = useTimeBlockStore()

      // 🔥 先删除临时时间片（如果存在）
      if (optimisticSnapshot?.tempId) {
        timeBlockStore.removeTimeBlock_mut(optimisticSnapshot.tempId)
      }

      // 🔥 然后添加真实的时间块
      await transactionProcessor.applyTimeBlockTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },

  'time_block.update': {
    meta: {
      description: '更新时间块',
      category: 'system',
      resourceIdentifier: (payload) => [`time_block:${payload.id}`],
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

  'time_block.delete': {
    meta: {
      description: '删除时间块',
      category: 'system',
      resourceIdentifier: (payload) => [`time_block:${payload.id}`],
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

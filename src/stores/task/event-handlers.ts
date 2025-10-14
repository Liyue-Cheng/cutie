/**
 * Task Store 事件处理器（重构版 v3.0）
 *
 * 职责：
 * - 处理 SSE 推送的领域事件
 * - 使用 transactionProcessor 统一处理，自动去重和应用副作用
 * - 极简化设计，后端保证数据一致性
 *
 * 架构变化：
 * - v1.0: 复杂的 correlation tracker + 手动副作用处理
 * - v2.0: 部分使用 correlation tracker
 * - v3.0: 完全委托给 transactionProcessor，零手动处理
 */

import { logger, LogTags } from '@/infra/logging/logger'
import { transactionProcessor } from '@/infra/transaction/transactionProcessor'
import type { createTaskCore } from './core'

/**
 * 创建事件处理功能
 */
export function createEventHandlers(core: ReturnType<typeof createTaskCore>) {
  /**
   * 初始化事件订阅（由 main.ts 调用）
   */
  function initEventSubscriptions() {
    import('@/infra/events/events').then(({ getEventSubscriber }) => {
      const subscriber = getEventSubscriber()
      if (!subscriber) {
        logger.warn(LogTags.STORE_TASKS, 'Event subscriber not initialized yet')
        return
      }

      // 订阅所有任务事件（统一处理）
      subscriber.on('task.completed', handleTaskTransactionEvent)
      subscriber.on('task.updated', handleTaskTransactionEvent)
      subscriber.on('task.trashed', handleTaskTransactionEvent)
      subscriber.on('task.archived', handleTaskTransactionEvent)
      subscriber.on('task.unarchived', handleTaskTransactionEvent)
      subscriber.on('task.returned_to_staging', handleTaskTransactionEvent)
      subscriber.on('task.reopened', handleTaskTransactionEvent)
      subscriber.on('task.permanently_deleted', handleTaskTransactionEvent)

      // 订阅时间块事件（处理受影响的任务）
      subscriber.on('time_blocks.deleted', handleTimeBlockEvent)
      subscriber.on('time_blocks.updated', handleTimeBlockEvent)
      subscriber.on('time_blocks.linked', handleTimeBlockEvent)
      subscriber.on('time_blocks.created', handleTimeBlockEvent)

      logger.info(LogTags.STORE_TASKS, 'Task event subscriptions initialized (v3.0)')
    })
  }

  /**
   * 统一的任务事务事件处理器
   * ✅ 使用 transactionProcessor 自动处理所有逻辑
   * ✅ 自动去重（基于 correlation_id 或 event_id）
   * ✅ 自动应用所有副作用（deleted/truncated/updated time_blocks）
   */
  async function handleTaskTransactionEvent(event: any) {
    try {
      await transactionProcessor.applyTaskTransaction(event.payload, {
        correlation_id: event.correlation_id,
        event_id: event.event_id,
        source: 'sse',
      })
    } catch (error) {
      logger.error(
        LogTags.STORE_TASKS,
        'Failed to process task transaction event',
        error instanceof Error ? error : new Error(String(error)),
        {
          eventType: event.event_type,
          correlationId: event.correlation_id,
        }
      )
    }
  }

  /**
   * 时间块事件处理器
   * ✅ 现在后端已包含完整的 affected_tasks，直接应用即可
   */
  async function handleTimeBlockEvent(event: any) {
    const payload = event.payload
    const affectedTasks = payload?.affected_tasks || []

    if (affectedTasks.length > 0) {
      logger.info(LogTags.STORE_TASKS, 'Applying time block event affected tasks', {
        eventType: event.event_type,
        count: affectedTasks.length,
      })

      // 直接更新受影响的任务
      const { addOrUpdateTask } = core
      for (const task of affectedTasks) {
        addOrUpdateTask(task)
      }
    }
  }

  return {
    initEventSubscriptions,
    handleTaskTransactionEvent,
    handleTimeBlockEvent,
  }
}

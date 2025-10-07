import { useTimeBlockStore } from '@/stores/timeblock'
import type { createTaskCore } from './core'
import type { createCrudOperations } from './crud-operations'

/**
 * Task Store 事件处理器
 *
 * 职责：
 * - 处理 SSE 推送的领域事件
 * - 协调任务和时间块的副作用处理
 * - 基于 correlation_id 去重，避免重复更新
 */

/**
 * 创建事件处理功能
 */
export function createEventHandlers(
  core: ReturnType<typeof createTaskCore>,
  crudOps: ReturnType<typeof createCrudOperations>
) {
  const { addOrUpdateTask, removeTask } = core
  const { correlationTracker } = crudOps

  /**
   * 初始化事件订阅（由 main.ts 调用）
   */
  function initEventSubscriptions() {
    import('@/services/events').then(({ getEventSubscriber }) => {
      const subscriber = getEventSubscriber()
      if (!subscriber) {
        console.warn('[TaskStore] Event subscriber not initialized yet')
        return
      }

      // 订阅任务完成事件
      subscriber.on('task.completed', handleTaskCompletedEvent)

      // 订阅任务更新事件
      subscriber.on('task.updated', handleTaskUpdatedEvent)

      // 订阅任务删除事件
      subscriber.on('task.deleted', handleTaskDeletedEvent)

      // 订阅任务归档事件
      subscriber.on('task.archived', handleTaskArchivedEvent)

      // 订阅任务取消归档事件
      subscriber.on('task.unarchived', handleTaskUnarchivedEvent)

      // 订阅时间块删除事件
      subscriber.on('time_blocks.deleted', handleTimeBlocksDeletedEvent)

      // 订阅时间块更新事件
      subscriber.on('time_blocks.updated', handleTimeBlocksUpdatedEvent)

      // 订阅时间块截断事件
      subscriber.on('time_blocks.truncated', handleTimeBlocksTruncatedEvent)
    })
  }

  /**
   * 幂等事件处理器：任务完成
   * ✅ 一次性处理整个业务事务（任务 + 所有副作用）
   * ✅ 基于 correlation_id 去重，避免重复更新
   */
  async function handleTaskCompletedEvent(event: any) {
    const task = event.payload.task
    const sideEffects = event.payload.side_effects
    const correlationId = event.correlation_id

    // ✅ 数据验证：确保任务数据完整
    if (!task || !task.id || !task.title) {
      console.error('[TaskStore] Invalid task data in SSE event:', task)
      return
    }

    // 记录 SSE 事件接收时间
    if (correlationId) {
      correlationTracker.markSseReceived(correlationId, 'completeTask')
    }

    // 判断是否是自己触发的操作
    const isOwnOperation = correlationTracker.isOwnOperation(correlationId)

    if (isOwnOperation) {
      console.log(
        '[TaskStore] Skipping duplicate task update (own operation):',
        task.id,
        'correlation:',
        correlationId
      )
      // ⚠️ 不更新任务数据（HTTP 响应已更新），但副作用仍要处理
    } else {
      console.log(
        '[TaskStore] Handling task.completed event from other source:',
        task.id,
        sideEffects
      )
      // 这是其他窗口/客户端触发的，完整更新
      addOrUpdateTask(task)
    }

    // 副作用总是处理（因为 HTTP 响应没有副作用数据）
    if (sideEffects?.deleted_time_blocks?.length || sideEffects?.truncated_time_blocks?.length) {
      const timeBlockStore = useTimeBlockStore()
      await timeBlockStore.handleTimeBlockSideEffects(sideEffects)

      // 记录副作用处理完成
      if (correlationId) {
        correlationTracker.markSideEffectsCompleted(correlationId, 'completeTask')
      }
    } else {
      // 没有副作用，也输出总结
      if (correlationId) {
        correlationTracker.markCompleted(correlationId, 'completeTask')
      }
    }

    // 清理 correlation_id（如果有）
    if (correlationId) {
      correlationTracker.finishTracking(correlationId)
    }
  }

  /**
   * 幂等事件处理器：任务更新
   * ✅ 一次性处理整个业务事务（任务 + 所有副作用）
   * ✅ 基于 correlation_id 去重，避免重复更新
   */
  async function handleTaskUpdatedEvent(event: any) {
    const task = event.payload.task
    const sideEffects = event.payload.side_effects
    const correlationId = event.correlation_id

    // ✅ 数据验证：确保任务数据完整
    if (!task || !task.id || !task.title) {
      console.error('[TaskStore] Invalid task data in SSE event:', task)
      return
    }

    // 记录 SSE 事件接收时间
    if (correlationId) {
      correlationTracker.markSseReceived(correlationId, 'updateTask')
    }

    // 判断是否是自己触发的操作
    const isOwnOperation = correlationTracker.isOwnOperation(correlationId)

    if (isOwnOperation) {
      console.log(
        '[TaskStore] Skipping duplicate task update (own operation):',
        task.id,
        'correlation:',
        correlationId
      )
      // ⚠️ 不更新任务（HTTP 响应已更新），但副作用仍要处理
    } else {
      // 不是自己的操作，更新任务
      console.log('[TaskStore] Updating task from SSE:', task.id)
      addOrUpdateTask(task)
    }

    // 处理副作用（无论是否是自己的操作）
    if (sideEffects) {
      console.log('[TaskStore] Processing side effects for task.updated:', sideEffects)
      // 委托给 TimeBlockStore 处理时间块副作用
      const timeBlockStore = useTimeBlockStore()
      await timeBlockStore.handleTimeBlockSideEffects(sideEffects)

      // 记录副作用处理完成
      if (correlationId) {
        correlationTracker.markSideEffectsCompleted(correlationId, 'updateTask')
      }
    } else {
      // 没有副作用，也输出总结
      if (correlationId) {
        correlationTracker.markCompleted(correlationId, 'updateTask')
      }
    }

    // 清理 correlation_id（如果有）
    if (correlationId) {
      correlationTracker.finishTracking(correlationId)
    }
  }

  /**
   * 幂等事件处理器：任务删除
   * ✅ 一次性处理整个业务事务（任务删除 + 孤儿时间块删除）
   * ✅ 基于 correlation_id 去重，避免重复删除
   */
  async function handleTaskDeletedEvent(event: any) {
    const task = event.payload.task
    const taskId = task.id
    const sideEffects = event.payload.side_effects
    const correlationId = event.correlation_id

    // 记录 SSE 事件接收时间
    if (correlationId) {
      correlationTracker.markSseReceived(correlationId, 'deleteTask')
    }

    // 判断是否是自己触发的操作
    const isOwnOperation = correlationTracker.isOwnOperation(correlationId)

    if (isOwnOperation) {
      console.log(
        '[TaskStore] Skipping duplicate task deletion (own operation):',
        taskId,
        'correlation:',
        correlationId
      )
      // ⚠️ 不删除任务（HTTP 响应已删除），但副作用仍要处理
    } else {
      console.log('[TaskStore] Handling task.deleted event from other source:', taskId, sideEffects)
      // 这是其他窗口/客户端触发的，完整处理
      removeTask(taskId)
    }

    // 副作用总是处理（因为 HTTP 响应没有副作用数据）
    if (sideEffects?.deleted_time_blocks?.length) {
      const timeBlockStore = useTimeBlockStore()
      await timeBlockStore.handleTimeBlockSideEffects({
        deleted_time_blocks: sideEffects.deleted_time_blocks,
      })

      // 记录副作用处理完成
      if (correlationId) {
        correlationTracker.markSideEffectsCompleted(correlationId, 'deleteTask')
      }
    } else {
      // 没有副作用，也输出总结
      if (correlationId) {
        correlationTracker.markCompleted(correlationId, 'deleteTask')
      }
    }

    // 清理 correlation_id（如果有）
    if (correlationId) {
      correlationTracker.finishTracking(correlationId)
    }
  }

  /**
   * 幂等事件处理器：任务归档
   * ✅ 基于 correlation_id 去重，避免重复更新
   */
  async function handleTaskArchivedEvent(event: any) {
    const task = event.payload.task
    const correlationId = event.correlation_id

    // ✅ 数据验证：确保任务数据完整
    if (!task || !task.id || !task.title) {
      console.error('[TaskStore] Invalid task data in SSE event:', task)
      return
    }

    // 记录 SSE 事件接收时间
    if (correlationId) {
      correlationTracker.markSseReceived(correlationId, 'archiveTask')
    }

    // 判断是否是自己触发的操作
    const isOwnOperation = correlationTracker.isOwnOperation(correlationId)

    if (isOwnOperation) {
      console.log(
        '[TaskStore] Skipping duplicate task update (own operation):',
        task.id,
        'correlation:',
        correlationId
      )
      // HTTP 响应已更新，跳过
    } else {
      console.log('[TaskStore] Handling task.archived event from other source:', task.id)
      // 这是其他窗口/客户端触发的，完整更新
      addOrUpdateTask(task)
    }

    // 记录完成
    if (correlationId) {
      correlationTracker.markCompleted(correlationId, 'archiveTask')
    }

    // 清理 correlation_id（如果有）
    if (correlationId) {
      correlationTracker.finishTracking(correlationId)
    }
  }

  /**
   * 幂等事件处理器：任务取消归档
   * ✅ 基于 correlation_id 去重，避免重复更新
   */
  async function handleTaskUnarchivedEvent(event: any) {
    const task = event.payload.task
    const correlationId = event.correlation_id

    // ✅ 数据验证：确保任务数据完整
    if (!task || !task.id || !task.title) {
      console.error('[TaskStore] Invalid task data in SSE event:', task)
      return
    }

    // 记录 SSE 事件接收时间
    if (correlationId) {
      correlationTracker.markSseReceived(correlationId, 'unarchiveTask')
    }

    // 判断是否是自己触发的操作
    const isOwnOperation = correlationTracker.isOwnOperation(correlationId)

    if (isOwnOperation) {
      console.log(
        '[TaskStore] Skipping duplicate task update (own operation):',
        task.id,
        'correlation:',
        correlationId
      )
      // HTTP 响应已更新，跳过
    } else {
      console.log('[TaskStore] Handling task.unarchived event from other source:', task.id)
      // 这是其他窗口/客户端触发的，完整更新
      addOrUpdateTask(task)
    }

    // 记录完成
    if (correlationId) {
      correlationTracker.markCompleted(correlationId, 'unarchiveTask')
    }

    // 清理 correlation_id（如果有）
    if (correlationId) {
      correlationTracker.finishTracking(correlationId)
    }
  }

  /**
   * 处理时间块删除事件
   * 时间块被删除时，需要刷新相关任务的数据
   */
  async function handleTimeBlocksDeletedEvent(event: any) {
    const payload = event.payload
    const taskIds = payload?.affected_task_ids || []

    console.log('[TaskStore] Handling time_blocks.deleted event, affected tasks:', taskIds)

    // 重新获取受影响的任务数据
    for (const taskId of taskIds) {
      try {
        await crudOps.fetchTaskDetail(taskId)
      } catch (error) {
        console.error(
          `[TaskStore] Failed to refresh task ${taskId} after time block deletion:`,
          error
        )
      }
    }
  }

  /**
   * 处理时间块更新事件
   * 时间块被更新（移动时间）时，需要刷新相关任务的数据
   */
  async function handleTimeBlocksUpdatedEvent(event: any) {
    const payload = event.payload
    const taskIds = payload?.affected_task_ids || []

    console.log('[TaskStore] Handling time_blocks.updated event, affected tasks:', taskIds)

    // 重新获取受影响的任务数据
    for (const taskId of taskIds) {
      try {
        await crudOps.fetchTaskDetail(taskId)
      } catch (error) {
        console.error(
          `[TaskStore] Failed to refresh task ${taskId} after time block update:`,
          error
        )
      }
    }
  }

  /**
   * 处理时间块截断事件
   * 时间块被截断时，需要刷新相关任务的数据
   */
  async function handleTimeBlocksTruncatedEvent(event: any) {
    const payload = event.payload
    const taskIds = payload?.affected_task_ids || []

    console.log('[TaskStore] Handling time_blocks.truncated event, affected tasks:', taskIds)

    // 重新获取受影响的任务数据
    for (const taskId of taskIds) {
      try {
        await crudOps.fetchTaskDetail(taskId)
      } catch (error) {
        console.error(
          `[TaskStore] Failed to refresh task ${taskId} after time block truncation:`,
          error
        )
      }
    }
  }

  return {
    initEventSubscriptions,
    handleTaskCompletedEvent,
    handleTaskUpdatedEvent,
    handleTaskDeletedEvent,
    handleTaskArchivedEvent,
    handleTaskUnarchivedEvent,
    handleTimeBlocksDeletedEvent,
    handleTimeBlocksUpdatedEvent,
    handleTimeBlocksTruncatedEvent,
  }
}

/**
 * Trash Store - Event Handlers
 */
import { getEventSubscriber } from '@/infra/events/events'
import { addOrUpdateTrashedTask, removeTrashedTask } from './core'
import type { DomainEvent } from '@/infra/events/events'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 初始化事件订阅
 */
export function initEventSubscriptions() {
  const subscriber = getEventSubscriber()
  if (!subscriber) {
    logger.warn(LogTags.STORE_TRASH, 'Event subscriber not initialized')
    return
  }

  subscriber.on('task.trashed', handleTaskTrashedEvent)
  subscriber.on('task.restored', handleTaskRestoredEvent)
  subscriber.on('task.permanently_deleted', handleTaskPermanentlyDeletedEvent)
  subscriber.on('trash.emptied', handleTrashEmptiedEvent)
}

function handleTaskTrashedEvent(event: DomainEvent) {
  const task = event.payload?.task
  if (task) {
    logger.info(LogTags.STORE_TRASH, 'Task trashed', { taskId: task.id })
    addOrUpdateTrashedTask(task)
  }
}

function handleTaskRestoredEvent(event: DomainEvent) {
  const task = event.payload?.task
  if (task) {
    logger.info(LogTags.STORE_TRASH, 'Task restored', { taskId: task.id })
    removeTrashedTask(task.id)
  }
}

function handleTaskPermanentlyDeletedEvent(event: DomainEvent) {
  const taskId = event.payload?.task_id
  if (taskId) {
    logger.info(LogTags.STORE_TRASH, 'Task permanently deleted', { taskId })
    removeTrashedTask(taskId)
  }
}

function handleTrashEmptiedEvent(event: DomainEvent) {
  const deletedTaskIds = event.payload?.deleted_task_ids || []
  logger.info(LogTags.STORE_TRASH, 'Trash emptied', { deletedTasksCount: deletedTaskIds.length })

  // 从回收站移除所有已删除的任务
  for (const taskId of deletedTaskIds) {
    removeTrashedTask(taskId)
  }
}

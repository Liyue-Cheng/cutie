/**
 * Trash Store - Event Handlers
 */
import { getEventSubscriber } from '@/services/events'
import { addOrUpdateTrashedTask, removeTrashedTask, clearAllTrashedTasks } from './core'
import type { DomainEvent } from '@/services/events'

/**
 * 初始化事件订阅
 */
export function initEventSubscriptions() {
  const subscriber = getEventSubscriber()
  if (!subscriber) {
    console.warn('[TrashStore] Event subscriber not initialized')
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
    console.log('[TrashStore] Task trashed:', task.id)
    addOrUpdateTrashedTask(task)
  }
}

function handleTaskRestoredEvent(event: DomainEvent) {
  const task = event.payload?.task
  if (task) {
    console.log('[TrashStore] Task restored:', task.id)
    removeTrashedTask(task.id)
  }
}

function handleTaskPermanentlyDeletedEvent(event: DomainEvent) {
  const taskId = event.payload?.task_id
  if (taskId) {
    console.log('[TrashStore] Task permanently deleted:', taskId)
    removeTrashedTask(taskId)
  }
}

function handleTrashEmptiedEvent(event: DomainEvent) {
  const deletedTaskIds = event.payload?.deleted_task_ids || []
  console.log('[TrashStore] Trash emptied, deleted tasks:', deletedTaskIds.length)

  // 从回收站移除所有已删除的任务
  for (const taskId of deletedTaskIds) {
    removeTrashedTask(taskId)
  }
}

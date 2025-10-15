/**
 * Trash Store - Event Handlers
 *
 * v2.0: 所有事件通过 INT（中断管理器）注册
 */
import { addOrUpdateTrashedTask, removeTrashedTask } from './core'
import type { InterruptEvent } from '@/cpu/interrupt/InterruptHandler'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 初始化事件订阅（v2.0 - via INT）
 */
export function initEventSubscriptions() {
  import('@/cpu/interrupt/InterruptHandler').then(({ interruptHandler }) => {
    // 🔥 注册到 INT（中断管理器）
    interruptHandler.on('task.trashed', handleTaskTrashedEvent)
    interruptHandler.on('task.restored', handleTaskRestoredEvent)
    interruptHandler.on('task.permanently_deleted', handleTaskPermanentlyDeletedEvent)
    interruptHandler.on('trash.emptied', handleTrashEmptiedEvent)

    logger.info(LogTags.STORE_TRASH, 'Trash event subscriptions initialized (v2.0 - via INT)')
  })
}

function handleTaskTrashedEvent(event: InterruptEvent) {
  const task = event.payload?.task
  if (task) {
    logger.info(LogTags.STORE_TRASH, 'Task trashed', { taskId: task.id })
    addOrUpdateTrashedTask(task)
  }
}

function handleTaskRestoredEvent(event: InterruptEvent) {
  const task = event.payload?.task
  if (task) {
    logger.info(LogTags.STORE_TRASH, 'Task restored', { taskId: task.id })
    removeTrashedTask(task.id)
  }
}

function handleTaskPermanentlyDeletedEvent(event: InterruptEvent) {
  const taskId = event.payload?.task_id
  if (taskId) {
    logger.info(LogTags.STORE_TRASH, 'Task permanently deleted', { taskId })
    removeTrashedTask(taskId)
  }
}

function handleTrashEmptiedEvent(event: InterruptEvent) {
  const deletedTaskIds = event.payload?.deleted_task_ids || []
  logger.info(LogTags.STORE_TRASH, 'Trash emptied', { deletedTasksCount: deletedTaskIds.length })

  // 从回收站移除所有已删除的任务
  for (const taskId of deletedTaskIds) {
    removeTrashedTask(taskId)
  }
}

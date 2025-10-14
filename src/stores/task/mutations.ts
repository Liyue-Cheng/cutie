/**
 * Task Store Mutations (纯数据操作)
 *
 * 职责：
 * - 提供纯粹的状态修改方法
 * - ❌ 不调用 API
 * - ❌ 不包含业务逻辑
 * - ✅ 只修改内存中的数据
 *
 * 命名规范：
 * - addOrUpdateTask_mut - 添加或更新任务
 * - removeTask_mut - 移除任务
 * - clearAllTasks_mut - 清空所有任务
 */

import type { TaskCard, TaskDetail } from '@/types/dtos'
import type { createTaskCore } from './core'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 创建纯数据操作方法
 */
export function createMutations(core: ReturnType<typeof createTaskCore>) {
  const { tasks } = core

  /**
   * 添加或更新任务
   *
   * @param task 任务数据
   */
  function addOrUpdateTask_mut(task: TaskCard | TaskDetail): void {
    tasks.value.set(task.id, task)
    logger.debug(LogTags.STORE_TASKS, 'Task added/updated in store', {
      taskId: task.id,
      title: task.title,
    })
  }

  /**
   * 批量添加或更新任务
   *
   * @param taskList 任务列表
   */
  function batchAddOrUpdateTasks_mut(taskList: (TaskCard | TaskDetail)[]): void {
    taskList.forEach((task) => {
      tasks.value.set(task.id, task)
    })
    logger.debug(LogTags.STORE_TASKS, 'Tasks batch added/updated in store', {
      count: taskList.length,
    })
  }

  /**
   * 移除任务
   *
   * @param taskId 任务ID
   */
  function removeTask_mut(taskId: string): void {
    const removed = tasks.value.delete(taskId)
    if (removed) {
      logger.debug(LogTags.STORE_TASKS, 'Task removed from store', { taskId })
    } else {
      logger.warn(LogTags.STORE_TASKS, 'Task not found in store', { taskId })
    }
  }

  /**
   * 批量移除任务
   *
   * @param taskIds 任务ID列表
   */
  function batchRemoveTasks_mut(taskIds: string[]): void {
    taskIds.forEach((taskId) => {
      tasks.value.delete(taskId)
    })
    logger.debug(LogTags.STORE_TASKS, 'Tasks batch removed from store', {
      count: taskIds.length,
    })
  }

  /**
   * 清空所有任务
   */
  function clearAllTasks_mut(): void {
    const count = tasks.value.size
    tasks.value.clear()
    logger.debug(LogTags.STORE_TASKS, 'All tasks cleared from store', { count })
  }

  /**
   * 部分更新任务字段
   *
   * @param taskId 任务ID
   * @param updates 要更新的字段
   */
  function patchTask_mut(taskId: string, updates: Partial<TaskCard | TaskDetail>): void {
    const existingTask = tasks.value.get(taskId)
    if (!existingTask) {
      logger.warn(LogTags.STORE_TASKS, 'Cannot patch: task not found', { taskId })
      return
    }

    const updatedTask = {
      ...existingTask,
      ...updates,
    }

    tasks.value.set(taskId, updatedTask)
    logger.debug(LogTags.STORE_TASKS, 'Task patched in store', {
      taskId,
      updatedFields: Object.keys(updates),
    })
  }

  return {
    addOrUpdateTask_mut,
    batchAddOrUpdateTasks_mut,
    removeTask_mut,
    batchRemoveTasks_mut,
    clearAllTasks_mut,
    patchTask_mut,
  }
}

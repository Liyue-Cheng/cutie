/**
 * 任务业务操作层
 *
 * 职责：
 * - 编排多个Store的协作，处理任务操作的副作用
 * - 提供语义化的业务操作API给UI组件
 * - 统一处理复杂的副作用逻辑，避免UI层代码重复
 *
 * 新架构说明：
 * - TaskStore 负责数据和动态过滤，任务状态改变会自动反映在视图中
 * - ViewStore 只负责排序，不需要手动同步任务列表
 * - 因此这里的操作变得非常简单
 */

import { useTaskStore, type CreateTaskPayload } from '@/stores/task'

export function useTaskOperations() {
  const taskStore = useTaskStore()

  /**
   * 完成任务
   * ✅ 新架构：任务状态改变后会自动从视图中消失（动态过滤）
   */
  async function completeTask(taskId: string): Promise<boolean> {
    try {
      const task = await taskStore.completeTask(taskId)

      if (!task) return false

      console.log('[TaskOperations] Task completed successfully:', taskId)
      // ✅ 无需任何额外处理，视图会自动更新
      return true
    } catch (error) {
      console.error('[TaskOperations] Error completing task:', error)
      return false
    }
  }

  /**
   * 删除任务
   * ✅ 新架构：任务删除后会自动从所有视图消失（动态过滤）
   */
  async function deleteTask(taskId: string): Promise<boolean> {
    try {
      const success = await taskStore.deleteTask(taskId)

      if (!success) return false

      console.log('[TaskOperations] Task deleted successfully:', taskId)
      // ✅ 无需手动从视图移除，TaskStore.getTaskById 返回 undefined 后自动过滤
      return true
    } catch (error) {
      console.error('[TaskOperations] Error deleting task:', error)
      return false
    }
  }

  /**
   * 重新打开任务
   * ✅ 新架构：任务重新打开后会自动出现在视图中（动态过滤）
   */
  async function reopenTask(taskId: string): Promise<boolean> {
    try {
      const task = await taskStore.reopenTask(taskId)

      if (!task) return false

      console.log('[TaskOperations] Task reopened successfully:', taskId)
      // ✅ 无需任何额外处理，视图会自动更新
      return true
    } catch (error) {
      console.error('[TaskOperations] Error reopening task:', error)
      return false
    }
  }

  /**
   * 创建任务
   * ✅ 新架构：新任务会自动出现在对应视图中（动态过滤）
   */
  async function createTask(payload: CreateTaskPayload): Promise<string | null> {
    try {
      const task = await taskStore.createTask(payload)
      if (!task) return null

      console.log('[TaskOperations] Task created successfully:', task.id)
      // ✅ 无需手动添加到视图，会自动出现在对应的过滤列表中
      return task.id
    } catch (error) {
      console.error('[TaskOperations] Error creating task:', error)
      return null
    }
  }

  /**
   * 归档任务
   * ✅ 新架构：任务归档后会自动从所有视图消失（动态过滤）
   */
  async function archiveTask(taskId: string): Promise<boolean> {
    try {
      const task = await taskStore.archiveTask(taskId)

      if (!task) return false

      console.log('[TaskOperations] Task archived successfully:', taskId)
      // ✅ 无需任何额外处理，视图会自动更新
      return true
    } catch (error) {
      console.error('[TaskOperations] Error archiving task:', error)
      return false
    }
  }

  /**
   * 取消归档任务
   * ✅ 新架构：任务取消归档后会自动出现在对应视图中（动态过滤）
   */
  async function unarchiveTask(taskId: string): Promise<boolean> {
    try {
      const task = await taskStore.unarchiveTask(taskId)

      if (!task) return false

      console.log('[TaskOperations] Task unarchived successfully:', taskId)
      // ✅ 无需任何额外处理，视图会自动更新
      return true
    } catch (error) {
      console.error('[TaskOperations] Error unarchiving task:', error)
      return false
    }
  }

  return {
    completeTask,
    deleteTask,
    reopenTask,
    createTask,
    archiveTask,
    unarchiveTask,
  }
}

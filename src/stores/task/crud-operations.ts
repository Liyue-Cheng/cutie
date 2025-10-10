import type { TaskCard, TaskDetail } from '@/types/dtos'
import { apiGet, apiPost, apiPatch, apiDelete, createCorrelationTracker } from '@/stores/shared'
import type {
  CreateTaskPayload,
  UpdateTaskPayload,
  CompleteTaskResponse,
  ReopenTaskResponse,
  ArchiveTaskResponse,
  UnarchiveTaskResponse,
} from './types'
import type { createTaskCore } from './core'
import { logger, LogTags } from '@/services/logger'

/**
 * Task Store CRUD 操作
 *
 * 职责：
 * - 任务的增删改查操作
 * - 任务状态变更（完成/重新打开）
 * - 性能监控和关联追踪
 */

/**
 * 创建 CRUD 操作功能
 */
export function createCrudOperations(core: ReturnType<typeof createTaskCore>) {
  const { addOrUpdateTask, removeTask, withLoading } = core
  const correlationTracker = createCorrelationTracker()

  /**
   * 创建新任务
   * API: POST /tasks
   */
  async function createTask(payload: CreateTaskPayload): Promise<TaskCard | null> {
    logger.info(LogTags.STORE_TASKS, 'Creating task', { payload })

    return withLoading(async () => {
      const newTask: TaskCard = await apiPost('/tasks', payload)
      addOrUpdateTask(newTask)
      logger.info(LogTags.STORE_TASKS, 'Created task', { taskId: newTask.id, title: newTask.title })
      return newTask
    }, 'create task')
  }

  /**
   * 创建任务并添加日程（合并操作）
   * API: POST /tasks/with-schedule
   */
  async function createTaskWithSchedule(
    payload: CreateTaskPayload & { scheduled_day: string }
  ): Promise<TaskCard | null> {
    const correlationId = correlationTracker.startTracking('createTaskWithSchedule')

    return withLoading(async () => {
      try {
        correlationTracker.markHttpSent(correlationId, 'createTaskWithSchedule')

        const newTask: TaskCard = await apiPost('/tasks/with-schedule', payload, correlationId)

        correlationTracker.markHttpReceived(correlationId, 'createTaskWithSchedule')

        addOrUpdateTask(newTask)
        logger.info(LogTags.STORE_TASKS, 'Created task with schedule (HTTP)', {
          taskId: newTask.id,
          title: newTask.title,
          scheduledDay: payload.scheduled_day,
          correlationId,
        })
        return newTask
      } catch (e) {
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, 'create task with schedule')
  }

  /**
   * 更新任务
   * API: PATCH /tasks/:id
   */
  async function updateTask(id: string, payload: UpdateTaskPayload): Promise<TaskCard | null> {
    logger.info(LogTags.STORE_TASKS, 'Updating task', { taskId: id, payload })

    return withLoading(async () => {
      const result = await apiPatch(`/tasks/${id}`, payload)
      const updatedTask: TaskCard = result.task
      addOrUpdateTask(updatedTask)
      logger.info(LogTags.STORE_TASKS, 'Updated task', {
        taskId: updatedTask.id,
        title: updatedTask.title,
      })
      return updatedTask
    }, `update task ${id}`)
  }

  /**
   * 获取任务详情
   * API: GET /tasks/:id
   */
  async function fetchTaskDetail(id: string): Promise<TaskDetail | null> {
    return withLoading(async () => {
      const taskDetail: TaskDetail = await apiGet(`/tasks/${id}`)
      addOrUpdateTask(taskDetail) // 会自动合并并覆盖旧的 TaskCard 数据
      logger.info(LogTags.STORE_TASKS, 'Fetched task detail', {
        taskId: taskDetail.id,
        title: taskDetail.title,
      })
      return taskDetail
    }, `fetch task detail ${id}`)
  }

  /**
   * 删除任务
   * API: DELETE /tasks/:id
   */
  async function deleteTask(id: string): Promise<boolean> {
    // 生成 correlation_id 用于请求追踪和事件去重
    const correlationId = correlationTracker.startTracking('deleteTask')

    return (
      withLoading(async () => {
        try {
          // 记录 HTTP 请求发送
          correlationTracker.markHttpSent(correlationId, 'deleteTask')

          await apiDelete(`/tasks/${id}`, correlationId)

          // 记录 HTTP 响应接收
          correlationTracker.markHttpReceived(correlationId, 'deleteTask')

          // 删除任务（主要响应数据）
          removeTask(id)

          // ✅ 注意：副作用（deleted orphan time blocks）已通过 SSE 推送
          // HTTP响应体现在只返回 success 标志，真实的副作用由事件处理器处理

          logger.info(LogTags.STORE_TASKS, 'Deleted task (HTTP)', { taskId: id, correlationId })
          return true
        } catch (e) {
          // 清理失败的追踪
          correlationTracker.cleanupFailedTracking(correlationId)
          throw e
        } finally {
          // 10秒后清理 correlation_id 和性能计时器（防止内存泄漏）
          correlationTracker.finishTracking(correlationId, 10000)
        }
      }, `delete task ${id}`) !== null
    )
  }

  /**
   * 完成任务
   * API: POST /tasks/:id/complete
   */
  async function completeTask(id: string): Promise<TaskCard | null> {
    // 生成 correlation_id 用于请求追踪和事件去重
    const correlationId = correlationTracker.startTracking('completeTask')

    return withLoading(async () => {
      try {
        // 记录 HTTP 请求发送
        correlationTracker.markHttpSent(correlationId, 'completeTask')

        const data: CompleteTaskResponse = await apiPost(
          `/tasks/${id}/completion`,
          undefined,
          correlationId
        )

        // 记录 HTTP 响应接收
        correlationTracker.markHttpReceived(correlationId, 'completeTask')

        // 更新任务（主要响应数据）
        addOrUpdateTask(data.task)

        // ✅ 注意：副作用（deleted/truncated time blocks）已通过 SSE 推送
        // HTTP响应体现在返回空的ID列表，真实的副作用由事件处理器处理

        logger.info(LogTags.STORE_TASKS, 'Completed task (HTTP)', {
          taskId: data.task.id,
          correlationId,
        })
        return data.task
      } catch (e) {
        // 清理失败的追踪
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        // 10秒后清理 correlation_id 和性能计时器（防止内存泄漏）
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, `complete task ${id}`)
  }

  /**
   * 重新打开任务
   * API: DELETE /tasks/:id/completion
   */
  async function reopenTask(id: string): Promise<TaskCard | null> {
    // 生成 correlation_id 用于请求追踪和事件去重
    const correlationId = correlationTracker.startTracking('reopenTask')

    return withLoading(async () => {
      try {
        // 记录 HTTP 请求发送
        correlationTracker.markHttpSent(correlationId, 'reopenTask')
        const httpSentTimestamp = new Date().toISOString()
        logger.debug(LogTags.PERF, 'HTTP request sent for reopen task', {
          timestamp: httpSentTimestamp,
          correlationId,
        })

        const data: ReopenTaskResponse = await apiDelete(`/tasks/${id}/completion`, correlationId)

        // 记录 HTTP 响应接收
        correlationTracker.markHttpReceived(correlationId, 'reopenTask')
        const httpReceivedTimestamp = new Date().toISOString()
        logger.debug(LogTags.PERF, 'HTTP response received for reopen task', {
          timestamp: httpReceivedTimestamp,
          correlationId,
        })

        const reopenedTask: TaskCard = data.task
        addOrUpdateTask(reopenedTask)
        logger.info(LogTags.STORE_TASKS, 'Reopened task (HTTP)', {
          taskId: reopenedTask.id,
          correlationId,
        })
        return reopenedTask
      } catch (e) {
        // 清理失败的追踪
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        // 10秒后清理 correlation_id 和性能计时器（防止内存泄漏）
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, `reopen task ${id}`)
  }

  /**
   * 添加日程
   * API: POST /tasks/:id/schedules
   */
  async function addSchedule(taskId: string, scheduledDay: string): Promise<TaskCard | null> {
    const correlationId = correlationTracker.startTracking('addSchedule')

    return withLoading(async () => {
      try {
        correlationTracker.markHttpSent(correlationId, 'addSchedule')

        const data = await apiPost(
          `/tasks/${taskId}/schedules`,
          { scheduled_day: scheduledDay },
          correlationId
        )

        correlationTracker.markHttpReceived(correlationId, 'addSchedule')

        const updatedTask: TaskCard = data.task_card
        addOrUpdateTask(updatedTask)

        logger.info(LogTags.STORE_TASKS, 'Added schedule (HTTP)', {
          taskId: updatedTask.id,
          correlationId,
        })
        return updatedTask
      } catch (e) {
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, `add schedule for task ${taskId}`)
  }

  /**
   * 更新日程
   * API: PATCH /tasks/:id/schedules/:date
   */
  async function updateSchedule(
    taskId: string,
    date: string,
    payload: { new_date?: string; outcome?: string }
  ): Promise<TaskCard | null> {
    const correlationId = correlationTracker.startTracking('updateSchedule')

    return withLoading(async () => {
      try {
        correlationTracker.markHttpSent(correlationId, 'updateSchedule')

        const data = await apiPatch(`/tasks/${taskId}/schedules/${date}`, payload, correlationId)

        correlationTracker.markHttpReceived(correlationId, 'updateSchedule')

        const updatedTask: TaskCard = data.task_card
        addOrUpdateTask(updatedTask)

        logger.info(LogTags.STORE_TASKS, 'Updated schedule (HTTP)', {
          taskId: updatedTask.id,
          correlationId,
        })
        return updatedTask
      } catch (e) {
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, `update schedule for task ${taskId}`)
  }

  /**
   * 删除日程
   * API: DELETE /tasks/:id/schedules/:date
   */
  async function deleteSchedule(taskId: string, date: string): Promise<TaskCard | null> {
    const correlationId = correlationTracker.startTracking('deleteSchedule')

    return withLoading(async () => {
      try {
        correlationTracker.markHttpSent(correlationId, 'deleteSchedule')

        const data = await apiDelete(`/tasks/${taskId}/schedules/${date}`, correlationId)

        correlationTracker.markHttpReceived(correlationId, 'deleteSchedule')

        const updatedTask: TaskCard = data.task_card
        addOrUpdateTask(updatedTask)

        // ✅ 注意：副作用（deleted time blocks）已通过 SSE 推送

        logger.info(LogTags.STORE_TASKS, 'Deleted schedule (HTTP)', {
          taskId: updatedTask.id,
          correlationId,
        })
        return updatedTask
      } catch (e) {
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, `delete schedule for task ${taskId}`)
  }

  /**
   * 返回暂存区
   * API: POST /tasks/:id/return-to-staging
   */
  async function returnToStaging(taskId: string): Promise<TaskCard | null> {
    const correlationId = correlationTracker.startTracking('returnToStaging')

    return withLoading(async () => {
      try {
        correlationTracker.markHttpSent(correlationId, 'returnToStaging')

        const data = await apiPost(`/tasks/${taskId}/return-to-staging`, undefined, correlationId)

        correlationTracker.markHttpReceived(correlationId, 'returnToStaging')

        const updatedTask: TaskCard = data.task_card
        addOrUpdateTask(updatedTask)

        // ✅ 注意：副作用（deleted time blocks）已通过 SSE 推送

        logger.info(LogTags.STORE_TASKS, 'Returned to staging (HTTP)', {
          taskId: updatedTask.id,
          correlationId,
        })
        return updatedTask
      } catch (e) {
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, `return task ${taskId} to staging`)
  }

  /**
   * 归档任务
   * API: POST /tasks/:id/archive
   */
  async function archiveTask(id: string): Promise<TaskCard | null> {
    const correlationId = correlationTracker.startTracking('archiveTask')

    return withLoading(async () => {
      try {
        correlationTracker.markHttpSent(correlationId, 'archiveTask')

        const data: ArchiveTaskResponse = await apiPost(
          `/tasks/${id}/archive`,
          undefined,
          correlationId
        )

        correlationTracker.markHttpReceived(correlationId, 'archiveTask')

        addOrUpdateTask(data.task)

        logger.info(LogTags.STORE_TASKS, 'Archived task (HTTP)', {
          taskId: data.task.id,
          correlationId,
        })
        return data.task
      } catch (e) {
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, `archive task ${id}`)
  }

  /**
   * 取消归档任务
   * API: POST /tasks/:id/unarchive
   */
  async function unarchiveTask(id: string): Promise<TaskCard | null> {
    const correlationId = correlationTracker.startTracking('unarchiveTask')

    return withLoading(async () => {
      try {
        correlationTracker.markHttpSent(correlationId, 'unarchiveTask')

        const data: UnarchiveTaskResponse = await apiPost(
          `/tasks/${id}/unarchive`,
          undefined,
          correlationId
        )

        correlationTracker.markHttpReceived(correlationId, 'unarchiveTask')

        addOrUpdateTask(data.task)

        logger.info(LogTags.STORE_TASKS, 'Unarchived task (HTTP)', {
          taskId: data.task.id,
          correlationId,
        })
        return data.task
      } catch (e) {
        correlationTracker.cleanupFailedTracking(correlationId)
        throw e
      } finally {
        correlationTracker.finishTracking(correlationId, 10000)
      }
    }, `unarchive task ${id}`)
  }

  return {
    createTask,
    createTaskWithSchedule,
    updateTask,
    fetchTaskDetail,
    deleteTask,
    completeTask,
    reopenTask,
    archiveTask,
    unarchiveTask,
    addSchedule,
    updateSchedule,
    deleteSchedule,
    returnToStaging,
    // 暴露 correlation tracker 供事件处理器使用
    correlationTracker,
  }
}

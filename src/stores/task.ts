import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { TaskCard, TaskDetail } from '@/types/dtos'
import { waitForApiReady } from '@/composables/useApiConfig'

/**
 * Task Store
 *
 * 架构原则：
 * - State: 只存储最原始、最规范化的数据
 * - Actions: 负责执行操作、调用API、修改State
 * - Getters: 只负责从State中读取和计算数据，不修改State
 */

// --- Payload Types for API calls ---
export interface CreateTaskPayload {
  title: string
  glance_note?: string | null
  detail_note?: string | null
  area_id?: string | null
  due_date?: string | null
  due_date_type?: 'soft' | 'hard' | null
  project_id?: string | null
  subtasks?: Array<{
    title: string
    is_completed: boolean
  }> | null
}

export interface UpdateTaskPayload {
  title?: string
  glance_note?: string | null
  detail_note?: string | null
  area_id?: string | null
  due_date?: string | null
  due_date_type?: 'soft' | 'hard' | null
  project_id?: string | null
  subtasks?: Array<{
    id?: string
    title: string
    is_completed: boolean
  }> | null
}

/**
 * 完成任务的响应数据
 */
export interface CompleteTaskResponse {
  task: TaskCard
  // 注意：副作用（deleted/truncated time blocks）已通过 SSE 推送
}

/**
 * 删除任务的响应数据（副作用通过SSE）
 */
export interface DeleteTaskResponse {
  success: boolean
}

/**
 * 重新打开任务的响应数据
 */
export interface ReopenTaskResponse {
  task: TaskCard
}

export const useTaskStore = defineStore('task', () => {
  // ============================================================
  // STATE - 只存储最原始、最规范化的数据
  // ============================================================

  /**
   * 任务映射表 (单一数据源)
   * key: task_id
   * value: TaskCard | TaskDetail (总是保存当前最完整的信息)
   *
   * 说明：TaskDetail extends TaskCard，所以可以安全地存储两种类型
   * 当获取详情时，会用 TaskDetail 覆盖原有的 TaskCard
   */
  const tasks = ref(new Map<string, TaskCard | TaskDetail>())

  /**
   * 加载状态
   */
  const isLoading = ref(false)

  /**
   * 错误信息
   */
  const error = ref<string | null>(null)

  // ============================================================
  // GETTERS - 动态过滤（所有视图的数据源）
  // ============================================================

  /**
   * 基础数组缓存层（性能优化）
   * ✅ 只转换一次 Map → Array，所有其他 getter 复用此数组
   */
  const allTasksArray = computed(() => {
    return Array.from(tasks.value.values())
  })

  /**
   * 获取所有任务（数组形式）
   */
  const allTasks = computed(() => {
    return allTasksArray.value
  })

  /**
   * Staging 任务（未安排且未完成）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   */
  const stagingTasks = computed(() => {
    return allTasksArray.value.filter(
      (task) => task.schedule_status === 'staging' && !task.is_completed
    )
  })

  /**
   * Planned 任务（已安排且未完成）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   */
  const plannedTasks = computed(() => {
    return allTasksArray.value.filter(
      (task) => task.schedule_status === 'scheduled' && !task.is_completed
    )
  })

  /**
   * 未完成的任务（所有状态）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   */
  const incompleteTasks = computed(() => {
    return allTasksArray.value.filter((task) => !task.is_completed)
  })

  /**
   * 已完成的任务
   * ✅ 性能优化：复用 allTasksArray
   */
  const completedTasks = computed(() => {
    return allTasksArray.value.filter((task) => task.is_completed)
  })

  /**
   * 已安排的任务（包括已完成和未完成）
   * @deprecated 使用 plannedTasks（只含未完成）
   */
  const scheduledTasks = computed(() => {
    return allTasksArray.value.filter((task) => task.schedule_status === 'scheduled')
  })

  /**
   * 根据 ID 获取任务（返回当前最完整的信息）
   */
  function getTaskById(id: string): TaskCard | TaskDetail | undefined {
    return tasks.value.get(id)
  }

  /**
   * 根据项目 ID 获取任务列表
   * ✅ 性能优化：复用 allTasksArray
   */
  const getTasksByProject = computed(() => {
    return (projectId: string) => {
      return allTasksArray.value.filter((task) => task.project_id === projectId)
    }
  })

  /**
   * 根据区域 ID 获取任务列表
   * ✅ 性能优化：复用 allTasksArray
   */
  const getTasksByArea = computed(() => {
    return (areaId: string) => {
      return allTasksArray.value.filter((task) => task.area?.id === areaId)
    }
  })

  // ============================================================
  // ACTIONS - 负责执行操作、调用API、修改State
  // ============================================================

  /**
   * 批量添加或更新任务（单一数据源）
   * 使用扩展运算符合并，保证新数据覆盖旧数据，但不会丢失已有字段
   */
  function addOrUpdateTasks(newTasks: (TaskCard | TaskDetail)[]) {
    const newMap = new Map(tasks.value)
    for (const task of newTasks) {
      // 合并现有数据和新数据，新数据优先
      const existingTask = newMap.get(task.id) || {}
      newMap.set(task.id, { ...existingTask, ...task })
    }
    tasks.value = newMap
  }

  /**
   * 添加或更新单个任务
   */
  function addOrUpdateTask(task: TaskCard | TaskDetail) {
    addOrUpdateTasks([task])
  }

  /**
   * 从 state 中移除任务
   */
  function removeTask(id: string) {
    const newMap = new Map(tasks.value)
    newMap.delete(id)
    tasks.value = newMap
  }

  /**
   * 获取所有任务（包括已完成）
   * API: GET /views/all
   */
  async function fetchAllTasks() {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/views/all`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const tasks: TaskCard[] = result.data
      addOrUpdateTasks(tasks)
      console.log('[TaskStore] Fetched', tasks.length, 'all tasks')
    } catch (e) {
      error.value = `Failed to fetch all tasks: ${e}`
      console.error('[TaskStore] Error fetching all tasks:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取所有未完成任务
   * API: GET /views/all-incomplete
   */
  async function fetchAllIncompleteTasks() {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/views/all-incomplete`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const tasks: TaskCard[] = result.data
      addOrUpdateTasks(tasks)
      console.log('[TaskStore] Fetched', tasks.length, 'incomplete tasks')
    } catch (e) {
      error.value = `Failed to fetch incomplete tasks: ${e}`
      console.error('[TaskStore] Error fetching incomplete tasks:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取已排期任务
   * API: GET /views/planned
   */
  async function fetchPlannedTasks() {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/views/planned`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const tasks: TaskCard[] = result.data
      addOrUpdateTasks(tasks)
      console.log('[TaskStore] Fetched', tasks.length, 'planned tasks')
    } catch (e) {
      error.value = `Failed to fetch planned tasks: ${e}`
      console.error('[TaskStore] Error fetching planned tasks:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取 Staging 区的任务
   * API: GET /views/staging
   */
  async function fetchStagingTasks() {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/views/staging`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const stagingTasks: TaskCard[] = result.data // 后端返回 { data: [...], timestamp: ... }
      addOrUpdateTasks(stagingTasks)
      console.log('[TaskStore] Fetched', stagingTasks.length, 'staging tasks')
    } catch (e) {
      error.value = `Failed to fetch staging tasks: ${e}`
      console.error('[TaskStore] Error fetching staging tasks:', e)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 创建新任务
   * API: POST /tasks
   */
  async function createTask(payload: CreateTaskPayload): Promise<TaskCard | null> {
    isLoading.value = true
    error.value = null
    console.log('[TaskStore] Creating task with payload:', payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!response.ok) {
        const errorData = await response.json()
        console.error('[TaskStore] API error:', errorData)
        throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorData)}`)
      }
      const result = await response.json()
      const newTask: TaskCard = result.data // 后端返回 { data: {...}, timestamp: ... }
      addOrUpdateTask(newTask)
      console.log('[TaskStore] Created task:', newTask)
      return newTask
    } catch (e) {
      error.value = `Failed to create task: ${e}`
      console.error('[TaskStore] Error creating task:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 更新任务
   * API: PATCH /tasks/:id
   */
  async function updateTask(id: string, payload: UpdateTaskPayload): Promise<TaskCard | null> {
    isLoading.value = true
    error.value = null
    console.log('[TaskStore] Updating task', id, 'with payload:', payload)
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const updatedTask: TaskCard = result.data
      addOrUpdateTask(updatedTask)
      console.log('[TaskStore] Updated task:', updatedTask)
      return updatedTask
    } catch (e) {
      error.value = `Failed to update task ${id}: ${e}`
      console.error('[TaskStore] Error updating task:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取任务详情
   * API: GET /tasks/:id
   */
  async function fetchTaskDetail(id: string): Promise<TaskDetail | null> {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const taskDetail: TaskDetail = result.data
      addOrUpdateTask(taskDetail) // 会自动合并并覆盖旧的 TaskCard 数据
      console.log('[TaskStore] Fetched task detail:', taskDetail)
      return taskDetail
    } catch (e) {
      error.value = `Failed to fetch task detail ${id}: ${e}`
      console.error('[TaskStore] Error fetching task detail:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 删除任务
   * API: DELETE /tasks/:id
   */
  async function deleteTask(id: string): Promise<boolean> {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}`, {
        method: 'DELETE',
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)

      // 删除任务（主要响应数据）
      removeTask(id)

      // ✅ 注意：副作用（deleted orphan time blocks）已通过 SSE 推送
      // HTTP响应体现在只返回 success 标志，真实的副作用由事件处理器处理

      console.log('[TaskStore] Deleted task:', id)
      return true
    } catch (e) {
      error.value = `Failed to delete task ${id}: ${e}`
      console.error('[TaskStore] Error deleting task:', e)
      return false
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 完成任务
   * API: POST /tasks/:id/complete
   */
  async function completeTask(id: string): Promise<TaskCard | null> {
    isLoading.value = true
    error.value = null
    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}/completion`, {
        method: 'POST',
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const data = result.data as CompleteTaskResponse

      // 更新任务（主要响应数据）
      addOrUpdateTask(data.task)

      // ✅ 注意：副作用（deleted/truncated time blocks）已通过 SSE 推送
      // HTTP响应体现在返回空的ID列表，真实的副作用由事件处理器处理

      console.log('[TaskStore] Completed task:', data.task)
      return data.task
    } catch (e) {
      error.value = `Failed to complete task ${id}: ${e}`
      console.error('[TaskStore] Error completing task:', e)
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 重新打开任务
   * API: DELETE /tasks/:id/completion
   */
  async function reopenTask(id: string): Promise<TaskCard | null> {
    isLoading.value = true
    error.value = null
    console.log('[TaskStore] Reopening task:', id)

    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/tasks/${id}/completion`, {
        method: 'DELETE',
      })
      if (!response.ok) {
        const errorData = await response.json()
        console.error('[TaskStore] API error:', errorData)
        throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorData)}`)
      }
      const result = await response.json()
      const data = result.data as ReopenTaskResponse
      const reopenedTask: TaskCard = data.task
      addOrUpdateTask(reopenedTask)
      console.log('[TaskStore] Reopened task:', reopenedTask)
      return reopenedTask
    } catch (e) {
      error.value = `Failed to reopen task ${id}: ${e}`
      console.error('[TaskStore] Error reopening task:', e)
      throw e // 重新抛出错误，让调用者处理
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 搜索任务
   * API: GET /tasks/search?q=...
   */
  async function searchTasks(query: string, limit?: number): Promise<TaskCard[]> {
    isLoading.value = true
    error.value = null
    try {
      // TODO: 实现 API 调用
      // const apiBaseUrl = await waitForApiReady()
      // const params = new URLSearchParams({ q: query })
      // if (limit) params.append('limit', limit.toString())
      // const response = await fetch(`${apiBaseUrl}/tasks/search?${params}`)
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)
      // const results: TaskCard[] = await response.json()
      // addOrUpdateTasks(results)
      // return results

      console.log('[TaskStore] searchTasks - API not implemented yet', { query, limit })
      return []
    } catch (e) {
      error.value = `Failed to search tasks: ${e}`
      console.error('[TaskStore] Error searching tasks:', e)
      return []
    } finally {
      isLoading.value = false
    }
  }

  // ============================================================
  // 事件订阅器 - 处理 SSE 推送的领域事件
  // ============================================================

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

      // 订阅任务删除事件
      subscriber.on('task.deleted', handleTaskDeletedEvent)
    })
  }

  /**
   * 幂等事件处理器：任务完成
   * ✅ 一次性处理整个业务事务（任务 + 所有副作用）
   */
  async function handleTaskCompletedEvent(event: any) {
    const task = event.payload.task
    const sideEffects = event.payload.side_effects
    console.log('[TaskStore] Handling task.completed event:', task.id, sideEffects)

    // 直接使用事件中的完整数据，无需额外 HTTP 请求 ✅
    addOrUpdateTask(task)

    // 处理副作用：通知 TimeBlockStore
    if (sideEffects?.deleted_time_blocks?.length || sideEffects?.truncated_time_blocks?.length) {
      const { useTimeBlockStore } = await import('./timeblock')
      const timeBlockStore = useTimeBlockStore()
      timeBlockStore.handleTimeBlockSideEffects(sideEffects)
    }
  }

  /**
   * 幂等事件处理器：任务删除
   * ✅ 一次性处理整个业务事务（任务删除 + 孤儿时间块删除）
   */
  async function handleTaskDeletedEvent(event: any) {
    const taskId = event.payload.task_id
    const sideEffects = event.payload.side_effects
    console.log('[TaskStore] Handling task.deleted event:', taskId, sideEffects)

    // 从本地状态移除任务
    removeTask(taskId)

    // 处理副作用：通知 TimeBlockStore 删除孤儿时间块
    if (sideEffects?.deleted_time_blocks?.length) {
      const { useTimeBlockStore } = await import('./timeblock')
      const timeBlockStore = useTimeBlockStore()
      timeBlockStore.handleTimeBlockSideEffects({
        deleted_time_blocks: sideEffects.deleted_time_blocks,
      })
    }
  }

  return {
    // State
    tasks,
    isLoading,
    error,

    // Getters - 所有视图的数据源
    allTasks,
    stagingTasks, // ✅ 动态过滤
    plannedTasks, // ✅ 动态过滤
    incompleteTasks, // ✅ 动态过滤
    completedTasks,
    scheduledTasks, // @deprecated
    getTaskById,
    getTasksByProject,
    getTasksByArea,

    // Actions
    addOrUpdateTasks,
    addOrUpdateTask,
    removeTask,
    fetchAllTasks,
    fetchAllIncompleteTasks,
    fetchPlannedTasks,
    fetchStagingTasks,
    createTask,
    updateTask,
    fetchTaskDetail,
    deleteTask,
    completeTask,
    reopenTask,
    searchTasks,

    // Event handlers
    initEventSubscriptions,
  }
})

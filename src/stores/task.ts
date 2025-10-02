import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { TaskCard, TaskDetail } from '@/types/dtos'
import { waitForApiReady } from '@/composables/useApiConfig'
import { useTimeBlockStore } from './timeblock'

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

  /**
   * 待处理的 Correlation IDs（用于去重和请求追踪）
   *
   * 原理：
   * - HTTP 请求时生成并记录 correlation_id
   * - SSE 事件到达时检查是否是自己触发的
   * - 如果是，跳过任务数据更新（HTTP 已更新），但仍处理副作用
   * - 5秒后自动清理（防止内存泄漏）
   */
  const pendingCorrelations = ref(new Set<string>())

  /**
   * 性能计时器：记录每个请求的各阶段时间戳
   * key: correlation_id
   * value: { start, httpSent, httpReceived, sseReceived, completed }
   */
  const performanceTimers = ref(
    new Map<
      string,
      {
        start: number
        httpSent: number
        httpReceived?: number
        sseReceived?: number
        sideEffectsCompleted?: number
      }
    >()
  )

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
  // function addOrUpdateTasks(newTasks: (TaskCard | TaskDetail)[]) {
  //   const newMap = new Map(tasks.value)
  //   for (const task of newTasks) {
  //     // 合并现有数据和新数据，新数据优先
  //     const existingTask = newMap.get(task.id) || {}
  //     newMap.set(task.id, { ...existingTask, ...task })
  //   }
  //   tasks.value = newMap
  // }
  function addOrUpdateTasks(newTasks: (TaskCard | TaskDetail)[]) {
    for (const task of newTasks) {
      if (!task || !task.id) {
        console.warn('[TaskStore] Skipping task without ID', task)
        continue
      }

      // 正确的做法：直接用服务器返回的权威数据进行设置
      // tasks.value 是一个响应式 Map，调用 .set() 会被 Vue 侦测到
      // Vue 会自动将新设置的 task 对象转换为响应式代理
      tasks.value.set(task.id, task)
    }
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
      const updatedTask: TaskCard = result.data.task
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

    // 生成 correlation_id 用于请求追踪和事件去重
    const correlationId = crypto.randomUUID()
    pendingCorrelations.value.add(correlationId)

    // ⏱️ 性能计时：阶段1 - 开始
    const startTime = performance.now()
    performanceTimers.value.set(correlationId, {
      start: startTime,
      httpSent: 0,
    })
    console.log(`[⏱️ Performance] deleteTask START | task: ${id} | correlation: ${correlationId}`)

    try {
      const apiBaseUrl = await waitForApiReady()

      // ⏱️ 性能计时：阶段2 - HTTP 请求发送
      const httpSentTime = performance.now()
      const timer = performanceTimers.value.get(correlationId)
      if (timer) {
        timer.httpSent = httpSentTime
      }
      console.log(
        `[⏱️ Performance] HTTP REQUEST SENT | Δ=${(httpSentTime - startTime).toFixed(2)}ms | correlation: ${correlationId}`
      )

      const response = await fetch(`${apiBaseUrl}/tasks/${id}`, {
        method: 'DELETE',
        headers: {
          'X-Correlation-ID': correlationId,
        },
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)

      // ⏱️ 性能计时：阶段3 - HTTP 响应接收
      const httpReceivedTime = performance.now()
      if (timer) {
        timer.httpReceived = httpReceivedTime
      }
      const httpRoundtrip = httpReceivedTime - httpSentTime
      const totalSoFar = httpReceivedTime - startTime
      console.log(
        `[⏱️ Performance] HTTP RESPONSE RECEIVED | Δ=${httpRoundtrip.toFixed(2)}ms | Total=${totalSoFar.toFixed(2)}ms | correlation: ${correlationId}`
      )

      // 删除任务（主要响应数据）
      removeTask(id)

      // ✅ 注意：副作用（deleted orphan time blocks）已通过 SSE 推送
      // HTTP响应体现在只返回 success 标志，真实的副作用由事件处理器处理

      console.log('[TaskStore] Deleted task (HTTP):', id, 'correlation:', correlationId)
      return true
    } catch (e) {
      error.value = `Failed to delete task ${id}: ${e}`
      console.error('[TaskStore] Error deleting task:', e)
      // 清理性能计时器
      performanceTimers.value.delete(correlationId)
      return false
    } finally {
      isLoading.value = false
      // 10秒后清理 correlation_id 和性能计时器（防止内存泄漏）
      setTimeout(() => {
        pendingCorrelations.value.delete(correlationId)
        performanceTimers.value.delete(correlationId)
      }, 10000)
    }
  }

  /**
   * 完成任务
   * API: POST /tasks/:id/complete
   */
  async function completeTask(id: string): Promise<TaskCard | null> {
    isLoading.value = true
    error.value = null

    // 生成 correlation_id 用于请求追踪和事件去重
    const correlationId = crypto.randomUUID()
    pendingCorrelations.value.add(correlationId)

    // ⏱️ 性能计时：阶段1 - 开始
    const startTime = performance.now()
    performanceTimers.value.set(correlationId, {
      start: startTime,
      httpSent: 0,
    })
    console.log(`[⏱️ Performance] completeTask START | task: ${id} | correlation: ${correlationId}`)

    try {
      const apiBaseUrl = await waitForApiReady()

      // ⏱️ 性能计时：阶段2 - HTTP 请求发送
      const httpSentTime = performance.now()
      const timer = performanceTimers.value.get(correlationId)
      if (timer) {
        timer.httpSent = httpSentTime
      }
      console.log(
        `[⏱️ Performance] HTTP REQUEST SENT | Δ=${(httpSentTime - startTime).toFixed(2)}ms | correlation: ${correlationId}`
      )

      const response = await fetch(`${apiBaseUrl}/tasks/${id}/completion`, {
        method: 'POST',
        headers: {
          'X-Correlation-ID': correlationId,
        },
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const data = result.data as CompleteTaskResponse

      // ⏱️ 性能计时：阶段3 - HTTP 响应接收
      const httpReceivedTime = performance.now()
      if (timer) {
        timer.httpReceived = httpReceivedTime
      }
      const httpRoundtrip = httpReceivedTime - httpSentTime
      const totalSoFar = httpReceivedTime - startTime
      console.log(
        `[⏱️ Performance] HTTP RESPONSE RECEIVED | Δ=${httpRoundtrip.toFixed(2)}ms | Total=${totalSoFar.toFixed(2)}ms | correlation: ${correlationId}`
      )

      // 更新任务（主要响应数据）
      addOrUpdateTask(data.task)

      // ✅ 注意：副作用（deleted/truncated time blocks）已通过 SSE 推送
      // HTTP响应体现在返回空的ID列表，真实的副作用由事件处理器处理

      console.log('[TaskStore] Completed task (HTTP):', data.task, 'correlation:', correlationId)
      return data.task
    } catch (e) {
      error.value = `Failed to complete task ${id}: ${e}`
      console.error('[TaskStore] Error completing task:', e)
      // 清理性能计时器
      performanceTimers.value.delete(correlationId)
      return null
    } finally {
      isLoading.value = false
      // 10秒后清理 correlation_id 和性能计时器（防止内存泄漏）
      setTimeout(() => {
        pendingCorrelations.value.delete(correlationId)
        performanceTimers.value.delete(correlationId)
      }, 10000)
    }
  }

  /**
   * 重新打开任务
   * API: DELETE /tasks/:id/completion
   */
  async function reopenTask(id: string): Promise<TaskCard | null> {
    isLoading.value = true
    error.value = null

    // 生成 correlation_id 用于请求追踪和事件去重
    const correlationId = crypto.randomUUID()
    pendingCorrelations.value.add(correlationId)

    // ⏱️ 性能计时：阶段1 - 开始
    const startTime = performance.now()
    performanceTimers.value.set(correlationId, {
      start: startTime,
      httpSent: 0,
    })
    console.log(`[⏱️ Performance] reopenTask START | task: ${id} | correlation: ${correlationId}`)

    try {
      const apiBaseUrl = await waitForApiReady()

      // ⏱️ 性能计时：阶段2 - HTTP 请求发送
      const httpSentTime = performance.now()
      const httpSentTimestamp = new Date().toISOString()
      const timer = performanceTimers.value.get(correlationId)
      if (timer) {
        timer.httpSent = httpSentTime
      }
      console.log(
        `[⏱️ Performance] HTTP REQUEST SENT | Δ=${(httpSentTime - startTime).toFixed(2)}ms | timestamp=${httpSentTimestamp} | correlation: ${correlationId}`
      )

      const response = await fetch(`${apiBaseUrl}/tasks/${id}/completion`, {
        method: 'DELETE',
        headers: {
          'X-Correlation-ID': correlationId,
        },
      })
      if (!response.ok) {
        const errorData = await response.json()
        console.error('[TaskStore] API error:', errorData)
        throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorData)}`)
      }
      const result = await response.json()
      const data = result.data as ReopenTaskResponse
      const reopenedTask: TaskCard = data.task

      // ⏱️ 性能计时：阶段3 - HTTP 响应接收
      const httpReceivedTime = performance.now()
      const httpReceivedTimestamp = new Date().toISOString()
      if (timer) {
        timer.httpReceived = httpReceivedTime
      }
      const httpRoundtrip = httpReceivedTime - httpSentTime
      const totalSoFar = httpReceivedTime - startTime
      console.log(
        `[⏱️ Performance] HTTP RESPONSE RECEIVED | Δ=${httpRoundtrip.toFixed(2)}ms | Total=${totalSoFar.toFixed(2)}ms | timestamp=${httpReceivedTimestamp} | correlation: ${correlationId}`
      )

      addOrUpdateTask(reopenedTask)
      console.log('[TaskStore] Reopened task (HTTP):', reopenedTask)
      return reopenedTask
    } catch (e) {
      error.value = `Failed to reopen task ${id}: ${e}`
      console.error('[TaskStore] Error reopening task:', e)
      // 清理性能计时器
      performanceTimers.value.delete(correlationId)
      throw e // 重新抛出错误，让调用者处理
    } finally {
      isLoading.value = false
      // 10秒后清理 correlation_id 和性能计时器（防止内存泄漏）
      setTimeout(() => {
        pendingCorrelations.value.delete(correlationId)
        performanceTimers.value.delete(correlationId)
      }, 10000)
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

      // 订阅任务更新事件
      subscriber.on('task.updated', handleTaskUpdatedEvent)

      // 订阅任务删除事件
      subscriber.on('task.deleted', handleTaskDeletedEvent)
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

    // ⏱️ 性能计时：阶段4 - SSE 事件接收
    const sseReceivedTime = performance.now()
    const timer = correlationId ? performanceTimers.value.get(correlationId) : undefined
    if (timer) {
      timer.sseReceived = sseReceivedTime
      const sseDelay = sseReceivedTime - (timer.httpReceived || timer.httpSent)
      const totalSoFar = sseReceivedTime - timer.start
      console.log(
        `[⏱️ Performance] SSE EVENT RECEIVED | Δ=${sseDelay.toFixed(2)}ms | Total=${totalSoFar.toFixed(2)}ms | correlation: ${correlationId}`
      )
    }

    // 判断是否是自己触发的操作
    const isOwnOperation = correlationId && pendingCorrelations.value.has(correlationId)

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
      const { useTimeBlockStore } = await import('./timeblock')
      const timeBlockStore = useTimeBlockStore()
      timeBlockStore.handleTimeBlockSideEffects(sideEffects)

      // ⏱️ 性能计时：阶段5 - 副作用处理完成
      const sideEffectsCompletedTime = performance.now()
      if (timer) {
        timer.sideEffectsCompleted = sideEffectsCompletedTime
        const sideEffectsDuration = sideEffectsCompletedTime - sseReceivedTime
        const totalDuration = sideEffectsCompletedTime - timer.start

        console.log(
          `[⏱️ Performance] SIDE EFFECTS COMPLETED | Δ=${sideEffectsDuration.toFixed(2)}ms | Total=${totalDuration.toFixed(2)}ms | correlation: ${correlationId}`
        )
        console.log(
          `[⏱️ Performance] 📊 COMPLETE SUMMARY | correlation: ${correlationId}\n` +
            `  ├─ Preparation:        ${(timer.httpSent - timer.start).toFixed(2)}ms\n` +
            `  ├─ HTTP Roundtrip:     ${((timer.httpReceived || 0) - timer.httpSent).toFixed(2)}ms\n` +
            `  ├─ SSE Delay:          ${((timer.sseReceived || 0) - (timer.httpReceived || timer.httpSent)).toFixed(2)}ms\n` +
            `  ├─ Side Effects:       ${sideEffectsDuration.toFixed(2)}ms\n` +
            `  └─ TOTAL:              ${totalDuration.toFixed(2)}ms ✅`
        )
      }
    } else {
      // 没有副作用，也输出总结
      if (timer) {
        const totalDuration = sseReceivedTime - timer.start
        console.log(
          `[⏱️ Performance] 📊 COMPLETE SUMMARY (no side effects) | correlation: ${correlationId}\n` +
            `  ├─ Preparation:        ${(timer.httpSent - timer.start).toFixed(2)}ms\n` +
            `  ├─ HTTP Roundtrip:     ${((timer.httpReceived || 0) - timer.httpSent).toFixed(2)}ms\n` +
            `  ├─ SSE Delay:          ${(sseReceivedTime - (timer.httpReceived || timer.httpSent)).toFixed(2)}ms\n` +
            `  └─ TOTAL:              ${totalDuration.toFixed(2)}ms ✅`
        )
      }
    }

    // 清理 correlation_id（如果有）
    if (correlationId) {
      pendingCorrelations.value.delete(correlationId)
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
    // ⏱️ 性能计时：阶段4 - SSE 事件接收
    const sseReceivedTime = performance.now()
    const timer = correlationId ? performanceTimers.value.get(correlationId) : undefined
    if (timer) {
      timer.sseReceived = sseReceivedTime
      const sseDelay = sseReceivedTime - (timer.httpReceived || timer.httpSent)
      const totalSoFar = sseReceivedTime - timer.start
      console.log(
        `[⏱️ Performance] SSE EVENT RECEIVED | Δ=${sseDelay.toFixed(2)}ms | Total=${totalSoFar.toFixed(2)}ms | correlation: ${correlationId}`
      )
    }
    // 判断是否是自己触发的操作
    const isOwnOperation = correlationId && pendingCorrelations.value.has(correlationId)
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
    }
    // ⏱️ 性能计时：阶段5 - 完成更新处理
    if (timer) {
      timer.sideEffectsCompleted = performance.now()
      const totalDuration = timer.sideEffectsCompleted - timer.start
      if (sideEffects && Object.keys(sideEffects).length > 0) {
        const sideEffectsDuration = timer.sideEffectsCompleted - sseReceivedTime
        console.log(
          `[⏱️ Performance] 📊 UPDATE SUMMARY (with side effects) | correlation: ${correlationId}\n` +
            `  ├─ Preparation:        ${(timer.httpSent - timer.start).toFixed(2)}ms\n` +
            `  ├─ HTTP Roundtrip:     ${((timer.httpReceived || 0) - timer.httpSent).toFixed(2)}ms\n` +
            `  ├─ SSE Delay:          ${(sseReceivedTime - (timer.httpReceived || timer.httpSent)).toFixed(2)}ms\n` +
            `  ├─ Side Effects:       ${sideEffectsDuration.toFixed(2)}ms\n` +
            `  └─ TOTAL:              ${totalDuration.toFixed(2)}ms ✅`
        )
      } else {
        console.log(
          `[⏱️ Performance] 📊 UPDATE SUMMARY (no side effects) | correlation: ${correlationId}\n` +
            `  ├─ Preparation:        ${(timer.httpSent - timer.start).toFixed(2)}ms\n` +
            `  ├─ HTTP Roundtrip:     ${((timer.httpReceived || 0) - timer.httpSent).toFixed(2)}ms\n` +
            `  ├─ SSE Delay:          ${(sseReceivedTime - (timer.httpReceived || timer.httpSent)).toFixed(2)}ms\n` +
            `  └─ TOTAL:              ${totalDuration.toFixed(2)}ms ✅`
        )
      }
    }
    // 清理 correlation_id（如果有）
    if (correlationId) {
      pendingCorrelations.value.delete(correlationId)
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

    // ⏱️ 性能计时：阶段4 - SSE 事件接收
    const sseReceivedTime = performance.now()
    const timer = correlationId ? performanceTimers.value.get(correlationId) : undefined
    if (timer) {
      timer.sseReceived = sseReceivedTime
      const sseDelay = sseReceivedTime - (timer.httpReceived || timer.httpSent)
      const totalSoFar = sseReceivedTime - timer.start
      console.log(
        `[⏱️ Performance] SSE EVENT RECEIVED | Δ=${sseDelay.toFixed(2)}ms | Total=${totalSoFar.toFixed(2)}ms | correlation: ${correlationId}`
      )
    }

    // 判断是否是自己触发的操作
    const isOwnOperation = correlationId && pendingCorrelations.value.has(correlationId)

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
      const { useTimeBlockStore } = await import('./timeblock')
      const timeBlockStore = useTimeBlockStore()
      timeBlockStore.handleTimeBlockSideEffects({
        deleted_time_blocks: sideEffects.deleted_time_blocks,
      })

      // ⏱️ 性能计时：阶段5 - 副作用处理完成
      const sideEffectsCompletedTime = performance.now()
      if (timer) {
        timer.sideEffectsCompleted = sideEffectsCompletedTime
        const sideEffectsDuration = sideEffectsCompletedTime - sseReceivedTime
        const totalDuration = sideEffectsCompletedTime - timer.start

        console.log(
          `[⏱️ Performance] SIDE EFFECTS COMPLETED | Δ=${sideEffectsDuration.toFixed(2)}ms | Total=${totalDuration.toFixed(2)}ms | correlation: ${correlationId}`
        )
        console.log(
          `[⏱️ Performance] 📊 DELETE SUMMARY | correlation: ${correlationId}\n` +
            `  ├─ Preparation:        ${(timer.httpSent - timer.start).toFixed(2)}ms\n` +
            `  ├─ HTTP Roundtrip:     ${((timer.httpReceived || 0) - timer.httpSent).toFixed(2)}ms\n` +
            `  ├─ SSE Delay:          ${((timer.sseReceived || 0) - (timer.httpReceived || timer.httpSent)).toFixed(2)}ms\n` +
            `  ├─ Side Effects:       ${sideEffectsDuration.toFixed(2)}ms\n` +
            `  └─ TOTAL:              ${totalDuration.toFixed(2)}ms ✅`
        )
      }
    } else {
      // 没有副作用，也输出总结
      if (timer) {
        const totalDuration = sseReceivedTime - timer.start
        console.log(
          `[⏱️ Performance] 📊 DELETE SUMMARY (no side effects) | correlation: ${correlationId}\n` +
            `  ├─ Preparation:        ${(timer.httpSent - timer.start).toFixed(2)}ms\n` +
            `  ├─ HTTP Roundtrip:     ${((timer.httpReceived || 0) - timer.httpSent).toFixed(2)}ms\n` +
            `  ├─ SSE Delay:          ${(sseReceivedTime - (timer.httpReceived || timer.httpSent)).toFixed(2)}ms\n` +
            `  └─ TOTAL:              ${totalDuration.toFixed(2)}ms ✅`
        )
      }
    }

    // 清理 correlation_id（如果有）
    if (correlationId) {
      pendingCorrelations.value.delete(correlationId)
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

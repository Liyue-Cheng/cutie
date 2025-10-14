import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { TimeBlockView, TimeType } from '@/types/dtos'
import { getEventSubscriber } from '@/infra/events/events'
import { logger, LogTags } from '@/infra/logging/logger'
import { apiGet, apiPost, apiPatch, apiDelete } from '@/stores/shared'

/**
 * TimeBlock Store
 *
 * 职责：管理日历上的时间块
 *
 * 架构原则：
 * - State: 只存储最原始、最规范化的数据（TimeBlockView 映射表）
 * - Actions: 负责执行操作、调用API、修改State
 * - Getters: 只负责从State中读取和计算数据，不修改State
 */

// --- Payload Types for API calls ---
export interface CreateTimeBlockPayload {
  title?: string | null
  glance_note?: string | null
  detail_note?: string | null
  start_time: string // ISO 8601 UTC
  end_time: string // ISO 8601 UTC
  /** 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
  start_time_local?: string | null
  /** 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
  end_time_local?: string | null
  /** 时间类型，默认为FLOATING */
  time_type?: TimeType
  /** 创建时的时区（占位字段） */
  creation_timezone?: string | null
  is_all_day?: boolean
  area_id?: string | null
}

export interface CreateFromTaskPayload {
  task_id: string
  start_time: string // ISO 8601 UTC
  end_time: string // ISO 8601 UTC
  /** 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
  start_time_local?: string | null
  /** 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
  end_time_local?: string | null
  /** 时间类型，默认为FLOATING */
  time_type?: TimeType
  /** 创建时的时区（占位字段） */
  creation_timezone?: string | null
  title?: string | null // 可选，默认使用任务标题
  is_all_day?: boolean // 可选，是否为全天事件
}

export interface CreateFromTaskResponse {
  time_block: TimeBlockView
  updated_task: import('@/types/dtos').TaskCard // 更新后的任务
}

export interface UpdateTimeBlockPayload {
  title?: string | null
  glance_note?: string | null
  detail_note?: string | null
  start_time?: string
  end_time?: string
  /** 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
  start_time_local?: string | null
  /** 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时使用 */
  end_time_local?: string | null
  /** 时间类型 */
  time_type?: TimeType
  /** 创建时的时区（占位字段） */
  creation_timezone?: string | null
  is_all_day?: boolean
  area_id?: string | null
}

export const useTimeBlockStore = defineStore('timeblock', () => {
  // ============================================================
  // STATE - 只存储最原始、最规范化的数据
  // ============================================================

  /**
   * 时间块映射表
   * key: timeblock_id
   */
  const timeBlocks = ref(new Map<string, TimeBlockView>())

  /**
   * 加载状态
   */
  const isLoading = ref(false)

  /**
   * 错误信息
   */
  const error = ref<string | null>(null)

  // ============================================================
  // GETTERS - 只负责从State中读取和计算数据
  // ============================================================

  /**
   * 获取所有时间块（按开始时间排序）
   */
  const allTimeBlocks = computed(() => {
    return Array.from(timeBlocks.value.values()).sort(
      (a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime()
    )
  })

  /**
   * 根据 ID 获取时间块
   */
  function getTimeBlockById(id: string): TimeBlockView | undefined {
    return timeBlocks.value.get(id)
  }

  /**
   * 获取指定日期的时间块列表
   */
  const getTimeBlocksForDate = computed(() => {
    return (date: string): TimeBlockView[] => {
      // date: YYYY-MM-DD 字符串，直接比较日期部分
      return Array.from(timeBlocks.value.values())
        .filter((block) => {
          const blockDate = block.start_time.split('T')[0] // 提取 YYYY-MM-DD
          return blockDate === date
        })
        .sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime())
    }
  })

  /**
   * 获取指定时间范围的时间块列表
   */
  const getTimeBlocksInRange = computed(() => {
    return (startTime: string, endTime: string): TimeBlockView[] => {
      const start = new Date(startTime).getTime()
      const end = new Date(endTime).getTime()

      return Array.from(timeBlocks.value.values())
        .filter((block) => {
          const blockStart = new Date(block.start_time).getTime()
          const blockEnd = new Date(block.end_time).getTime()
          // 检查是否有时间重叠
          return blockStart < end && blockEnd > start
        })
        .sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime())
    }
  })

  /**
   * 根据区域 ID 获取时间块列表
   */
  const getTimeBlocksByArea = computed(() => {
    return (areaId: string): TimeBlockView[] => {
      return Array.from(timeBlocks.value.values())
        .filter((block) => block.area_id === areaId)
        .sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime())
    }
  })

  /**
   * 获取包含指定任务的时间块列表
   */
  const getTimeBlocksWithTask = computed(() => {
    return (taskId: string): TimeBlockView[] => {
      return Array.from(timeBlocks.value.values())
        .filter((block) => block.linked_tasks.some((task) => task.id === taskId))
        .sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime())
    }
  })

  // ============================================================
  // ACTIONS - 负责执行操作、调用API、修改State
  // ============================================================

  /**
   * 批量添加或更新时间块
   */
  function addOrUpdateTimeBlocks(blocks: TimeBlockView[]) {
    const newMap = new Map(timeBlocks.value)
    for (const block of blocks) {
      newMap.set(block.id, block)
    }
    timeBlocks.value = newMap
  }

  /**
   * 添加或更新单个时间块
   */
  function addOrUpdateTimeBlock(block: TimeBlockView) {
    const newMap = new Map(timeBlocks.value)
    newMap.set(block.id, block)
    timeBlocks.value = newMap
  }

  /**
   * 从 state 中移除时间块
   */
  function removeTimeBlock(id: string) {
    const newMap = new Map(timeBlocks.value)
    newMap.delete(id)
    timeBlocks.value = newMap
  }

  // ============================================================
  // MUTATIONS - 纯数据操作（RTL 命名规范）
  // ============================================================

  /**
   * Mutation: 添加或更新时间块（纯数据操作）
   */
  function addOrUpdateTimeBlock_mut(block: TimeBlockView) {
    addOrUpdateTimeBlock(block)
    logger.debug(LogTags.STORE_TIMEBLOCK, `TimeBlock ${block.id} added/updated in store`)
  }

  /**
   * Mutation: 批量添加或更新时间块
   */
  function batchAddOrUpdateTimeBlocks_mut(blocks: TimeBlockView[]) {
    addOrUpdateTimeBlocks(blocks)
    logger.debug(LogTags.STORE_TIMEBLOCK, `Batch updated ${blocks.length} time blocks`)
  }

  /**
   * Mutation: 移除时间块
   */
  function removeTimeBlock_mut(id: string) {
    removeTimeBlock(id)
    logger.debug(LogTags.STORE_TIMEBLOCK, `TimeBlock ${id} removed from store`)
  }

  /**
   * Mutation: 批量移除时间块
   */
  function batchRemoveTimeBlocks_mut(ids: string[]) {
    const newMap = new Map(timeBlocks.value)
    for (const id of ids) {
      newMap.delete(id)
    }
    timeBlocks.value = newMap
    logger.debug(LogTags.STORE_TIMEBLOCK, `Batch removed ${ids.length} time blocks`)
  }

  /**
   * 获取指定日期的时间块
   * API: GET /time-blocks?date=YYYY-MM-DD
   */
  async function fetchTimeBlocksForDate(date: string): Promise<TimeBlockView[]> {
    isLoading.value = true
    error.value = null

    try {
      // TODO: 实现 API 调用
      // const apiBaseUrl = await waitForApiReady()
      // const response = await fetch(`${apiBaseUrl}/time-blocks?date=${date}`)
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)
      // const blocks: TimeBlockView[] = await response.json()
      // addOrUpdateTimeBlocks(blocks)
      // return blocks

      logger.info(LogTags.STORE_TIMEBLOCK, 'fetchTimeBlocksForDate - API not implemented yet', {
        date,
      })
      return []
    } catch (e) {
      error.value = `Failed to fetch time blocks for ${date}: ${e}`
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Error fetching time blocks',
        e instanceof Error ? e : new Error(String(e)),
        { date }
      )
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 获取日期范围内的时间块
   * API: GET /time-blocks?start_date=...&end_date=...
   */
  async function fetchTimeBlocksForRange(
    startDate: string,
    endDate: string
  ): Promise<TimeBlockView[]> {
    isLoading.value = true
    error.value = null

    try {
      const params = new URLSearchParams()
      params.append('start_date', startDate)
      params.append('end_date', endDate)
      const blocks: TimeBlockView[] = await apiGet(`/time-blocks?${params}`)
      addOrUpdateTimeBlocks(blocks)
      logger.info(LogTags.STORE_TIMEBLOCK, 'Fetched time blocks for range', {
        count: blocks.length,
        startDate,
        endDate,
      })
      return blocks
    } catch (e) {
      error.value = `Failed to fetch time blocks for range: ${e}`
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Error fetching time blocks for range',
        e instanceof Error ? e : new Error(String(e)),
        { startDate, endDate }
      )
      return []
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 从任务创建时间块（拖动场景专用）
   * API: POST /time-blocks/from-task
   */
  async function createTimeBlockFromTask(
    payload: CreateFromTaskPayload
  ): Promise<CreateFromTaskResponse | null> {
    isLoading.value = true
    error.value = null
    logger.info(LogTags.STORE_TIMEBLOCK, 'Creating time block from task', { payload })

    try {
      const data: CreateFromTaskResponse = await apiPost('/time-blocks/from-task', payload)

      // 更新时间块
      addOrUpdateTimeBlock(data.time_block)
      logger.info(LogTags.STORE_TIMEBLOCK, 'Created time block from task', {
        timeBlockId: data.time_block.id,
        taskId: payload.task_id,
      })

      return data // 返回完整响应，包含 updated_task
    } catch (e) {
      error.value = `Failed to create time block from task: ${e}`
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Error creating time block from task',
        e instanceof Error ? e : new Error(String(e)),
        { payload }
      )
      throw e
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 创建空时间块（直接在日历上创建）
   * API: POST /time-blocks
   */
  async function createTimeBlock(payload: CreateTimeBlockPayload): Promise<TimeBlockView | null> {
    isLoading.value = true
    error.value = null
    logger.info(LogTags.STORE_TIMEBLOCK, 'Creating time block', { payload })

    try {
      const newBlock: TimeBlockView = await apiPost('/time-blocks', payload)
      addOrUpdateTimeBlock(newBlock)
      logger.info(LogTags.STORE_TIMEBLOCK, 'Created time block', {
        timeBlockId: newBlock.id,
        title: newBlock.title,
      })
      return newBlock
    } catch (e) {
      error.value = `Failed to create time block: ${e}`
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Error creating time block',
        e instanceof Error ? e : new Error(String(e)),
        { payload }
      )
      throw e // 重新抛出错误，让调用者处理
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 更新时间块
   * API: PATCH /time-blocks/:id
   */
  async function updateTimeBlock(
    id: string,
    payload: UpdateTimeBlockPayload
  ): Promise<TimeBlockView | null> {
    isLoading.value = true
    error.value = null
    logger.info(LogTags.STORE_TIMEBLOCK, 'Updating time block', { id, payload })

    try {
      const updatedBlock: TimeBlockView = await apiPatch(`/time-blocks/${id}`, payload)
      addOrUpdateTimeBlock(updatedBlock)
      logger.info(LogTags.STORE_TIMEBLOCK, 'Updated time block', {
        timeBlockId: updatedBlock.id,
        title: updatedBlock.title,
      })
      return updatedBlock
    } catch (e) {
      error.value = `Failed to update time block ${id}: ${e}`
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Error updating time block',
        e instanceof Error ? e : new Error(String(e)),
        { id, payload }
      )
      throw e // 重新抛出错误，让调用者处理
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 删除时间块
   * API: DELETE /time-blocks/:id
   */
  async function deleteTimeBlock(id: string): Promise<{ updated_tasks?: import('@/types/dtos').TaskCard[] } | null> {
    isLoading.value = true
    error.value = null

    try {
      // 接收包含副作用的响应
      const response = await apiDelete(`/time-blocks/${id}`)
      removeTimeBlock(id)
      logger.info(LogTags.STORE_TIMEBLOCK, 'Deleted time block', { timeBlockId: id })

      // 返回副作用数据供调用者处理
      return response?.side_effects || null
    } catch (e) {
      error.value = `Failed to delete time block ${id}: ${e}`
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Error deleting time block',
        e instanceof Error ? e : new Error(String(e)),
        { timeBlockId: id }
      )
      return null
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 将任务链接到时间块
   * API: POST /time-blocks/:id/tasks
   */
  async function linkTaskToBlock(blockId: string, taskId: string): Promise<boolean> {
    isLoading.value = true
    error.value = null
    logger.info(LogTags.STORE_TIMEBLOCK, 'Linking task to block', { taskId, blockId })

    try {
      // TODO: 实现 API 调用
      // const apiBaseUrl = await waitForApiReady()
      // const response = await fetch(`${apiBaseUrl}/time-blocks/${blockId}/tasks`, {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify({ task_id: taskId })
      // })
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)

      // // 重新获取该时间块以更新链接的任务列表
      // const block = timeBlocks.value.get(blockId)
      // if (block) {
      //   const date = new Date(block.start_time).toISOString().split('T')[0]
      //   await fetchTimeBlocksForDate(date)
      // }
      // return true

      logger.info(LogTags.STORE_TIMEBLOCK, 'linkTaskToBlock - API not implemented yet', {
        taskId,
        blockId,
      })
      return false
    } catch (e) {
      error.value = `Failed to link task to block: ${e}`
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Error linking task to block',
        e instanceof Error ? e : new Error(String(e)),
        { taskId, blockId }
      )
      return false
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 从时间块取消链接任务
   * API: DELETE /time-blocks/:id/tasks/:task_id
   */
  async function unlinkTaskFromBlock(blockId: string, taskId: string): Promise<boolean> {
    isLoading.value = true
    error.value = null
    logger.info(LogTags.STORE_TIMEBLOCK, 'Unlinking task from block', { taskId, blockId })

    try {
      // TODO: 实现 API 调用
      // const apiBaseUrl = await waitForApiReady()
      // const response = await fetch(`${apiBaseUrl}/time-blocks/${blockId}/tasks/${taskId}`, {
      //   method: 'DELETE'
      // })
      // if (!response.ok) throw new Error(`HTTP ${response.status}`)

      // // 重新获取该时间块以更新链接的任务列表
      // const block = timeBlocks.value.get(blockId)
      // if (block) {
      //   const date = new Date(block.start_time).toISOString().split('T')[0]
      //   await fetchTimeBlocksForDate(date)
      // }
      // return true

      logger.info(LogTags.STORE_TIMEBLOCK, 'unlinkTaskFromBlock - API not implemented yet', {
        taskId,
        blockId,
      })
      return false
    } catch (e) {
      error.value = `Failed to unlink task from block: ${e}`
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Error unlinking task from block',
        e instanceof Error ? e : new Error(String(e)),
        { taskId, blockId }
      )
      return false
    } finally {
      isLoading.value = false
    }
  }

  // ============================================================
  // 事件订阅器 - 处理 SSE 推送的领域事件
  // ============================================================

  /**
   * 统一的副作用处理器
   * ✅ 禁止片面数据：接收完整的 TimeBlockView 对象
   * ✅ 零额外请求：直接使用事件中的数据，无需 HTTP 请求
   */
  async function handleTimeBlockSideEffects(sideEffects: {
    deleted_time_blocks?: TimeBlockView[]
    truncated_time_blocks?: TimeBlockView[]
    updated_time_blocks?: TimeBlockView[]
  }) {
    logger.debug(LogTags.STORE_TIMEBLOCK, 'Handling side effects', { sideEffects })

    // 处理删除的时间块：直接使用完整对象，无需查询
    if (sideEffects.deleted_time_blocks?.length) {
      for (const block of sideEffects.deleted_time_blocks) {
        removeTimeBlock(block.id)
        logger.debug(LogTags.STORE_TIMEBLOCK, 'Removed time block', {
          timeBlockId: block.id,
          title: block.title,
        })
      }
    }

    // 处理截断的时间块：直接更新完整数据，无需 HTTP 请求 ✅
    if (sideEffects.truncated_time_blocks?.length) {
      for (const block of sideEffects.truncated_time_blocks) {
        addOrUpdateTimeBlock(block)
        logger.debug(LogTags.STORE_TIMEBLOCK, 'Updated truncated time block', {
          timeBlockId: block.id,
          endTime: block.end_time,
        })
      }
    }

    // 处理更新的时间块：直接更新完整数据，无需 HTTP 请求 ✅
    if (sideEffects.updated_time_blocks?.length) {
      for (const block of sideEffects.updated_time_blocks) {
        addOrUpdateTimeBlock(block)
        logger.debug(LogTags.STORE_TIMEBLOCK, 'Updated time block', {
          timeBlockId: block.id,
          title: block.title,
          areaId: block.area_id || 'none',
        })
      }
    }
  }

  // ============================================================
  // SSE EVENT HANDLERS - 监听后端事件并自动更新State
  // ============================================================

  /**
   * 初始化SSE事件订阅
   */
  function initEventSubscriptions() {
    const subscriber = getEventSubscriber()
    if (!subscriber) {
      logger.error(LogTags.STORE_TIMEBLOCK, 'EventSubscriber not initialized yet')
      return
    }

    // 订阅时间块创建事件
    subscriber.on('time_blocks.created', handleTimeBlockCreatedEvent)

    // 订阅时间块更新事件
    subscriber.on('time_blocks.updated', handleTimeBlockUpdatedEvent)

    // 订阅时间块删除事件
    subscriber.on('time_blocks.deleted', handleTimeBlockDeletedEvent)

    // 订阅时间块链接事件
    subscriber.on('time_blocks.linked', handleTimeBlockLinkedEvent)

    logger.info(LogTags.STORE_TIMEBLOCK, 'SSE event subscriptions initialized')
  }

  /**
   * 处理时间块创建事件
   */
  async function handleTimeBlockCreatedEvent(event: any) {
    const timeBlock = event.payload?.time_block
    const updatedTask = event.payload?.updated_task

    if (!timeBlock) {
      logger.warn(LogTags.STORE_TIMEBLOCK, 'time_blocks.created event missing time_block')
      return
    }

    logger.info(LogTags.STORE_TIMEBLOCK, 'Handling time_blocks.created event', {
      timeBlockId: timeBlock.id,
    })

    // 更新时间块
    addOrUpdateTimeBlock(timeBlock)

    // ✅ 关键：更新任务（schedule_status 已变化）
    if (updatedTask) {
      const { useTaskStore } = await import('@/stores/task')
      const taskStore = useTaskStore()
      taskStore.addOrUpdateTask_mut(updatedTask)
      logger.debug(LogTags.STORE_TIMEBLOCK, 'Updated task schedule_status', {
        taskId: updatedTask.id,
        scheduleStatus: updatedTask.schedule_status,
      })
    }
  }

  /**
   * 处理时间块更新事件
   */
  async function handleTimeBlockUpdatedEvent(event: any) {
    const timeBlockId = event.payload?.time_block_id
    if (!timeBlockId) return

    logger.info(LogTags.STORE_TIMEBLOCK, 'Handling time_blocks.updated event', { timeBlockId })

    // 重新获取该时间块的完整数据
    try {
      const blocks: TimeBlockView[] = await apiGet(`/time-blocks?ids=${timeBlockId}`)
      const block = blocks[0]
      if (block) {
        addOrUpdateTimeBlock(block)
      }
    } catch (error) {
      logger.error(
        LogTags.STORE_TIMEBLOCK,
        'Failed to fetch time block',
        error instanceof Error ? error : new Error(String(error)),
        { timeBlockId }
      )
    }
  }

  /**
   * 处理时间块删除事件
   */
  function handleTimeBlockDeletedEvent(event: any) {
    const timeBlockId = event.payload?.time_block_id
    if (!timeBlockId) return

    logger.info(LogTags.STORE_TIMEBLOCK, 'Handling time_blocks.deleted event', { timeBlockId })
    removeTimeBlock(timeBlockId)
  }

  /**
   * 处理时间块链接事件（链接任务后，时间块可能继承了任务的area）
   */
  async function handleTimeBlockLinkedEvent(event: any) {
    const timeBlock = event.payload?.time_block
    const updatedTask = event.payload?.updated_task

    if (!timeBlock) {
      logger.warn(LogTags.STORE_TIMEBLOCK, 'time_blocks.linked event missing time_block data')
      return
    }

    logger.info(LogTags.STORE_TIMEBLOCK, 'Handling time_blocks.linked event', {
      timeBlockId: timeBlock.id,
      areaId: timeBlock.area_id,
      taskId: timeBlock.task_id,
    })

    // 直接使用 payload 中的完整数据（包括更新后的 area_id）
    addOrUpdateTimeBlock(timeBlock)

    // ✅ 关键：更新任务（schedule_status 已变化）
    if (updatedTask) {
      const { useTaskStore } = await import('@/stores/task')
      const taskStore = useTaskStore()
      taskStore.addOrUpdateTask_mut(updatedTask)
      logger.debug(LogTags.STORE_TIMEBLOCK, 'Updated task schedule_status', {
        taskId: updatedTask.id,
        scheduleStatus: updatedTask.schedule_status,
      })
    }
  }

  return {
    // State
    timeBlocks,
    isLoading,
    error,

    // Getters
    allTimeBlocks,
    getTimeBlockById,
    getTimeBlocksForDate,
    getTimeBlocksInRange,
    getTimeBlocksByArea,
    getTimeBlocksWithTask,

    // Actions
    addOrUpdateTimeBlocks,
    addOrUpdateTimeBlock,
    removeTimeBlock,
    fetchTimeBlocksForDate,
    fetchTimeBlocksForRange,
    createTimeBlock,
    createTimeBlockFromTask,
    updateTimeBlock,
    deleteTimeBlock,
    linkTaskToBlock,
    unlinkTaskFromBlock,

    // Event handlers
    handleTimeBlockSideEffects,

    // Initialization
    initEventSubscriptions,

    // Mutations (RTL 命名规范)
    addOrUpdateTimeBlock_mut,
    batchAddOrUpdateTimeBlocks_mut,
    removeTimeBlock_mut,
    batchRemoveTimeBlocks_mut,
  }
})

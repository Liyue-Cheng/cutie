import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { TimeBlockView } from '@/types/dtos'
import { waitForApiReady } from '@/composables/useApiConfig'

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
  area_id?: string | null
}

export interface CreateFromTaskPayload {
  task_id: string
  start_time: string // ISO 8601 UTC
  end_time: string // ISO 8601 UTC
  title?: string | null // 可选，默认使用任务标题
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
      const targetDate = new Date(date).toDateString()
      return Array.from(timeBlocks.value.values())
        .filter((block) => new Date(block.start_time).toDateString() === targetDate)
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
        .filter((block) => block.area?.id === areaId)
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

      console.log('[TimeBlockStore] fetchTimeBlocksForDate - API not implemented yet', { date })
      return []
    } catch (e) {
      error.value = `Failed to fetch time blocks for ${date}: ${e}`
      console.error('[TimeBlockStore] Error fetching time blocks:', e)
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
      const apiBaseUrl = await waitForApiReady()
      const params = new URLSearchParams()
      params.append('start_date', startDate)
      params.append('end_date', endDate)
      const response = await fetch(`${apiBaseUrl}/time-blocks?${params}`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const result = await response.json()
      const blocks: TimeBlockView[] = result.data
      addOrUpdateTimeBlocks(blocks)
      console.log('[TimeBlockStore] Fetched', blocks.length, 'time blocks for range')
      return blocks
    } catch (e) {
      error.value = `Failed to fetch time blocks for range: ${e}`
      console.error('[TimeBlockStore] Error fetching time blocks for range:', e)
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
    console.log('[TimeBlockStore] Creating time block from task:', payload)

    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks/from-task`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!response.ok) {
        const errorData = await response.json()
        console.error('[TimeBlockStore] API error:', errorData)
        throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorData)}`)
      }
      const result = await response.json()
      const data: CreateFromTaskResponse = result.data

      // 更新时间块
      addOrUpdateTimeBlock(data.time_block)
      console.log('[TimeBlockStore] Created time block from task:', data)

      return data // 返回完整响应，包含 updated_task
    } catch (e) {
      error.value = `Failed to create time block from task: ${e}`
      console.error('[TimeBlockStore] Error creating time block from task:', e)
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
    console.log('[TimeBlockStore] Creating time block:', payload)

    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!response.ok) {
        const errorData = await response.json()
        console.error('[TimeBlockStore] API error:', errorData)
        throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorData)}`)
      }
      const result = await response.json()
      const newBlock: TimeBlockView = result.data // 提取 ApiResponse 的 data 字段
      addOrUpdateTimeBlock(newBlock)
      console.log('[TimeBlockStore] Created time block:', newBlock)
      return newBlock
    } catch (e) {
      error.value = `Failed to create time block: ${e}`
      console.error('[TimeBlockStore] Error creating time block:', e)
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
    console.log('[TimeBlockStore] Updating time block', id, ':', payload)

    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
      if (!response.ok) {
        const errorData = await response.json()
        console.error('[TimeBlockStore] API error:', errorData)
        throw new Error(`HTTP ${response.status}: ${JSON.stringify(errorData)}`)
      }
      const result = await response.json()
      const updatedBlock: TimeBlockView = result.data // 提取 ApiResponse 的 data 字段
      addOrUpdateTimeBlock(updatedBlock)
      console.log('[TimeBlockStore] Updated time block:', updatedBlock)
      return updatedBlock
    } catch (e) {
      error.value = `Failed to update time block ${id}: ${e}`
      console.error('[TimeBlockStore] Error updating time block:', e)
      throw e // 重新抛出错误，让调用者处理
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 删除时间块
   * API: DELETE /time-blocks/:id
   */
  async function deleteTimeBlock(id: string): Promise<boolean> {
    isLoading.value = true
    error.value = null

    try {
      const apiBaseUrl = await waitForApiReady()
      const response = await fetch(`${apiBaseUrl}/time-blocks/${id}`, {
        method: 'DELETE',
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      removeTimeBlock(id)
      console.log('[TimeBlockStore] Deleted time block:', id)
      return true
    } catch (e) {
      error.value = `Failed to delete time block ${id}: ${e}`
      console.error('[TimeBlockStore] Error deleting time block:', e)
      return false
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
    console.log('[TimeBlockStore] Linking task', taskId, 'to block', blockId)

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

      console.log('[TimeBlockStore] linkTaskToBlock - API not implemented yet')
      return false
    } catch (e) {
      error.value = `Failed to link task to block: ${e}`
      console.error('[TimeBlockStore] Error linking task to block:', e)
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
    console.log('[TimeBlockStore] Unlinking task', taskId, 'from block', blockId)

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

      console.log('[TimeBlockStore] unlinkTaskFromBlock - API not implemented yet')
      return false
    } catch (e) {
      error.value = `Failed to unlink task from block: ${e}`
      console.error('[TimeBlockStore] Error unlinking task from block:', e)
      return false
    } finally {
      isLoading.value = false
    }
  }

  // ============================================================
  // 事件订阅器 - 处理 SSE 推送的领域事件
  // ============================================================

  /**
   * 占位函数：保持与 TaskStore 的接口兼容
   * 注意：TimeBlockStore 不再独立订阅事件，而是由 TaskStore 调用
   */
  function initEventSubscriptions() {
    // 不再需要单独订阅 time_blocks.deleted 和 time_blocks.truncated
    // 这些副作用现在包含在 task.completed 和 task.deleted 事件中
    console.log('[TimeBlockStore] Event subscriptions managed by TaskStore')
  }

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
    console.log('[TimeBlockStore] Handling side effects:', sideEffects)

    // 处理删除的时间块：直接使用完整对象，无需查询
    if (sideEffects.deleted_time_blocks?.length) {
      for (const block of sideEffects.deleted_time_blocks) {
        removeTimeBlock(block.id)
        console.log(`[TimeBlockStore] Removed time block: ${block.id} ("${block.title}")`)
      }
    }

    // 处理截断的时间块：直接更新完整数据，无需 HTTP 请求 ✅
    if (sideEffects.truncated_time_blocks?.length) {
      for (const block of sideEffects.truncated_time_blocks) {
        addOrUpdateTimeBlock(block)
        console.log(
          `[TimeBlockStore] Updated truncated time block: ${block.id} (end_time: ${block.end_time})`
        )
      }
    }

    // 处理更新的时间块：直接更新完整数据，无需 HTTP 请求 ✅
    if (sideEffects.updated_time_blocks?.length) {
      for (const block of sideEffects.updated_time_blocks) {
        addOrUpdateTimeBlock(block)
        console.log(
          `[TimeBlockStore] Updated time block: ${block.id} (title: "${block.title}", area: ${block.area?.name || 'none'})`
        )
      }
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
    initEventSubscriptions,
    handleTimeBlockSideEffects,
  }
})

/**
 * useCrossViewDragTarget - 跨看板拖放目标逻辑
 *
 * 封装作为跨看板拖放目标的所有逻辑，包括：
 * - dragenter/dragleave 处理
 * - 幽灵元素插入
 * - 容器级定位
 */

import { ref, computed, watch } from 'vue'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata } from '@/types/drag'
import { useCrossViewDrag } from './useCrossViewDrag'
import { logger, LogTags } from '@/services/logger'

/**
 * 跨看板拖放目标 Composable
 * @param viewMetadata - 当前看板的元数据
 */
export function useCrossViewDragTarget(viewMetadata: ViewMetadata) {
  const crossViewDrag = useCrossViewDrag()

  // ==================== 状态 ====================

  /** 从其他看板拖入的任务 */
  const draggedTask = ref<TaskCard | null>(null)

  /** 是否正在接收跨看板拖放 */
  const isReceivingDrag = ref(false)

  /** 目标插入索引 */
  const targetIndex = ref<number | null>(null)

  /** 进入深度计数（用于稳定 dragenter/dragleave） */
  const enterDepth = ref(0)

  /** 节流控制 */
  let lastDragOverTime = 0
  const DRAG_THROTTLE_MS = 50

  // ==================== 计算属性 ====================

  /**
   * 是否有活动的跨看板拖放
   */
  const hasActiveDrag = computed(() => {
    const context = crossViewDrag.currentContext.value
    return context !== null && context.sourceView.id !== viewMetadata.id
  })

  // ==================== 监听器 ====================

  /**
   * 监听全局目标看板变化，若目标离开本列则清理状态
   */
  watch(
    () => crossViewDrag.targetViewId.value,
    (newId) => {
      if (newId !== viewMetadata.id && isReceivingDrag.value) {
        logger.debug(LogTags.DRAG_CROSS_VIEW, 'Target moved away, clearing state')
        clearReceivingState()
      }
    }
  )

  // ==================== 操作方法 ====================

  /**
   * 处理 dragenter 事件
   */
  function handleEnter(event: DragEvent): void {
    event.preventDefault()

    if (!hasActiveDrag.value) return

    const context = crossViewDrag.currentContext.value!

    // 进入深度计数，避免子元素切换导致抖动
    enterDepth.value += 1

    if (enterDepth.value === 1) {
      logger.debug(LogTags.DRAG_CROSS_VIEW, 'Cross-view drag entered', {
        from: context.sourceView.id,
        to: viewMetadata.id,
        task: context.task.title,
      })

      // 设置全局目标看板ID
      crossViewDrag.setTargetViewId(viewMetadata.id)

      // 激活接收模式
      isReceivingDrag.value = true
      draggedTask.value = context.task
      targetIndex.value = null // 等待第一次 dragover
    }
  }

  /**
   * 处理 dragleave 事件
   */
  function handleLeave(event: DragEvent): void {
    if (!hasActiveDrag.value) return

    // 优先基于几何判断是否真正离开容器
    const container = event.currentTarget as HTMLElement
    const rect = container.getBoundingClientRect()
    const x = event.clientX
    const y = event.clientY
    const reallyLeft = x < rect.left || x > rect.right || y < rect.top || y > rect.bottom

    // 维持原有深度计数，兼容子元素切换
    enterDepth.value = Math.max(0, enterDepth.value - 1)

    if (reallyLeft || enterDepth.value === 0) {
      logger.debug(LogTags.DRAG_CROSS_VIEW, 'Cross-view drag left', {
        viewId: viewMetadata.id,
        reallyLeft,
        depth: enterDepth.value,
      })

      clearReceivingState()

      // 清理全局目标
      if (crossViewDrag.targetViewId.value === viewMetadata.id) {
        crossViewDrag.setTargetViewId(null)
      }
    }
  }

  /**
   * 容器级 dragover：根据鼠标 Y 定位插入位置
   * @param event - DragEvent
   * @param wrappers - 任务卡片包装器元素列表
   */
  function handleContainerDragOver(event: DragEvent, wrappers: HTMLElement[]): void {
    if (!isReceivingDrag.value) return

    // 节流
    const now = Date.now()
    if (now - lastDragOverTime < DRAG_THROTTLE_MS) {
      return
    }
    lastDragOverTime = now

    const mouseY = event.clientY

    // 忽略幽灵元素自身（防止自我影响引起抖动）
    const ghostId = draggedTask.value?.id || null
    const candidates = ghostId
      ? wrappers.filter((el) => (el.dataset.taskId || '') !== ghostId)
      : wrappers

    // 计算插入索引：第一个"中心点"在鼠标之下的元素索引
    let index = candidates.length
    for (let i = 0; i < candidates.length; i++) {
      const el = candidates[i]
      if (!el) continue
      const rect = el.getBoundingClientRect()
      const centerY = rect.top + rect.height / 2
      if (mouseY < centerY) {
        index = i
        break
      }
    }

    targetIndex.value = index
  }

  /**
   * 处理 drop 事件
   * @param event - DragEvent
   * @returns 处理结果
   */
  async function handleDrop(event: DragEvent): Promise<{
    isHandled: boolean
    success?: boolean
    error?: string
    taskId?: string
  }> {
    const context = crossViewDrag.currentContext.value

    // 检查是否是跨看板拖放
    if (!context || context.sourceView.id === viewMetadata.id) {
      return { isHandled: false }
    }

    logger.debug(LogTags.DRAG_CROSS_VIEW, 'Cross-view drop detected')

    // 调用跨看板拖放框架
    const result = await crossViewDrag.handleDrop(viewMetadata, event)

    // 清理状态
    clearReceivingState()
    crossViewDrag.setTargetViewId(null)

    if (result.success) {
      logger.info(LogTags.DRAG_CROSS_VIEW, 'Cross-view drop success', { message: result.message })
      return {
        isHandled: true,
        success: true,
        taskId: context.task.id,
      }
    } else {
      logger.error(
        LogTags.DRAG_CROSS_VIEW,
        'Cross-view drop failed',
        new Error(result.error || 'Unknown error')
      )
      return {
        isHandled: true,
        success: false,
        error: result.error,
      }
    }
  }

  /**
   * 获取包含幽灵元素的任务列表
   * @param tasks - 原始任务列表
   * @returns 包含幽灵元素的任务列表
   */
  function getTasksWithGhost(tasks: TaskCard[]): TaskCard[] {
    // 只有在接收跨看板拖放，且全局目标确认为本列时才添加幽灵元素
    if (
      !isReceivingDrag.value ||
      !draggedTask.value ||
      crossViewDrag.targetViewId.value !== viewMetadata.id
    ) {
      return tasks
    }

    const taskList = [...tasks]
    const existingIndex = taskList.findIndex((t) => t.id === draggedTask.value!.id)

    // 外来任务不在列表中，添加它
    if (existingIndex === -1) {
      if (targetIndex.value !== null) {
        taskList.splice(targetIndex.value, 0, draggedTask.value)
      } else {
        taskList.push(draggedTask.value)
      }
    }

    return taskList
  }

  /**
   * 清理接收状态
   */
  function clearReceivingState(): void {
    isReceivingDrag.value = false
    draggedTask.value = null
    targetIndex.value = null
    enterDepth.value = 0
    lastDragOverTime = 0
  }

  return {
    // 状态（只读）
    isReceivingDrag: computed(() => isReceivingDrag.value),
    draggedTask: computed(() => draggedTask.value),
    targetIndex: computed(() => targetIndex.value),
    hasActiveDrag,

    // 操作方法
    handleEnter,
    handleLeave,
    handleContainerDragOver,
    handleDrop,
    getTasksWithGhost,
    clearReceivingState,
  }
}

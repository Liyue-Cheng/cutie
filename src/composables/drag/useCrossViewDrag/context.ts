/**
 * useCrossViewDrag/context - 拖拽上下文管理
 *
 * 管理当前拖拽的上下文信息（全局单例）
 */

import { ref, readonly, computed } from 'vue'
import type { Ref } from 'vue'
import type { DragContext, ViewMetadata, NormalDragMode, SnapDragMode } from '@/types/drag'
import type { TaskCard } from '@/types/dtos'

// ==================== 全局上下文状态 ====================

const currentContext = ref<DragContext | null>(null)
// 🆕 当前目标看板ID（用于源看板判断是否应隐藏幽灵元素）
const currentTargetViewId = ref<string | null>(null)
// 🆕 标记：是否正处于 drop 执行中（用于避免 dragend 过早清理）
const dropInProgress = ref<boolean>(false)

/**
 * 拖拽上下文管理
 */
export function useDragContext() {
  /**
   * 是否正在拖拽
   */
  const isDragging = computed(() => currentContext.value !== null)

  /**
   * 当前拖放模式
   */
  const currentMode = computed(() => currentContext.value?.dragMode.mode || null)

  /**
   * 当前拖拽的任务
   */
  const currentTask = computed(() => currentContext.value?.task || null)

  /**
   * 源看板
   */
  const sourceView = computed(() => currentContext.value?.sourceView || null)

  /**
   * 🆕 当前目标看板ID
   */
  const targetViewId = computed(() => currentTargetViewId.value)

  /**
   * 🆕 是否处于 drop 执行中
   */
  const isDropInProgress = computed(() => dropInProgress.value)

  /**
   * 开始普通拖放
   * @param task - 被拖拽的任务
   * @param sourceView - 源看板元数据
   */
  function startNormalDrag(task: TaskCard, sourceView: ViewMetadata): void {
    const dragMode: NormalDragMode = { mode: 'normal' }

    currentContext.value = {
      task,
      sourceView,
      dragMode,
      startTime: Date.now(),
    }

    console.log('[DragContext] 🚀 Started normal drag:', {
      taskId: task.id,
      taskTitle: task.title,
      sourceViewType: sourceView.type,
      sourceViewId: sourceView.id,
    })
  }

  /**
   * 开始吸附式拖放
   * @param task - 被拖拽的任务
   * @param sourceView - 源看板元数据
   * @param activatedBy - 激活按钮的标识
   * @param params - 额外参数
   */
  function startSnapDrag(
    task: TaskCard,
    sourceView: ViewMetadata,
    activatedBy: string,
    params?: Record<string, any>
  ): void {
    const dragMode: SnapDragMode = {
      mode: 'snap',
      activatedBy,
      params,
    }

    currentContext.value = {
      task,
      sourceView,
      dragMode,
      startTime: Date.now(),
    }

    console.log('[DragContext] 📍 Started snap drag:', {
      taskId: task.id,
      taskTitle: task.title,
      sourceViewType: sourceView.type,
      sourceViewId: sourceView.id,
      activatedBy,
      params,
    })
  }

  /**
   * 更新拖拽上下文的元数据
   * @param metadata - 附加元数据
   */
  function updateMetadata(metadata: Record<string, any>): void {
    if (!currentContext.value) {
      console.warn('[DragContext] Cannot update metadata: no active drag context')
      return
    }

    currentContext.value = {
      ...currentContext.value,
      metadata: {
        ...currentContext.value.metadata,
        ...metadata,
      },
    }

    console.log('[DragContext] Updated metadata:', metadata)
  }

  /**
   * 🆕 设置当前目标看板ID
   */
  function setTargetViewId(viewId: string | null): void {
    currentTargetViewId.value = viewId
    if (viewId) {
      console.log('[DragContext] 🎯 Target view changed:', viewId)
    }
  }

  /**
   * 🆕 设置 drop 执行中标记
   */
  function setDropInProgress(inProgress: boolean): void {
    dropInProgress.value = inProgress
  }

  /**
   * 清除拖拽上下文
   */
  function clearContext(): void {
    if (!currentContext.value) return

    const duration = Date.now() - currentContext.value.startTime

    console.log('[DragContext] ✅ Cleared context:', {
      duration: `${duration}ms`,
      mode: currentContext.value.dragMode.mode,
    })

    currentContext.value = null
    currentTargetViewId.value = null // 🆕 同时清理目标看板ID
    dropInProgress.value = false // 🆕 确保复位
  }

  /**
   * 获取拖拽持续时间
   * @returns 持续时间（毫秒），如果未拖拽则返回 0
   */
  function getDragDuration(): number {
    if (!currentContext.value) return 0
    return Date.now() - currentContext.value.startTime
  }

  return {
    // 只读状态
    currentContext: readonly(currentContext) as Readonly<Ref<DragContext | null>>,
    isDragging: readonly(isDragging),
    currentMode: readonly(currentMode),
    currentTask: readonly(currentTask),
    sourceView: readonly(sourceView),
    targetViewId: readonly(targetViewId), // 🆕 导出目标看板ID
    isDropInProgress: readonly(isDropInProgress), // 🆕 导出 drop 状态

    // 操作方法
    startNormalDrag,
    startSnapDrag,
    updateMetadata,
    setTargetViewId, // 🆕 导出设置方法
    setDropInProgress, // 🆕 导出设置方法
    clearContext,
    getDragDuration,
  }
}

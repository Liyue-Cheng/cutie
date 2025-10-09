/**
 * useDragState - 拖拽状态管理
 *
 * 管理全局拖拽状态（单例模式）
 */

import { ref, readonly } from 'vue'
import type { Ref } from 'vue'
import { logger, LogTags } from '@/services/logger'

// ==================== 全局状态 ====================
// 使用单例模式，确保整个应用共享同一个状态

const isDragging = ref(false)
const draggedItem = ref<any>(null)
const dragStartTime = ref<number>(0)

/**
 * 拖拽状态管理
 */
export function useDragState<T = any>() {
  /**
   * 开始拖拽
   * @param item - 被拖拽的项目
   */
  function startDrag(item: T): void {
    isDragging.value = true
    draggedItem.value = item
    dragStartTime.value = Date.now()

    logger.debug(LogTags.DRAG_CROSS_VIEW, 'Drag started', {
      itemType: typeof item,
      timestamp: dragStartTime.value,
    })
  }

  /**
   * 结束拖拽
   */
  function endDrag(): void {
    const duration = Date.now() - dragStartTime.value

    logger.debug(LogTags.DRAG_CROSS_VIEW, 'Drag ended', {
      duration: `${duration}ms`,
    })

    isDragging.value = false
    draggedItem.value = null
    dragStartTime.value = 0
  }

  /**
   * 更新拖拽项
   * @param item - 新的拖拽项
   */
  function updateDraggedItem(item: T): void {
    draggedItem.value = item

    logger.debug(LogTags.DRAG_CROSS_VIEW, 'Dragged item updated')
  }

  /**
   * 获取拖拽持续时间
   * @returns 持续时间（毫秒），如果未拖拽则返回 0
   */
  function getDragDuration(): number {
    if (!isDragging.value) return 0
    return Date.now() - dragStartTime.value
  }

  return {
    // 只读状态
    isDragging: readonly(isDragging) as Readonly<Ref<boolean>>,
    draggedItem: readonly(draggedItem) as Readonly<Ref<T | null>>,

    // 操作方法
    startDrag,
    endDrag,
    updateDraggedItem,
    getDragDuration,
  }
}

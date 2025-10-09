/**
 * useDragTransfer - HTML5 拖放数据传递工具
 *
 * 封装 dataTransfer API，提供类型安全的数据传递
 */

import type { DragTransferData } from '@/types/drag'
import { logger, LogTags } from '@/services/logger'

const DRAG_DATA_TYPE = 'application/x-cutie-task'

/**
 * 数据传递工具
 */
export function useDragTransfer() {
  /**
   * 设置拖拽数据
   * @param event - DragEvent
   * @param data - 要传递的数据
   */
  function setDragData(event: DragEvent, data: DragTransferData): void {
    if (!event.dataTransfer) {
      logger.warn(LogTags.DRAG_CROSS_VIEW, 'dataTransfer is null')
      return
    }

    try {
      const jsonString = JSON.stringify(data)
      event.dataTransfer.setData(DRAG_DATA_TYPE, jsonString)
      event.dataTransfer.effectAllowed = 'copyMove' // ✅ 修复：允许 copy 和 move

      logger.debug(LogTags.DRAG_CROSS_VIEW, 'Drag data set', {
        type: data.type,
        taskId: data.task.id,
        sourceView: data.sourceView.id,
        dragMode: data.dragMode.mode,
      })

      // 🔍 检查点1：effectAllowed/dropEffect 匹配
      logger.debug(LogTags.DRAG_CROSS_VIEW, 'Effect allowed and types', {
        effectAllowed: event.dataTransfer.effectAllowed,
        types: Array.from(event.dataTransfer.types),
      })
    } catch (error) {
      logger.error(
        LogTags.DRAG_CROSS_VIEW,
        'Failed to set drag data',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  }

  /**
   * 获取拖拽数据
   * @param event - DragEvent
   * @returns 解析后的数据，如果失败返回 null
   */
  function getDragData(event: DragEvent): DragTransferData | null {
    if (!event.dataTransfer) {
      logger.warn(LogTags.DRAG_CROSS_VIEW, 'dataTransfer is null')
      return null
    }

    try {
      const jsonString = event.dataTransfer.getData(DRAG_DATA_TYPE)
      if (!jsonString) {
        logger.warn(LogTags.DRAG_CROSS_VIEW, 'No drag data found')
        return null
      }

      const data = JSON.parse(jsonString) as DragTransferData

      logger.debug(LogTags.DRAG_CROSS_VIEW, 'Drag data retrieved', {
        type: data.type,
        taskId: data.task.id,
        sourceView: data.sourceView.id,
        dragMode: data.dragMode.mode,
      })

      return data
    } catch (error) {
      logger.error(
        LogTags.DRAG_CROSS_VIEW,
        'Failed to get drag data',
        error instanceof Error ? error : new Error(String(error))
      )
      return null
    }
  }

  /**
   * 清除拖拽数据
   * @param event - DragEvent
   */
  function clearDragData(event: DragEvent): void {
    if (!event.dataTransfer) return

    try {
      event.dataTransfer.clearData()
      logger.debug(LogTags.DRAG_CROSS_VIEW, 'Drag data cleared')
    } catch (error) {
      logger.error(
        LogTags.DRAG_CROSS_VIEW,
        'Failed to clear drag data',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  }

  /**
   * 检查是否有拖拽数据
   * @param event - DragEvent
   * @returns 是否包含有效数据
   */
  function hasDragData(event: DragEvent): boolean {
    if (!event.dataTransfer) return false

    const types = Array.from(event.dataTransfer.types)
    return types.includes(DRAG_DATA_TYPE)
  }

  return {
    setDragData,
    getDragData,
    clearDragData,
    hasDragData,
  }
}

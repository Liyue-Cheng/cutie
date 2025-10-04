/**
 * useDragTransfer - HTML5 拖放数据传递工具
 *
 * 封装 dataTransfer API，提供类型安全的数据传递
 */

import type { DragTransferData } from '@/types/drag'

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
      console.warn('[useDragTransfer] dataTransfer is null')
      return
    }

    try {
      const jsonString = JSON.stringify(data)
      event.dataTransfer.setData(DRAG_DATA_TYPE, jsonString)
      event.dataTransfer.effectAllowed = 'copyMove' // ✅ 修复：允许 copy 和 move

      console.log('[useDragTransfer] Data set:', {
        type: data.type,
        taskId: data.task.id,
        sourceView: data.sourceView.id,
        dragMode: data.dragMode.mode,
      })

      // 🔍 检查点1：effectAllowed/dropEffect 匹配
      console.log(
        '[CHK-1] effectAllowed=',
        event.dataTransfer.effectAllowed,
        'types=',
        Array.from(event.dataTransfer.types)
      )
    } catch (error) {
      console.error('[useDragTransfer] Failed to set drag data:', error)
    }
  }

  /**
   * 获取拖拽数据
   * @param event - DragEvent
   * @returns 解析后的数据，如果失败返回 null
   */
  function getDragData(event: DragEvent): DragTransferData | null {
    if (!event.dataTransfer) {
      console.warn('[useDragTransfer] dataTransfer is null')
      return null
    }

    try {
      const jsonString = event.dataTransfer.getData(DRAG_DATA_TYPE)
      if (!jsonString) {
        console.warn('[useDragTransfer] No data found')
        return null
      }

      const data = JSON.parse(jsonString) as DragTransferData

      console.log('[useDragTransfer] Data retrieved:', {
        type: data.type,
        taskId: data.task.id,
        sourceView: data.sourceView.id,
        dragMode: data.dragMode.mode,
      })

      return data
    } catch (error) {
      console.error('[useDragTransfer] Failed to get drag data:', error)
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
      console.log('[useDragTransfer] Data cleared')
    } catch (error) {
      console.error('[useDragTransfer] Failed to clear drag data:', error)
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

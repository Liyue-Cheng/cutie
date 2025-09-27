import { computed } from 'vue'
import { manager } from './drag-coordinator'
import type { DraggableOptions } from './types'

/**
 * 可拖拽 composable
 * 提供拖拽状态和逻辑
 */
export function useDraggable(options: DraggableOptions) {
  /**
   * 当前是否正在拖拽此项
   * 通过比较拖拽数据来确定是否是当前项被拖拽
   */
  const isDragging = computed(() => {
    return manager.state.value.isDragging && 
           manager.state.value.dragData === options.data
  })

  /**
   * 启动拖拽的处理函数
   * 供指令调用
   */
  const startDrag = (event: PointerEvent) => {
    manager.startDragByEvent(options, event)
  }

  return {
    isDragging,
    startDrag,
  }
}

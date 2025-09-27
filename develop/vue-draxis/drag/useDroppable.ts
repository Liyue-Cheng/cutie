import { computed } from 'vue'
import { manager } from './drag-coordinator'
import type { DroppableOptions } from './types'

/**
 * 可放置区域 composable
 * 提供放置状态和逻辑
 */
export function useDroppable(options: DroppableOptions) {
  /**
   * 当前是否有拖拽项悬停在此放置区上方
   */
  const isOver = computed(() => {
    return manager.state.value.activeDroppable === options
  })

  /**
   * 当前拖拽的数据类型是否被此放置区接受
   */
  const canAccept = computed(() => {
    const currentDataType = manager.state.value.dataType
    return currentDataType ? options.acceptedDataTypes.includes(currentDataType) : false
  })

  /**
   * 是否正在拖拽且可以接受当前拖拽项
   */
  const isValidDropTarget = computed(() => {
    return manager.state.value.isDragging && canAccept.value
  })

  /**
   * 注册放置区的处理函数
   * 供指令调用
   */
  const registerDropzone = (element: HTMLElement) => {
    manager.registerDroppable(element, options)
  }

  /**
   * 注销放置区的处理函数
   * 供指令调用
   */
  const unregisterDropzone = (element: HTMLElement) => {
    manager.unregisterDroppable(element)
  }

  return {
    isOver,
    canAccept,
    isValidDropTarget,
    registerDropzone,
    unregisterDropzone,
  }
}

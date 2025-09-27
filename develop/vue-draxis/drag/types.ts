import type { Component } from 'vue'

/**
 * 拖放数据类型定义
 */
export interface DragData {
  [key: string]: any
}

/**
 * 拖放坐标位置
 */
export interface Position {
  x: number
  y: number
}

/**
 * 放置区配置选项
 */
export interface DroppableOptions {
  acceptedDataTypes: string[]
  onDrop?: (data: DragData, dataType: string) => void
  onDragEnter?: (data: DragData, dataType: string) => void
  onDragOver?: (data: DragData, dataType: string, event?: PointerEvent) => void
  onDragLeave?: () => void
}

/**
 * 可拖拽元素配置选项
 */
export interface DraggableOptions {
  data: DragData
  dataType: string
  ghostComponent?: Component
  ghostProps?: Record<string, any>
}

/**
 * 拖拽创建器配置选项
 */
export interface DragCreatorOptions {
  createData: () => DragData
  dataType: string
  ghostComponent?: Component
  ghostProps?: Record<string, any> | ((data: DragData) => Record<string, any>)
}

/**
 * 源元素快照信息
 */
export interface ElementSnapshot {
  width: number
  height: number
  innerHTML: string
  // 元素在页面中的位置
  boundingRect: {
    left: number
    top: number
    width: number
    height: number
  }
  computedStyle: {
    backgroundColor: string
    color: string
    fontSize: string
    fontFamily: string
    borderRadius: string
    padding: string
    border: string
    boxShadow: string
    display: string
    alignItems: string
    justifyContent: string
    gap: string
  }
}

/**
 * 拖放状态接口
 */
export interface DragState {
  isDragging: boolean
  dragData: DragData | null
  dataType: string | null
  sourceElement: HTMLElement | null
  currentPosition: Position
  activeDroppable: DroppableOptions | null
  ghostComponent: Component | null
  ghostProps: Record<string, any>
  // 源元素的样式快照
  sourceElementSnapshot: ElementSnapshot | null
  // 鼠标相对于源元素的偏移量
  mouseOffset: Position
  // 拖拽准备状态（已按下但未达到拖拽阈值）
  isPreparing: boolean
  // 初始按下位置
  initialPosition: Position
}

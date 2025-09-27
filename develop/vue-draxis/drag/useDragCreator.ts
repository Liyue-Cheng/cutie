import { manager } from './drag-coordinator'
import type { DragCreatorOptions, Position } from './types'

/**
 * 拖拽创建器 composable
 * 用于程序化创建并拖拽新项
 */
export function useDragCreator(options: DragCreatorOptions) {
  /**
   * 启动程序化拖拽
   * @param position 可选的初始位置，如果不提供则使用鼠标当前位置
   */
  const startDrag = (position?: Position) => {
    // 创建数据
    const data = options.createData()

    // 计算 ghostProps
    let ghostProps: Record<string, any> = {}
    if (options.ghostProps) {
      if (typeof options.ghostProps === 'function') {
        ghostProps = options.ghostProps(data)
      } else {
        ghostProps = options.ghostProps
      }
    }

    // 启动程序化拖拽
    manager.startProgrammaticDrag({
      data,
      dataType: options.dataType,
      ghostComponent: options.ghostComponent,
      ghostProps,
      position,
    })
  }

  /**
   * 从鼠标事件启动程序化拖拽
   * 便捷方法，自动获取鼠标位置
   */
  const startDragFromEvent = (event: MouseEvent | PointerEvent) => {
    startDrag({ x: event.clientX, y: event.clientY })
  }

  return {
    startDrag,
    startDragFromEvent,
  }
}

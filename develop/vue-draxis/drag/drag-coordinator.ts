import { shallowRef, readonly } from 'vue'
import type {
  DragState,
  DraggableOptions,
  DroppableOptions,
  Position,
  ElementSnapshot,
} from './types'

/**
 * 全局拖放状态
 */
const state = shallowRef<DragState>({
  isDragging: false,
  dragData: null,
  dataType: null,
  sourceElement: null,
  currentPosition: { x: 0, y: 0 },
  activeDroppable: null,
  ghostComponent: null,
  ghostProps: {},
  sourceElementSnapshot: null,
  mouseOffset: { x: 0, y: 0 },
  isPreparing: false,
  initialPosition: { x: 0, y: 0 },
})

/**
 * 已注册的放置区映射
 */
const droppables = new Map<HTMLElement, DroppableOptions>()

/**
 * 当前拖拽的清理函数 - 替代全局监听器数组的安全设计
 */
let currentCleanup: () => void = () => {}

/**
 * 拖拽阈值（像素）- 鼠标移动超过此距离才开始拖拽
 */
const DRAG_THRESHOLD = 5

/**
 * 自动滚动相关常量
 */
const AUTO_SCROLL_THRESHOLD = 50 // 距离边缘多少像素开始自动滚动
const AUTO_SCROLL_SPEED = 10 // 滚动速度
let autoScrollInterval: number | null = null
let currentScrollDirection = 0 // 当前滚动方向：-1上，0停止，1下

/**
 * 计算两点之间的距离
 */
function getDistance(pos1: Position, pos2: Position): number {
  const dx = pos2.x - pos1.x
  const dy = pos2.y - pos1.y
  return Math.sqrt(dx * dx + dy * dy)
}

/**
 * 停止自动滚动
 */
function stopAutoScroll() {
  if (autoScrollInterval !== null) {
    clearInterval(autoScrollInterval)
    autoScrollInterval = null
  }
  currentScrollDirection = 0
}

/**
 * 启动自动滚动
 */
function startAutoScroll(direction: number) {
  if (autoScrollInterval !== null) return // 已经在滚动中

  currentScrollDirection = direction
  autoScrollInterval = window.setInterval(() => {
    const viewportHeight = window.innerHeight
    const scrollTop = window.pageYOffset || document.documentElement.scrollTop
    const documentHeight = document.documentElement.scrollHeight
    const newScrollTop = scrollTop + currentScrollDirection * AUTO_SCROLL_SPEED

    // 检查滚动边界
    if (currentScrollDirection === -1 && newScrollTop <= 0) {
      window.scrollTo(0, 0)
      stopAutoScroll()
    } else if (currentScrollDirection === 1 && newScrollTop >= documentHeight - viewportHeight) {
      window.scrollTo(0, documentHeight - viewportHeight)
      stopAutoScroll()
    } else {
      window.scrollTo(0, newScrollTop)
    }
  }, 16) // ~60fps
}

/**
 * 查找可滚动的容器元素
 */
function findScrollableContainer(element: Element): HTMLElement | null {
  let current = element.parentElement

  while (current && current !== document.body) {
    const computedStyle = window.getComputedStyle(current)
    const overflowY = computedStyle.overflowY

    if (
      (overflowY === 'auto' || overflowY === 'scroll') &&
      current.scrollHeight > current.clientHeight
    ) {
      return current
    }

    current = current.parentElement
  }

  return null
}

/**
 * 处理容器自动滚动
 */
function handleContainerAutoScroll(container: HTMLElement, mouseY: number) {
  const containerRect = container.getBoundingClientRect()
  const containerTop = containerRect.top
  const containerHeight = containerRect.height

  // 计算鼠标在容器内的相对位置
  const relativeY = mouseY - containerTop
  const distanceFromContainerTop = relativeY
  const distanceFromContainerBottom = containerHeight - relativeY

  let newScrollDirection = 0

  // 判断是否需要在容器内滚动
  if (distanceFromContainerTop < AUTO_SCROLL_THRESHOLD && container.scrollTop > 0) {
    newScrollDirection = -1
  } else if (
    distanceFromContainerBottom < AUTO_SCROLL_THRESHOLD &&
    container.scrollTop + container.clientHeight < container.scrollHeight
  ) {
    newScrollDirection = 1
  }

  // 只有当滚动方向发生变化时才更新滚动状态
  if (newScrollDirection !== currentScrollDirection) {
    stopAutoScroll()
    if (newScrollDirection !== 0) {
      // 启动容器滚动
      currentScrollDirection = newScrollDirection
      autoScrollInterval = window.setInterval(() => {
        const newScrollTop = container.scrollTop + currentScrollDirection * AUTO_SCROLL_SPEED

        if (currentScrollDirection === -1 && newScrollTop <= 0) {
          container.scrollTop = 0
          stopAutoScroll()
        } else if (
          currentScrollDirection === 1 &&
          newScrollTop >= container.scrollHeight - container.clientHeight
        ) {
          container.scrollTop = container.scrollHeight - container.clientHeight
          stopAutoScroll()
        } else {
          container.scrollTop = newScrollTop
        }
      }, 16)
    }
  }
}

/**
 * 处理自动滚动（页面级别和容器级别）
 */
function handleAutoScroll(event: PointerEvent) {
  if (!state.value.isDragging) return

  // 首先尝试找到鼠标下方的可滚动容器
  const elementBelow = document.elementFromPoint(event.clientX, event.clientY)
  if (elementBelow) {
    const scrollableContainer = findScrollableContainer(elementBelow)
    if (scrollableContainer) {
      // 处理容器级别的滚动
      handleContainerAutoScroll(scrollableContainer, event.clientY)
      return
    }
  }

  // 如果没有找到可滚动容器，处理页面级别的滚动
  const viewportHeight = window.innerHeight
  const scrollTop = window.pageYOffset || document.documentElement.scrollTop
  const documentHeight = document.documentElement.scrollHeight

  // 计算鼠标距离视口顶部和底部的距离
  const distanceFromTop = event.clientY
  const distanceFromBottom = viewportHeight - event.clientY

  let newScrollDirection = 0

  // 判断是否需要向上滚动
  if (distanceFromTop < AUTO_SCROLL_THRESHOLD && scrollTop > 0) {
    newScrollDirection = -1
  }
  // 判断是否需要向下滚动
  else if (
    distanceFromBottom < AUTO_SCROLL_THRESHOLD &&
    scrollTop + viewportHeight < documentHeight
  ) {
    newScrollDirection = 1
  }

  // 只有当滚动方向发生变化时才更新滚动状态
  if (newScrollDirection !== currentScrollDirection) {
    stopAutoScroll() // 停止当前滚动
    if (newScrollDirection !== 0) {
      startAutoScroll(newScrollDirection) // 启动新方向的滚动
    }
  }
}

/**
 * 捕获元素的样式快照
 */
function captureElementSnapshot(element: HTMLElement): ElementSnapshot {
  const computedStyle = window.getComputedStyle(element)
  const rect = element.getBoundingClientRect()

  return {
    width: rect.width,
    height: rect.height,
    innerHTML: element.innerHTML,
    boundingRect: {
      left: rect.left,
      top: rect.top,
      width: rect.width,
      height: rect.height,
    },
    computedStyle: {
      backgroundColor: computedStyle.backgroundColor,
      color: computedStyle.color,
      fontSize: computedStyle.fontSize,
      fontFamily: computedStyle.fontFamily,
      borderRadius: computedStyle.borderRadius,
      padding: computedStyle.padding,
      border: computedStyle.border,
      boxShadow: computedStyle.boxShadow,
      display: computedStyle.display,
      alignItems: computedStyle.alignItems,
      justifyContent: computedStyle.justifyContent,
      gap: computedStyle.gap,
    },
  }
}

/**
 * 更新鼠标位置
 */
function updatePosition(event: PointerEvent) {
  state.value = {
    ...state.value,
    currentPosition: { x: event.clientX, y: event.clientY },
  }
}

/**
 * 处理拖拽移动
 */
function handlePointerMove(event: PointerEvent) {
  // 如果在准备阶段，检查是否达到拖拽阈值
  if (state.value.isPreparing) {
    const currentPos = { x: event.clientX, y: event.clientY }
    const distance = getDistance(state.value.initialPosition, currentPos)

    if (distance >= DRAG_THRESHOLD) {
      // 达到阈值，正式开始拖拽
      state.value = {
        ...state.value,
        isDragging: true,
        isPreparing: false,
        currentPosition: currentPos,
      }
    } else {
      // 未达到阈值，只更新当前位置但不开始拖拽
      updatePosition(event)
      return
    }
  }

  if (!state.value.isDragging) return

  updatePosition(event)

  // 处理自动滚动
  handleAutoScroll(event)

  // 检查当前悬停的元素
  const elementBelow = document.elementFromPoint(event.clientX, event.clientY)
  let foundDroppable: DroppableOptions | null = null

  // 查找匹配的放置区
  for (const [element, options] of droppables) {
    if (element.contains(elementBelow)) {
      if (options.acceptedDataTypes.includes(state.value.dataType || '')) {
        foundDroppable = options
        break
      }
    }
  }

  // 处理放置区变化
  if (foundDroppable !== state.value.activeDroppable) {
    // 离开之前的放置区
    if (state.value.activeDroppable?.onDragLeave) {
      state.value.activeDroppable.onDragLeave()
    }

    // 进入新的放置区
    if (foundDroppable?.onDragEnter && state.value.dragData && state.value.dataType) {
      foundDroppable.onDragEnter(state.value.dragData, state.value.dataType)
    }

    state.value = {
      ...state.value,
      activeDroppable: foundDroppable,
    }
  } else if (foundDroppable?.onDragOver && state.value.dragData && state.value.dataType) {
    // 在当前放置区内移动
    foundDroppable.onDragOver(state.value.dragData, state.value.dataType, event)
  }
}

/**
 * 处理拖拽结束
 */
function handlePointerUp() {
  // 如果只是在准备阶段，直接结束（这相当于一个普通点击）
  if (state.value.isPreparing) {
    manager.endDrag()
    return
  }

  if (!state.value.isDragging) return

  // 尝试放置
  if (state.value.activeDroppable?.onDrop && state.value.dragData && state.value.dataType) {
    state.value.activeDroppable.onDrop(state.value.dragData, state.value.dataType)
  }

  // 结束拖拽
  manager.endDrag()
}

/**
 * 拖放管理器对象
 */
export const manager = {
  /**
   * 只读状态
   */
  state: readonly(state),

  /**
   * 通过事件启动拖拽（拖动现有项）
   */
  startDragByEvent(options: DraggableOptions, event: PointerEvent) {
    // 状态守卫：如果已经在拖拽中或准备中，直接返回
    if (state.value.isDragging || state.value.isPreparing) return

    // 强制清理任何可能残留的监听器，确保状态机处于干净的初始状态
    manager.endDrag()

    const sourceElement = event.target as HTMLElement
    const elementSnapshot = captureElementSnapshot(sourceElement)

    // 计算鼠标相对于源元素的偏移量
    const mouseOffset = {
      x: event.clientX - elementSnapshot.boundingRect.left,
      y: event.clientY - elementSnapshot.boundingRect.top,
    }

    const initialPosition = { x: event.clientX, y: event.clientY }

    // 设置准备状态（而不是立即开始拖拽）
    state.value = {
      isDragging: false, // 还没有正式开始拖拽
      isPreparing: true, // 进入准备阶段
      dragData: options.data,
      dataType: options.dataType,
      sourceElement,
      currentPosition: initialPosition,
      initialPosition,
      activeDroppable: null,
      ghostComponent: options.ghostComponent || null,
      ghostProps: options.ghostProps || {},
      sourceElementSnapshot: elementSnapshot,
      mouseOffset,
    }

    // 创建新的监听器
    const moveListener = (e: PointerEvent) => handlePointerMove(e)
    const upListener = () => handlePointerUp()

    // 注册全局监听器
    document.addEventListener('pointermove', moveListener)
    document.addEventListener('pointerup', upListener)

    // 设置清理函数 - 这是唯一的清理入口，避免了全局数组的复杂性
    currentCleanup = () => {
      document.removeEventListener('pointermove', moveListener)
      document.removeEventListener('pointerup', upListener)
      currentCleanup = () => {} // 重置为空函数，防止重复调用
    }
  },

  /**
   * 程序化启动拖拽（创建新项）
   */
  startProgrammaticDrag(options: {
    data: any
    dataType: string
    ghostComponent?: any
    ghostProps?: Record<string, any>
    position?: Position
  }) {
    // 状态守卫：如果已经在拖拽中或准备中，直接返回
    if (state.value.isDragging || state.value.isPreparing) return

    // 强制清理任何可能残留的监听器
    manager.endDrag()

    const currentPos = options.position || { x: 0, y: 0 }

    // 程序化拖拽直接开始拖拽（跳过准备阶段）
    state.value = {
      isDragging: true,
      isPreparing: false,
      dragData: options.data,
      dataType: options.dataType,
      sourceElement: null,
      currentPosition: currentPos,
      initialPosition: currentPos,
      activeDroppable: null,
      ghostComponent: options.ghostComponent || null,
      ghostProps: options.ghostProps || {},
      sourceElementSnapshot: null, // 程序化拖拽没有源元素
      mouseOffset: { x: 0, y: 0 }, // 程序化拖拽没有偏移
    }

    // 创建新的监听器（程序化拖拽也需要鼠标跟踪）
    const moveListener = (e: PointerEvent) => handlePointerMove(e)
    const upListener = () => handlePointerUp()

    // 注册全局监听器
    document.addEventListener('pointermove', moveListener)
    document.addEventListener('pointerup', upListener)

    // 设置清理函数
    currentCleanup = () => {
      document.removeEventListener('pointermove', moveListener)
      document.removeEventListener('pointerup', upListener)
      currentCleanup = () => {} // 重置为空函数
    }
  },

  /**
   * 注册放置区
   */
  registerDroppable(element: HTMLElement, options: DroppableOptions) {
    droppables.set(element, options)
  },

  /**
   * 注销放置区
   */
  unregisterDroppable(element: HTMLElement) {
    droppables.delete(element)
  },

  /**
   * 隐藏拖拽（但不结束）
   * 注意：这个方法只是隐藏视觉效果，监听器依然存在
   */
  hide() {
    if (!state.value.isDragging) return

    state.value = {
      ...state.value,
      isDragging: false,
    }
  },

  /**
   * 结束拖拽并彻底清理所有状态
   * 这是唯一的清理入口，确保幂等性和自清理特性
   */
  endDrag() {
    // 状态守卫：避免不必要的重复调用
    if (!state.value.isDragging && !state.value.isPreparing && currentCleanup === (() => {})) {
      return
    }

    // 停止自动滚动
    stopAutoScroll()

    // 无条件清理监听器 - 这是最关键的安全保证
    currentCleanup()

    // 重置所有状态到初始值
    state.value = {
      isDragging: false,
      isPreparing: false,
      dragData: null,
      dataType: null,
      sourceElement: null,
      currentPosition: { x: 0, y: 0 },
      initialPosition: { x: 0, y: 0 },
      activeDroppable: null,
      ghostComponent: null,
      ghostProps: {},
      sourceElementSnapshot: null,
      mouseOffset: { x: 0, y: 0 },
    }
  },
}

/**
 * 页面卸载时的安全清理
 * 这是最后一道防线，确保在任何情况下都不会有监听器泄漏
 */
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    manager.endDrag()
  })

  // 处理页面可见性变化（例如切换标签页）
  document.addEventListener('visibilitychange', () => {
    if (document.hidden && state.value.isDragging) {
      manager.endDrag()
    }
  })

  // 处理失焦事件（例如切换到其他应用）
  window.addEventListener('blur', () => {
    if (state.value.isDragging) {
      manager.endDrag()
    }
  })
}

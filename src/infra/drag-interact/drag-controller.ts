/**
 * Interact.js 拖放控制器
 *
 * 核心职责：
 * - 管理拖放状态机
 * - 集成 interact.js
 * - 处理幽灵元素
 * - 检测区域边界
 * - 触发越界回弹
 */

import interact from 'interactjs'
// Position 已在类型导入处声明，避免重复标识符导入
import { shallowRef } from 'vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { dragPreviewActions } from './preview-state'
import { calculateDropIndex, getDistance, showErrorMessage } from './utils'
import type {
  DragPhase,
  DragManagerState,
  DragSession,
  DraggableOptions,
  DropzoneOptions,
  Position,
  InterruptionDetector,
} from './types'

// ==================== 常量 ====================

const DRAG_THRESHOLD = 5 // 拖拽阈值（像素）
const LEAVE_GRACE_MS = 80 // 离开缓冲（毫秒）

// ==================== 拖放控制器类 ====================

class InteractDragController {
  // ==================== 私有状态 ====================

  private state: DragManagerState = {
    phase: 'IDLE',
    session: null,
    targetZone: null,
    dropIndex: null,
  }

  private validZones = new Set<string>() // 记录所有可放置区域
  private ghost: HTMLElement | null = null
  private mouseOffset: Position = { x: 0, y: 0 }
  private interruptionDetector: InterruptionDetector | null = null
  private registeredSelectors = new Set<string>() // 记录已注册的选择器
  private registeredElements = new Set<HTMLElement>() // 记录已注册的元素
  private startPointer: Position | null = null // 记录拖拽起点，用于阈值计算
  private currentDropzoneElement: HTMLElement | null = null // 当前所在的 dropzone 元素
  private pendingLeaveTimer: number | null = null // 离开缓冲计时器
  private isProcessingDrop: boolean = false // 标记是否正在处理 drop（用于避免提前清理与重复执行）

  // ==================== 状态管理 ====================

  /**
   * 进入新阶段
   */
  private enterPhase(phase: DragPhase, updates: Partial<DragManagerState> = {}) {
    logger.debug(LogTags.DRAG_CROSS_VIEW, `[DragController] ${this.state.phase} → ${phase}`)

    // 清理旧状态
    if (phase === 'IDLE') {
      this.cleanup()
    }

    this.state = {
      ...this.state,
      phase,
      ...updates,
    }
    this.updateDebug()
  }

  /**
   * 清理所有状态
   */
  private cleanup() {
    this.cancelPendingLeave()
    this.removeGhost()
    dragPreviewActions.clear()
    this.state.session = null
    this.state.targetZone = null
    this.state.dropIndex = null
    this.updateDebug()
  }

  /**
   * 将内部状态同步到调试状态（供面板订阅）
   */
  private updateDebug() {
    controllerDebugState.value = {
      phase: this.state.phase,
      hasSession: !!this.state.session,
      targetZone: this.state.targetZone,
      dropIndex: this.state.dropIndex,
      validZones: Array.from(this.validZones),
      hasGhost: !!this.ghost,
    }
  }

  /**
   * 清理所有 interact.js 绑定
   */
  public cleanupAll() {
    // 清理所有已注册的选择器
    for (const selector of this.registeredSelectors) {
      interact(selector).unset()
    }
    this.registeredSelectors.clear()

    // 清理所有已注册的元素
    for (const element of this.registeredElements) {
      interact(element).unset()
    }
    this.registeredElements.clear()

    // 清理其他状态
    this.validZones.clear()
    this.cleanup()
  }

  // ==================== 幽灵元素管理 ====================

  /**
   * 创建幽灵元素
   * @param sourceElement 源元素
   * @param mouseX 鼠标X坐标（可选，用于计算精确偏移）
   * @param mouseY 鼠标Y坐标（可选，用于计算精确偏移）
   */
  private createGhost(sourceElement: HTMLElement, mouseX?: number, mouseY?: number) {
    // 移除旧的幽灵元素
    this.removeGhost()

    // 克隆源元素
    this.ghost = sourceElement.cloneNode(true) as HTMLElement

    // 获取源元素的尺寸和位置
    const rect = sourceElement.getBoundingClientRect()

    // 设置样式：保持原样，仅透明化
    this.ghost.style.position = 'fixed'
    this.ghost.style.pointerEvents = 'none' // 不阻挡鼠标事件
    this.ghost.style.zIndex = '9999'
    this.ghost.style.opacity = '0.6' // 仅设置透明度
    this.ghost.style.width = `${rect.width}px` // 保持原始宽度
    this.ghost.style.height = `${rect.height}px` // 保持原始高度
    this.ghost.style.transition = 'none' // 禁用过渡动画
    this.ghost.style.transform = 'none' // 不做任何变形

    // 🔥 计算鼠标偏移量：使用实际点击位置，避免跳动
    if (mouseX !== undefined && mouseY !== undefined) {
      // 使用鼠标相对于元素左上角的实际偏移
      this.mouseOffset = {
        x: mouseX - rect.left,
        y: mouseY - rect.top,
      }
    } else {
      // 降级方案：使用元素中心
      this.mouseOffset = {
        x: rect.width / 2,
        y: rect.height / 2,
      }
    }

    document.body.appendChild(this.ghost)
    this.updateDebug()
  }

  /**
   * 更新幽灵元素位置
   */
  private updateGhostPosition(x: number, y: number) {
    if (!this.ghost) return

    this.ghost.style.left = `${x - this.mouseOffset.x}px`
    this.ghost.style.top = `${y - this.mouseOffset.y}px`
  }

  /**
   * 移除幽灵元素
   */
  private removeGhost() {
    if (this.ghost) {
      this.ghost.remove()
      this.ghost = null
    }
    this.updateDebug()
  }

  /**
   * 安排离开目标区域（带缓冲）
   * 在缓冲时间内若重新进入任意 dropzone，将取消离开
   */
  private scheduleLeaveWithGrace() {
    if (this.pendingLeaveTimer !== null) return
    this.pendingLeaveTimer = window.setTimeout(() => {
      this.pendingLeaveTimer = null
      // 真正离开并回弹
      this.currentDropzoneElement = null
      this.leaveTarget()
      dragPreviewActions.triggerRebound()
    }, LEAVE_GRACE_MS)
  }

  /**
   * 取消离开缓冲
   */
  private cancelPendingLeave() {
    if (this.pendingLeaveTimer !== null) {
      clearTimeout(this.pendingLeaveTimer)
      this.pendingLeaveTimer = null
    }
  }

  // ==================== 拖放流程 ====================

  /**
   * 计算坐标下的顶层 dropzone 元素
   * 基于 elementsFromPoint/elementFromPoint + 最近的 [data-zone-id] 祖先
   */
  private getTopmostDropzoneAt(
    clientX: number,
    clientY: number
  ): { element: HTMLElement; zoneId: string; type: 'kanban' | 'calendar' } | null {
    const pickList: Element[] =
      (document as any).elementsFromPoint?.(clientX, clientY) ??
      (() => {
        const el = document.elementFromPoint(clientX, clientY)
        return el ? [el] : []
      })()

    for (const el of pickList) {
      const dropzoneEl = (el as HTMLElement).closest('[data-zone-id]') as HTMLElement | null
      if (dropzoneEl && this.registeredElements.has(dropzoneEl)) {
        const zoneId = dropzoneEl.getAttribute('data-zone-id')!
        const type =
          (dropzoneEl.getAttribute('data-zone-type') as 'kanban' | 'calendar') || 'kanban'
        return { element: dropzoneEl, zoneId, type }
      }
    }
    return null
  }

  /**
   * 开始拖动准备
   */
  private startPreparing(event: any, options: DraggableOptions) {
    if (this.state.phase !== 'IDLE') {
      logger.warn(LogTags.DRAG_CROSS_VIEW, 'Cannot start preparing: not idle')
      return
    }

    const sourceElement = event.target as HTMLElement
    // 记录拖拽起点（兼容性处理）
    this.startPointer = {
      x: event?.clientX ?? event?.pageX ?? event?.x0 ?? 0,
      y: event?.clientY ?? event?.pageY ?? event?.y0 ?? 0,
    }
    const dragData = options.getData(sourceElement)

    // 创建拖放会话（符合新策略系统的结构，支持泛型）
    const session: DragSession<any> = {
      id: `drag-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      source: {
        viewId: dragData.sourceView.id,
        viewType: dragData.sourceView.type,
        viewKey: dragData.sourceView.id, // viewKey = viewId
        elementId:
          sourceElement.getAttribute('data-task-id') ||
          sourceElement.getAttribute('data-object-id') ||
          (dragData.data as any).id,
      },
      object: {
        type: dragData.type,
        data: { ...dragData.data }, // 深拷贝快照
        originalIndex: dragData.index,
      },
      dragMode: 'normal', // 默认为 normal 模式
      target: undefined, // 初始时无目标
      startTime: Date.now(),
      metadata: {
        date: (dragData.sourceView.config as any)?.date,
        areaId: (dragData.data as any).area_id || undefined,
        // 🔥 V2: 保存源组件的灵活上下文数据
        sourceContext: dragData.sourceContext,
      },
    }

    this.enterPhase('PREPARING', { session })

    // 🔥 创建幽灵元素：传入鼠标坐标，避免跳动
    const mouseX = event?.clientX ?? event?.pageX ?? 0
    const mouseY = event?.clientY ?? event?.pageY ?? 0
    this.createGhost(sourceElement, mouseX, mouseY)

    // 立即更新幽灵元素位置到当前鼠标位置
    this.updateGhostPosition(mouseX, mouseY)
  }

  /**
   * 开始正式拖动
   */
  private startDragging() {
    if (this.state.phase !== 'PREPARING') {
      logger.warn(LogTags.DRAG_CROSS_VIEW, 'Cannot start dragging: not preparing')
      return
    }

    this.enterPhase('DRAGGING')
    logger.info(LogTags.DRAG_CROSS_VIEW, '🎬 拖放开始', {
      objectType: this.state.session?.object.type,
      objectId: this.state.session?.object.data.id,
      objectTitle: this.state.session?.object.data.title,
      sourceView: this.state.session?.source.viewId,
      dragMode: this.state.session?.dragMode,
    })
  }

  /**
   * 进入目标区域
   */
  private enterTarget(zoneId: string, dropIndex: number = 0) {
    if (this.state.phase !== 'DRAGGING' && this.state.phase !== 'OVER_TARGET') {
      return
    }

    this.enterPhase('OVER_TARGET', {
      targetZone: zoneId,
      dropIndex,
    })
  }

  /**
   * 离开目标区域
   */
  private leaveTarget() {
    if (this.state.phase !== 'OVER_TARGET') {
      return
    }

    this.enterPhase('DRAGGING', {
      targetZone: null,
      dropIndex: null,
    })
  }

  /**
   * 执行放置
   */
  private async executeDrop() {
    if (this.state.phase !== 'OVER_TARGET') {
      logger.warn(LogTags.DRAG_CROSS_VIEW, 'Cannot drop: not over target')
      this.cancel()
      return
    }

    if (!this.state.session) {
      logger.error(LogTags.DRAG_CROSS_VIEW, 'Cannot drop: no session')
      this.cancel()
      return
    }

    this.enterPhase('DROPPING')

    try {
      // 检查中断
      if (this.interruptionDetector) {
        const shouldInterrupt = await this.interruptionDetector.shouldInterrupt(this.state.session)
        if (shouldInterrupt) {
          const reason = this.interruptionDetector.getInterruptionReason()
          logger.warn(LogTags.DRAG_CROSS_VIEW, 'Drop interrupted', { reason })
          showErrorMessage(reason)
          this.cancel()
          return
        }
      }

      // TODO: 调用策略系统执行业务逻辑
      // const strategy = findStrategy(this.buildContext())
      // await strategy.execute(this.buildContext())

      logger.info(LogTags.DRAG_CROSS_VIEW, '✅ 拖放完成', {
        objectType: this.state.session.object.type,
        objectId: this.state.session.object.data.id,
        objectTitle: this.state.session.object.data.title,
        sourceView: this.state.session.source.viewId,
        targetZone: this.state.targetZone,
        dragMode: this.state.session.dragMode,
      })

      this.enterPhase('IDLE')
    } catch (error) {
      logger.error(LogTags.DRAG_CROSS_VIEW, 'Drop failed', error as Error)
      const errorMessage = error instanceof Error ? error.message : '未知错误'
      showErrorMessage(`操作失败: ${errorMessage}`)
      this.cancel()
    }
  }

  /**
   * 取消拖动
   */
  private cancel() {
    logger.info(LogTags.DRAG_CROSS_VIEW, '❌ 拖放取消', {
      phase: this.state.phase,
      hadTarget: !!this.state.targetZone,
    })
    this.enterPhase('IDLE')
  }

  // ==================== 公开 API ====================

  /**
   * 安装可拖拽元素
   */
  installDraggable(selector: string, options: DraggableOptions) {
    // 避免重复注册
    if (this.registeredSelectors.has(selector)) {
      logger.debug(LogTags.DRAG_CROSS_VIEW, `Selector already registered: ${selector}`)
      return
    }

    interact(selector).draggable({
      // 基础配置
      inertia: false, // 禁用惯性
      autoScroll: true, // 启用自动滚动

      listeners: {
        start: (event) => {
          // 阻止默认行为和事件冒泡
          event.preventDefault()
          this.startPreparing(event, options)
        },

        move: (event) => {
          // 更新幽灵元素位置
          this.updateGhostPosition(event.clientX, event.clientY)

          // 检查是否达到拖拽阈值
          if (this.state.phase === 'PREPARING') {
            const origin = this.startPointer ?? { x: event.x0, y: event.y0 }
            const distance = getDistance(origin, { x: event.clientX, y: event.clientY })

            if (distance >= DRAG_THRESHOLD) {
              this.startDragging()
            }
          }

          // 🔥 混合方案：
          // - DRAGGING 阶段：手动检测第一次进入（因为可能在起始 dropzone 内）
          // - OVER_TARGET 阶段：依赖原生事件 + 更新 dropIndex
          if (this.state.phase === 'DRAGGING' && this.state.session) {
            // 在起始 dropzone 内开始拖动时，原生 dragenter 不会触发
            // 需要手动检测并触发进入逻辑
            this.checkInitialDropzone(event.clientX, event.clientY)
          } else if (this.state.phase === 'OVER_TARGET') {
            // 动态检测顶层 dropzone，如发生切换则更新预览与状态
            const top = this.getTopmostDropzoneAt(event.clientX, event.clientY)

            if (!top) {
              // 不在任何 dropzone 上 → 启动离开缓冲，等待可能进入下一列
              this.scheduleLeaveWithGrace()
              dragPreviewActions.updateMousePosition({ x: event.clientX, y: event.clientY })
              return
            }

            // 若顶层 dropzone 改变，则切换
            if (!this.currentDropzoneElement || top.element !== this.currentDropzoneElement) {
              // 在切换/进入新列时取消离开缓冲
              this.cancelPendingLeave()
              this.currentDropzoneElement = top.element
              if (top.type === 'kanban' && this.state.session) {
                const dropIndex = this.calculateDropIndexForZone(event.clientY, top.element)
                dragPreviewActions.setKanbanPreview({
                  draggedObject: this.state.session.object.data,
                  objectType: this.state.session.object.type,
                  sourceZoneId: this.state.session.source.viewId,
                  targetZoneId: top.zoneId,
                  mousePosition: { x: event.clientX, y: event.clientY },
                  dropIndex,
                })
                this.enterTarget(top.zoneId, dropIndex)
              } else {
                dragPreviewActions.triggerRebound()
                this.enterTarget(top.zoneId, 0)
              }
            } else if (this.currentDropzoneElement) {
              // 在当前列内移动，确保取消任何挂起的离开
              this.cancelPendingLeave()
              // 顶层未变，在当前 dropzone 内更新 dropIndex
              const dropIndex = this.calculateDropIndexForZone(
                event.clientY,
                this.currentDropzoneElement,
                true
              )
              if (dropIndex !== this.state.dropIndex) {
                dragPreviewActions.updateDropIndex(dropIndex)
                this.state.dropIndex = dropIndex
                this.updateDebug()
              }
            }

            // 鼠标位置始终更新
            dragPreviewActions.updateMousePosition({ x: event.clientX, y: event.clientY })
          }
        },

        end: (event) => {
          event.preventDefault()
          // 如果 dropzone 正在处理 onDrop，避免重复执行或提前清理
          if (this.isProcessingDrop) {
            return
          }
          if (this.state.phase === 'OVER_TARGET') {
            // 非自定义 onDrop 情况才会走 executeDrop
            this.executeDrop()
          } else {
            this.cancel()
          }
        },
      },
    })

    // 记录已注册的选择器
    this.registeredSelectors.add(selector)
  }

  /**
   * 注册拖放区
   */
  registerDropzone(element: HTMLElement, options: DropzoneOptions) {
    const { zoneId, type } = options

    // 避免重复注册
    if (this.registeredElements.has(element)) {
      logger.debug(LogTags.DRAG_CROSS_VIEW, `Element already registered as dropzone: ${zoneId}`)
      return
    }

    // 记录为有效区域
    this.validZones.add(zoneId)

    // 设置 data 属性用于调试和碰撞检测
    element.setAttribute('data-zone-id', zoneId)
    element.setAttribute('data-zone-type', type)

    // ✅ 原生版本：完全依赖 interact.js 的 dropzone 事件
    const isPhysicalZone = type === 'kanban'

    interact(element).dropzone({
      // 接受任务与模板两类卡片包装元素（支持跨类型拖放）
      accept: '.task-card-wrapper, .template-card-wrapper',
      overlap: 'pointer', // 指针模式：鼠标进入即触发

      listeners: {
        dragenter: (event: any) => {
          logger.debug(LogTags.DRAG_CROSS_VIEW, `[✅ dropzone.dragenter] zoneId: ${zoneId}`)

          if (!this.state.session) {
            logger.warn(LogTags.DRAG_CROSS_VIEW, 'dragenter: No session found')
            return
          }

          // 获取鼠标位置（从 dragEvent 中提取）
          const dragEvent = event.dragEvent || event
          const clientX = dragEvent.clientX || 0
          const clientY = dragEvent.clientY || 0

          // 🔥 顶层 dropzone 判定：只允许顶层的 dropzone 响应
          const top = this.getTopmostDropzoneAt(clientX, clientY)
          if (!top || top.element !== element) {
            logger.debug(
              LogTags.DRAG_CROSS_VIEW,
              `[⛔ dropzone.dragenter ignored] zoneId: ${zoneId} is not topmost at pointer`
            )
            return
          }

          // 进入目标列，取消任何挂起的离开缓冲
          this.cancelPendingLeave()

          // 保存当前 dropzone 元素引用
          this.currentDropzoneElement = element

          if (isPhysicalZone) {
            // Kanban 区域：显示实体预览
            const dropIndex = this.calculateDropIndexForZone(clientY, element)

            dragPreviewActions.setKanbanPreview({
              draggedObject: this.state.session.object.data,
              objectType: this.state.session.object.type,
              sourceZoneId: this.state.session.source.viewId,
              targetZoneId: zoneId,
              mousePosition: { x: clientX, y: clientY },
              dropIndex,
            })
          } else {
            // 日历等非物理区域：触发回弹
            dragPreviewActions.triggerRebound()
          }

          // 进入目标区域状态
          this.enterTarget(
            zoneId,
            isPhysicalZone ? this.calculateDropIndexForZone(clientY, element) : 0
          )
        },

        dragover: () => {
          // dragover 在 dragenter 后持续触发
          // 我们在 draggable.move 中已经处理了更新，这里只需保持状态
          if (this.state.phase !== 'OVER_TARGET') {
            logger.warn(LogTags.DRAG_CROSS_VIEW, `dragover but phase is ${this.state.phase}`)
          }
        },

        dragleave: () => {
          logger.debug(LogTags.DRAG_CROSS_VIEW, `[dropzone.dragleave] zoneId: ${zoneId}`)

          // 不立即离开，安排缓冲期
          this.scheduleLeaveWithGrace()
        },

        drop: async () => {
          logger.debug(LogTags.DRAG_CROSS_VIEW, `[✅ dropzone.drop] zoneId: ${zoneId}`)

          // 🔒 关键检查：必须处于 OVER_TARGET 状态才能执行 drop
          // 防止在回弹状态下（DRAGGING）误触发 drop
          if (this.state.phase !== 'OVER_TARGET') {
            logger.warn(
              LogTags.DRAG_CROSS_VIEW,
              `[⛔ dropzone.drop rejected] phase is ${this.state.phase}, expected OVER_TARGET`
            )
            this.cancel()
            return
          }

          if (options.onDrop && this.state.session) {
            // 标记正在处理 drop，避免在 draggable.end 中提前清理预览
            this.isProcessingDrop = true
            try {
              await options.onDrop(this.state.session)
            } finally {
              // 在 onDrop 完成后再清理预览，避免视觉闪烁
              this.enterPhase('IDLE')
              this.isProcessingDrop = false
            }
          } else {
            await this.executeDrop()
          }
        },
      },
    })

    // 记录已注册的元素
    this.registeredElements.add(element)
  }

  /**
   * 取消注册拖放区
   */
  unregisterDropzone(element: HTMLElement) {
    if (this.registeredElements.has(element)) {
      interact(element).unset()
      this.registeredElements.delete(element)

      // 从有效区域中移除
      const zoneId = element.getAttribute('data-zone-id')
      if (zoneId) {
        this.validZones.delete(zoneId)
      }

      logger.debug(LogTags.DRAG_CROSS_VIEW, `Unregistered dropzone: ${zoneId}`)
    }
  }

  /**
   * 计算特定区域的插入位置
   * @param pointerY 鼠标Y坐标
   * @param element dropzone元素
   * @param useLastIndex 是否使用上一次的索引（施密特触发器）
   */
  private calculateDropIndexForZone(
    pointerY: number,
    element: HTMLElement,
    useLastIndex: boolean = false
  ): number {
    // 支持任务与模板两类卡片元素
    const wrappers = Array.from(
      element.querySelectorAll('.task-card-wrapper, .template-card-wrapper')
    ) as HTMLElement[]
    // 🔥 传入上一次的 dropIndex，启用施密特触发器
    const lastDropIndex = useLastIndex ? (this.state.dropIndex ?? undefined) : undefined
    return calculateDropIndex(pointerY, wrappers, lastDropIndex)
  }

  /**
   * 🔥 检查初始 dropzone
   * 用于解决"在起始 dropzone 内开始拖动时，原生 dragenter 不会触发"的问题
   */
  private checkInitialDropzone(clientX: number, clientY: number) {
    if (!this.state.session) return

    // 只在 DRAGGING 阶段第一次检测
    if (this.state.phase !== 'DRAGGING') return

    // 使用顶层 dropzone 判定
    const top = this.getTopmostDropzoneAt(clientX, clientY)
    if (!top) {
      this.currentDropzoneElement = null
      dragPreviewActions.triggerRebound()
      return
    }

    // 手动触发进入逻辑（模拟 dragenter）
    this.currentDropzoneElement = top.element
    const isPhysicalZone = top.type === 'kanban'

    if (isPhysicalZone) {
      const dropIndex = this.calculateDropIndexForZone(clientY, top.element)
      dragPreviewActions.setKanbanPreview({
        draggedObject: this.state.session.object.data,
        objectType: this.state.session.object.type,
        sourceZoneId: this.state.session.source.viewId,
        targetZoneId: top.zoneId,
        mousePosition: { x: clientX, y: clientY },
        dropIndex,
      })
      this.enterTarget(top.zoneId, dropIndex)
    } else {
      dragPreviewActions.triggerRebound()
      this.enterTarget(top.zoneId, 0)
    }
  }

  /**
   * 设置中断检测器（预留）
   */
  setInterruptionDetector(detector: InterruptionDetector) {
    this.interruptionDetector = detector
  }

  /**
   * 获取调试信息
   */
  getDebugInfo() {
    return {
      phase: this.state.phase,
      hasSession: !!this.state.session,
      targetZone: this.state.targetZone,
      dropIndex: this.state.dropIndex,
      validZones: Array.from(this.validZones),
      hasGhost: !!this.ghost,
    }
  }
}

// ==================== 单例导出 ====================

export const interactManager = new InteractDragController()

// 一个浅响应的调试状态，供面板订阅
export const controllerDebugState = shallowRef({
  phase: 'IDLE' as DragPhase,
  hasSession: false,
  targetZone: null as string | null,
  dropIndex: null as number | null,
  validZones: [] as string[],
  hasGhost: false,
})

// 初始化一次，以反映初始状态
controllerDebugState.value = interactManager.getDebugInfo()

// ==================== 全局清理 ====================

if (typeof window !== 'undefined') {
  // 页面卸载时清理
  window.addEventListener('beforeunload', () => {
    interactManager['cleanup']()
  })

  // 页面隐藏时清理（切换标签页）
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
      interactManager['cancel']()
    }
  })

  // 失焦时清理（切换到其他应用）
  window.addEventListener('blur', () => {
    interactManager['cancel']()
  })

  // ESC 键取消
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      interactManager['cancel']()
    }
  })
}

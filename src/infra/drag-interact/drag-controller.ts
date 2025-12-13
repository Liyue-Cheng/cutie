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
import { useRegisterStore } from '@/stores/register'
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
  private lastMouseY: number | null = null // 记录上一次鼠标Y坐标（用于方向进入判定）
  private registeredElements = new Set<HTMLElement>() // 记录已注册的元素
  private startPointer: Position | null = null // 记录拖拽起点，用于阈值计算
  private currentDropzoneElement: HTMLElement | null = null // 当前所在的 dropzone 元素
  private pendingLeaveTimer: number | null = null // 离开缓冲计时器
  private isProcessingDrop: boolean = false // 标记是否正在处理 drop（用于避免提前清理与重复执行）
  private isCompactModeActive: boolean = false // 标记当前拖动是否启用了截断模式
  private dragSourceElement: HTMLElement | null = null // 记录当前拖动的源元素
  private dynamicDropEnabled: boolean = false // 标记是否已启用动态 drop 匹配
  private globalEventHandlers: Map<string, EventListener> = new Map() // 🔥 存储全局事件处理器以便清理
  private registerStore: ReturnType<typeof useRegisterStore> | null = null // 延迟获取寄存器 Store

  // ==================== 状态管理 ====================

  /**
   * 获取寄存器 Store（延迟初始化以避免在 Pinia 未就绪时调用）
   */
  private getRegisterStore() {
    if (!this.registerStore) {
      this.registerStore = useRegisterStore()
    }
    return this.registerStore
  }

  /**
   * 记录全局拖拽激活状态
   */
  private markGlobalDragActive(sessionId: string) {
    const store = this.getRegisterStore()
    store.writeRegister(store.RegisterKeys.GLOBAL_DRAG_ACTIVE, {
      sessionId,
      startedAt: Date.now(),
    })
  }

  /**
   * 清除全局拖拽激活状态
   */
  private clearGlobalDragActive() {
    const store = this.getRegisterStore()
    store.deleteRegister(store.RegisterKeys.GLOBAL_DRAG_ACTIVE)
  }

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
    this.clearDragSourceElement()
    this.isCompactModeActive = false
    // 🔥 使用 forceReset 确保完全清理
    dragPreviewActions.forceReset()
    this.state.session = null
    this.state.targetZone = null
    this.state.dropIndex = null
    this.lastMouseY = null
    this.clearGlobalDragActive()
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
    // 🔥 先取消当前拖动操作
    if (this.state.phase !== 'IDLE') {
      this.cancel()
    }

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

    // 🔥 清理全局事件监听器
    for (const [eventName, handler] of this.globalEventHandlers) {
      if (eventName === 'keydown') {
        document.removeEventListener(eventName, handler)
      } else {
        window.removeEventListener(eventName, handler)
      }
    }
    this.globalEventHandlers.clear()

    // 清理其他状态
    this.validZones.clear()
    this.cleanup()

    logger.debug(LogTags.DRAG_CROSS_VIEW, 'All drag interactions cleaned up')
  }

  /**
   * 确保启用 interact.js 的动态 drop 功能
   * 该功能会在拖动过程中实时重新计算 dropzone 的位置与尺寸
   */
  private ensureDynamicDropEnabled() {
    if (this.dynamicDropEnabled) {
      return
    }

    interact.dynamicDrop(true)
    this.dynamicDropEnabled = true
    logger.debug(LogTags.DRAG_CROSS_VIEW, '[DragController] dynamicDrop enabled')
  }

  /**
   * 计算当前 dropzone 的实时矩形
   * 使用页面坐标系，确保在滚动或布局变化时仍能正确命中
   */
  private getDynamicDropzoneRect(element: HTMLElement) {
    const rect = element.getBoundingClientRect()
    const scrollX = window.scrollX ?? window.pageXOffset ?? 0
    const scrollY = window.scrollY ?? window.pageYOffset ?? 0

    return {
      left: rect.left + scrollX,
      right: rect.right + scrollX,
      top: rect.top + scrollY,
      bottom: rect.bottom + scrollY,
      width: rect.width,
      height: rect.height,
    }
  }

  /**
   * 判断当前任务卡是否需要进入截断模式
   * 统一采用高度阈值（像素），避免依赖具体结构
   */
  private shouldApplyCompactMode(element: HTMLElement): boolean {
    const rect = element.getBoundingClientRect()
    // 阈值经验值：标题栏 + 笔记约 120px，高于该值视为内容过长
    const COMPACT_HEIGHT_THRESHOLD = 120
    return rect.height > COMPACT_HEIGHT_THRESHOLD
  }

  /**
   * 应用/移除源元素的截断状态
   */
  private setDragSourceElement(element: HTMLElement, compact: boolean) {
    if (this.dragSourceElement && this.dragSourceElement !== element) {
      this.dragSourceElement.classList.remove('drag-compact')
    }

    this.dragSourceElement = element

    if (compact) {
      element.classList.add('drag-compact')
    } else {
      element.classList.remove('drag-compact')
    }
  }

  /**
   * 清理源元素的截断状态
   */
  private clearDragSourceElement() {
    if (this.dragSourceElement) {
      this.dragSourceElement.classList.remove('drag-compact')
      this.dragSourceElement = null
    }
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
    // 🔥 安全：如果已有定时器，先清除再创建新的
    this.cancelPendingLeave()

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

    const rawElement = event.target as HTMLElement
    const sourceElement =
      (rawElement.closest('[data-task-id], [data-object-id]') as HTMLElement | null) ?? rawElement
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

    this.markGlobalDragActive(session.id)

    const shouldCompact = this.shouldApplyCompactMode(sourceElement)
    this.isCompactModeActive = shouldCompact
    this.setDragSourceElement(sourceElement, shouldCompact)

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

    // 🔥 安全：保存session副本，防止在异步过程中被清理
    const sessionCopy = { ...this.state.session }

    this.enterPhase('DROPPING')

    try {
      // 检查中断
      if (this.interruptionDetector) {
        const shouldInterrupt = await this.interruptionDetector.shouldInterrupt(sessionCopy)
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
        objectType: sessionCopy.object.type,
        objectId: sessionCopy.object.data.id,
        objectTitle: sessionCopy.object.data.title,
        sourceView: sessionCopy.source.viewId,
        targetZone: this.state.targetZone,
        dragMode: sessionCopy.dragMode,
      })

      this.enterPhase('IDLE')
    } catch (error) {
      logger.error(LogTags.DRAG_CROSS_VIEW, 'Drop failed', error as Error)
      const errorMessage = error instanceof Error ? error.message : '未知错误'
      showErrorMessage(`操作失败: ${errorMessage}`)
      // 🔥 安全：确保即使出错也能返回到 IDLE 状态
      this.enterPhase('IDLE')
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
    // 🔥 确保全局事件监听器已注册
    this.setupGlobalEventListeners()

    // 🔥 安全：先清理旧的绑定再注册新的
    if (this.registeredSelectors.has(selector)) {
      logger.debug(
        LogTags.DRAG_CROSS_VIEW,
        `Selector already registered, re-registering: ${selector}`
      )
      interact(selector).unset() // 清理旧的绑定
    }

    interact(selector).draggable({
      // 基础配置
      inertia: false, // 禁用惯性
      autoScroll: true, // 启用自动滚动

      listeners: {
        start: (event) => {
          // 🔥 安全检查：确保状态为 IDLE 才允许开始
          if (this.state.phase !== 'IDLE') {
            logger.warn(
              LogTags.DRAG_CROSS_VIEW,
              `Cannot start drag: current phase is ${this.state.phase}`
            )
            event.preventDefault()
            return
          }
          // 阻止默认行为和事件冒泡
          event.preventDefault()
          this.startPreparing(event, options)
        },

        move: (event) => {
          // 🔥 防御性检查：确保事件有效
          if (!event || typeof event.clientX !== 'number' || typeof event.clientY !== 'number') {
            logger.warn(LogTags.DRAG_CROSS_VIEW, 'Invalid move event received', event)
            return
          }

          // 🔥 防御性检查：确保坐标在合理范围内
          if (!isFinite(event.clientX) || !isFinite(event.clientY)) {
            logger.warn(LogTags.DRAG_CROSS_VIEW, 'Invalid coordinates in move event', {
              x: event.clientX,
              y: event.clientY,
            })
            return
          }

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
                  isCompact: this.isCompactModeActive,
                })
                this.enterTarget(top.zoneId, dropIndex)
                // 初始化方向门控的参考坐标
                this.lastMouseY = event.clientY
              } else {
                dragPreviewActions.triggerRebound()
                this.enterTarget(top.zoneId, 0)
                this.lastMouseY = event.clientY
              }
            } else if (this.currentDropzoneElement) {
              // 在当前列内移动，确保取消任何挂起的离开
              this.cancelPendingLeave()
              // 顶层未变，在当前 dropzone 内更新 dropIndex（仅当按正确方向进入触发区时步进）
              const dropIndex = this.calculateDropIndexWithDirectionalGate(
                event.clientY,
                this.currentDropzoneElement
              )
              if (dropIndex !== this.state.dropIndex) {
                dragPreviewActions.updateDropIndex(dropIndex)
                this.state.dropIndex = dropIndex
                this.updateDebug()
              }
              // 更新上一次鼠标Y坐标
              this.lastMouseY = event.clientY
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

    this.ensureDynamicDropEnabled()

    // 记录为有效区域
    this.validZones.add(zoneId)

    // 设置 data 属性用于调试和碰撞检测
    element.setAttribute('data-zone-id', zoneId)
    element.setAttribute('data-zone-type', type)

    // ✅ 原生版本：完全依赖 interact.js 的 dropzone 事件
    const isPhysicalZone = type === 'kanban'

    const interactable = interact(element)
    const rectChecker = options.rectChecker ?? (() => this.getDynamicDropzoneRect(element))

    interactable.dropzone({
      // 接受可拖拽元素（按语义类型命名：{type}-draggable）
      accept: '.task-draggable, .template-draggable, .project-draggable',
      overlap: 'pointer', // 指针模式：鼠标进入即触发
      // 启用实时矩形检测，确保拖动过程中区域变化能被捕捉
      rectChecker,

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
              isCompact: this.isCompactModeActive,
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
    } as any)

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
    // 查找所有可拖拽元素（按语义类型命名）
    const wrappers = Array.from(
      element.querySelectorAll('.task-draggable, .template-draggable, .project-draggable')
    ) as HTMLElement[]
    // 🔥 传入上一次的 dropIndex，启用施密特触发器
    const lastDropIndex = useLastIndex ? (this.state.dropIndex ?? undefined) : undefined
    return calculateDropIndex(pointerY, wrappers, lastDropIndex)
  }

  /**
   * 在当前 dropzone 内，基于"方向进入触发区"步进
   * 规则：
   * - 仅当鼠标向下移动，且从触发区外进入下一项的触发区时，索引 +1
   * - 仅当鼠标向上移动，且从触发区外进入上一项的触发区时，索引 -1
   * - 🔥 修复：检测到进入触发区后，继续循环检测是否应该继续步进（处理快速移动）
   * - 其他情况返回当前索引（保持稳定）
   */
  private calculateDropIndexWithDirectionalGate(pointerY: number, element: HTMLElement): number {
    const wrappers = Array.from(
      element.querySelectorAll('.task-draggable, .template-draggable, .project-draggable')
    ) as HTMLElement[]

    const lastIndex = Math.max(0, Math.min(this.state.dropIndex ?? 0, wrappers.length))

    // 首次无历史坐标，使用现有算法给出初始位置
    if (this.lastMouseY === null) {
      return this.calculateDropIndexForZone(pointerY, element, true)
    }

    const deltaY = pointerY - this.lastMouseY
    const ZONE_RATIO = 0.1
    const MIN_ZONE_PX = 8
    const zonePx = (h: number) => Math.max(h * ZONE_RATIO, MIN_ZONE_PX)

    // 仅处理向下进入触发区的情形
    if (deltaY > 0) {
      const nextIndex = Math.min(lastIndex + 1, wrappers.length)
      if (nextIndex < wrappers.length) {
        const nextEl = wrappers[nextIndex]
        if (!nextEl) return lastIndex
        const rect = nextEl.getBoundingClientRect()
        const enterThreshold = rect.top + zonePx(rect.height)

        const wasOutside = this.lastMouseY < enterThreshold
        const nowInside = pointerY >= enterThreshold

        if (wasOutside && nowInside) {
          // 🔥 检测到进入触发区，继续循环检测是否应该继续下移
          let newIndex = lastIndex + 1
          while (newIndex < wrappers.length) {
            const checkEl = wrappers[newIndex]
            if (!checkEl) break
            const checkRect = checkEl.getBoundingClientRect()
            const checkThreshold = checkRect.top + zonePx(checkRect.height)
            if (pointerY >= checkThreshold) {
              newIndex++
            } else {
              break
            }
          }
          return Math.min(newIndex, wrappers.length)
        }
      }
      return lastIndex
    }

    // 处理向上进入触发区的情形（底部触发区只能向上触发）
    if (deltaY < 0) {
      const prevIndex = Math.max(lastIndex - 1, 0)
      if (prevIndex >= 0 && prevIndex < wrappers.length) {
        const prevEl = wrappers[prevIndex]
        if (!prevEl) return lastIndex
        const rect = prevEl.getBoundingClientRect()
        const enterThreshold = rect.bottom - zonePx(rect.height)

        const wasOutside = this.lastMouseY > enterThreshold
        const nowInside = pointerY <= enterThreshold

        if (wasOutside && nowInside) {
          // 🔥 检测到进入触发区，继续循环检测是否应该继续上移
          let newIndex = lastIndex - 1
          while (newIndex > 0) {
            const checkIndex = newIndex - 1
            const checkEl = wrappers[checkIndex]
            if (!checkEl) break
            const checkRect = checkEl.getBoundingClientRect()
            const checkThreshold = checkRect.bottom - zonePx(checkRect.height)
            if (pointerY <= checkThreshold) {
              newIndex--
            } else {
              break
            }
          }
          return Math.max(newIndex, 0)
        }
      }
      return lastIndex
    }

    // 未移动或极小移动：保持原索引
    return lastIndex
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
        isCompact: this.isCompactModeActive,
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

  /**
   * 🔥 注册全局事件监听器（可清理）
   */
  private setupGlobalEventListeners() {
    if (typeof window === 'undefined') return
    if (this.globalEventHandlers.size > 0) return // 已注册

    // beforeunload 事件
    const beforeunloadHandler = () => {
      this.cleanup()
    }
    window.addEventListener('beforeunload', beforeunloadHandler)
    this.globalEventHandlers.set('beforeunload', beforeunloadHandler)

    // visibilitychange 事件
    const visibilitychangeHandler = () => {
      if (document.hidden) {
        this.cancel()
      }
    }
    document.addEventListener('visibilitychange', visibilitychangeHandler as EventListener)
    this.globalEventHandlers.set('visibilitychange', visibilitychangeHandler as EventListener)

    // blur 事件
    const blurHandler = () => {
      this.cancel()
    }
    window.addEventListener('blur', blurHandler)
    this.globalEventHandlers.set('blur', blurHandler)

    // keydown 事件（ESC取消）
    const keydownHandler = (event: Event) => {
      if ((event as KeyboardEvent).key === 'Escape') {
        this.cancel()
      }
    }
    document.addEventListener('keydown', keydownHandler)
    this.globalEventHandlers.set('keydown', keydownHandler)

    logger.debug(LogTags.DRAG_CROSS_VIEW, 'Global event listeners registered')
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

// ==================== 🔥 全局清理已移至类内部管理 ====================
// 全局事件监听器现在通过 setupGlobalEventListeners() 方法管理
// 可以通过 cleanupAll() 方法清理

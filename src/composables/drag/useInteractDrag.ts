/**
 * useInteractDrag - 新的拖放 Composable
 *
 * 基于 interact.js 的统一拖放解决方案
 * 替代原有的 useSameViewDrag + useCrossViewDrag + useCrossViewDragTarget
 *
 * 核心特性：
 * - 单一 composable 管理所有拖放逻辑
 * - 响应式预览渲染
 * - 越界回弹支持
 * - 与现有策略系统兼容
 */

import { computed, onMounted, onBeforeUnmount, type Ref } from 'vue'
import { interactManager, dragPreviewState } from '@/infra/drag-interact'
import type { ViewMetadata } from '@/types/drag'
import type { TaskCard } from '@/types/dtos'
import type { DragData } from '@/infra/drag-interact/types'
import { makeDragDecision } from '@/services/dragDecisionService'

/**
 * useInteractDrag 配置选项
 */
export interface UseInteractDragOptions {
  /** 视图元数据 */
  viewMetadata: Ref<ViewMetadata>

  /** 任务列表 */
  tasks: Ref<TaskCard[]>

  /** 任务列表容器元素引用 */
  containerRef: Ref<HTMLElement | null>

  /** 可拖拽元素选择器 */
  draggableSelector: string

  /** 拖放区类型 */
  dropzoneType?: 'kanban' | 'calendar'

  /** 自定义放置处理函数 */
  onDrop?: (session: any) => Promise<void>
}

/**
 * useInteractDrag Composable
 */
export function useInteractDrag(options: UseInteractDragOptions) {
  const {
    viewMetadata,
    tasks,
    containerRef,
    draggableSelector,
    dropzoneType = 'kanban',
    onDrop,
  } = options

  // ==================== 响应式状态 ====================

  /**
   * 显示的任务列表（包含预览逻辑）
   * 这是核心的响应式计算，实现了需求文档中的"实体元素"预览
   */
  const displayTasks = computed(() => {
    const preview = dragPreviewState.value
    const currentTasks = tasks.value
    const currentViewId = viewMetadata.value.id

    // 没有预览 → 显示原始列表
    if (!preview) {
      return currentTasks
    }

    const { ghostTask, sourceZoneId, targetZoneId } = preview.raw
    const { dropIndex } = preview.computed

    // 🔥 场景C: 越界回弹 (targetZoneId === null)
    // 所有列表都回到原始状态
    if (targetZoneId === null) {
      return currentTasks
    }

    // 场景A: 实体元素在本列表中预览
    if (targetZoneId === currentViewId) {
      // 先移除被拖动的任务（如果在本列表中）
      const withoutDragged = currentTasks.filter((t) => t.id !== ghostTask.id)

      if (dropIndex !== undefined) {
        // 插入预览位置
        const preview = [...withoutDragged]
        const safeIndex = Math.max(0, Math.min(dropIndex, preview.length))

        preview.splice(safeIndex, 0, {
          ...ghostTask,
          _isPreview: true, // 标记为预览状态
        } as TaskCard & { _isPreview?: boolean })

        return preview
      }

      return withoutDragged
    }

    // 场景B: 实体元素在其他列表中预览（从本列表移除）
    if (sourceZoneId === currentViewId && targetZoneId !== currentViewId) {
      // 🔥 使用决策服务判断是否保留源元素
      const sourceViewKey = viewMetadata.value.id
      const targetViewKey = targetZoneId
      
      const sourceDate = sourceViewKey.startsWith('daily::') ? sourceViewKey.split('::')[1] : null
      const targetDate = targetViewKey.startsWith('daily::') ? targetViewKey.split('::')[1] : null

      if (sourceDate && targetDate) {
        // 获取今天的日期
        const today = new Date().toISOString().split('T')[0]
        
        // 使用决策服务
        const decision = makeDragDecision(ghostTask, sourceDate, targetDate, today)
        
        console.log('🔍 [useInteractDrag] Drag decision:', decision)
        
        if (decision.keepSourceElement) {
          // 保留源元素，不移除
          return currentTasks
        }
      }

      // 否则移除源元素（标准行为）
      return currentTasks.filter((t) => t.id !== ghostTask.id)
    }

    // 其他情况：显示原始列表
    return currentTasks
  })

  /**
   * 是否正在拖动
   */
  const isDragging = computed(() => {
    const preview = dragPreviewState.value
    return preview !== null && preview.raw.sourceZoneId === viewMetadata.value.id
  })

  /**
   * 是否正在接收拖放
   */
  const isReceiving = computed(() => {
    const preview = dragPreviewState.value
    return (
      preview !== null &&
      preview.raw.targetZoneId === viewMetadata.value.id &&
      preview.raw.sourceZoneId !== viewMetadata.value.id
    )
  })

  // ==================== 拖放设置 ====================

  /**
   * 获取拖拽数据的函数
   *
   * 重要：我们必须从原始的 tasks.value 中查找任务，
   * 因为 displayTasks 可能已经被预览状态修改了
   */
  const getDragData = (element: HTMLElement): DragData => {
    const taskId = element.getAttribute('data-task-id')
    if (!taskId) {
      throw new Error('Task ID not found on draggable element')
    }

    // 🔥 关键修复：只在原始任务列表中查找
    // 不要在 displayTasks 中查找，因为它可能已经被预览状态修改
    const task = tasks.value.find((t) => t.id === taskId)

    if (!task) {
      console.error('Task lookup failed:', {
        taskId,
        originalTasksIds: tasks.value.map((t) => t.id),
        displayTasksIds: displayTasks.value.map((t) => t.id),
        viewId: viewMetadata.value.id,
        message:
          'Task not found in original tasks list. This might indicate a timing issue with DOM updates.',
      })
      throw new Error(`Task not found: ${taskId}. Check if task exists in original list.`)
    }

    const index = tasks.value.indexOf(task)

    return {
      type: 'task',
      task,
      sourceView: viewMetadata.value,
      index,
      // 🔥 V2: 传递灵活的上下文数据
      sourceContext: {
        taskIds: displayTasks.value.map((t) => t.id),
        displayTasks: displayTasks.value,
        viewKey: viewMetadata.value.id,
        // 可以添加更多数据
      },
    }
  }

  /**
   * 初始化拖放功能
   */
  const initializeDragDrop = () => {
    if (!containerRef.value) {
      console.warn('[useInteractDrag] Container ref is null, skipping initialization')
      return
    }

    // 安装可拖拽元素
    interactManager.installDraggable(draggableSelector, {
      getData: getDragData,
    })

    // 注册拖放区
    interactManager.registerDropzone(containerRef.value, {
      zoneId: viewMetadata.value.id,
      type: dropzoneType,
      onDrop,
    })
  }

  /**
   * 清理拖放功能
   */
  const cleanupDragDrop = () => {
    if (containerRef.value) {
      interactManager.unregisterDropzone(containerRef.value)
    }
  }

  // ==================== 生命周期 ====================

  onMounted(() => {
    // 延迟初始化，确保 DOM 已渲染
    setTimeout(() => {
      initializeDragDrop()
    }, 0)
  })

  onBeforeUnmount(() => {
    cleanupDragDrop()
  })

  // ==================== 返回 API ====================

  return {
    // 响应式状态
    displayTasks,
    isDragging,
    isReceiving,

    // 工具方法
    initializeDragDrop,
    cleanupDragDrop,

    // 调试信息
    getDebugInfo: () => ({
      viewId: viewMetadata.value.id,
      taskCount: tasks.value.length,
      displayTaskCount: displayTasks.value.length,
      isDragging: isDragging.value,
      isReceiving: isReceiving.value,
      previewState: dragPreviewState.value,
    }),
  }
}

// ==================== 类型导出 ====================

export type UseInteractDragReturn = ReturnType<typeof useInteractDrag>

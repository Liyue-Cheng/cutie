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
import type { DragObject, DragObjectType, TaskCard } from '@/types/dtos'
import type { DragData } from '@/infra/drag-interact/types'
import { makeDragDecision } from '@/services/dragDecisionService'
import { getTodayDateString } from '@/infra/utils/dateUtils'

/**
 * useInteractDrag 配置选项
 *
 * @template T 拖放对象的类型，默认为 DragObject 联合类型
 */
export interface UseInteractDragOptions<T = DragObject> {
  /** 视图元数据 */
  viewMetadata: Ref<ViewMetadata>

  /** 对象列表（替代 tasks） */
  items: Ref<T[]>

  /** 对象列表容器元素引用 */
  containerRef: Ref<HTMLElement | null>

  /** 可拖拽元素选择器 */
  draggableSelector: string

  /** 拖放区类型 */
  dropzoneType?: 'kanban' | 'calendar'

  /** 对象类型标识 */
  objectType: DragObjectType

  /** 获取对象ID的函数 */
  getObjectId: (item: T) => string

  /** 自定义放置处理函数 */
  onDrop?: (session: any) => Promise<void>
}

/**
 * useInteractDrag Composable
 *
 * @template T 拖放对象的类型，默认为 DragObject 联合类型
 */
export function useInteractDrag<T = DragObject>(options: UseInteractDragOptions<T>) {
  const {
    viewMetadata,
    items,
    containerRef,
    draggableSelector,
    dropzoneType = 'kanban',
    objectType,
    getObjectId,
    onDrop,
  } = options

  // ==================== 响应式状态 ====================

  /**
   * 显示的对象列表（包含预览逻辑）
   * 这是核心的响应式计算，实现了需求文档中的"实体元素"预览
   */
  const displayItems = computed<T[]>(() => {
    const preview = dragPreviewState.value
    const currentItems = items.value
    const currentViewId = viewMetadata.value.id

    // 没有预览 → 显示原始列表
    if (!preview) {
      return currentItems
    }

    const { draggedObject, objectType: previewObjectType, sourceZoneId, targetZoneId } = preview.raw
    const { dropIndex } = preview.computed
    const isSourceView = sourceZoneId === currentViewId
    const isCompact = preview.computed.isCompact === true

    // 只处理匹配的对象类型
    if (previewObjectType !== objectType) {
      return currentItems
    }

    const draggedId = getObjectId(draggedObject as T)

    const applyCompactFlag = (list: T[]): T[] => {
      if (!isSourceView || !isCompact) {
        return list
      }

      let applied = false
      const mapped = list.map((item) => {
        if (getObjectId(item) === draggedId) {
          applied = true
          return {
            ...item,
            _dragCompact: true,
          } as T & { _dragCompact?: boolean }
        }
        return item
      })

      return applied ? (mapped as T[]) : list
    }

    // 🔥 场景C: 越界回弹 (targetZoneId === null)
    // 所有列表都回到原始状态
    if (targetZoneId === null) {
      return applyCompactFlag(currentItems)
    }

    // 场景A: 实体元素在本列表中预览
    if (targetZoneId === currentViewId) {
      // 先移除被拖动的对象（如果在本列表中）
      const withoutDragged = currentItems.filter((item) => getObjectId(item) !== draggedId)

      if (dropIndex !== undefined) {
        // 插入预览位置
        const previewList = [...withoutDragged]
        const safeIndex = Math.max(0, Math.min(dropIndex, previewList.length))

        previewList.splice(safeIndex, 0, {
          ...draggedObject,
          _isPreview: true, // 标记为预览状态
          _dragCompact: preview.computed.isCompact === true,
        } as T & { _isPreview?: boolean; _dragCompact?: boolean })

        return previewList
      }

      return applyCompactFlag(withoutDragged)
    }

    // 场景B: 实体元素在其他列表中预览（从本列表移除）
    if (sourceZoneId === currentViewId && targetZoneId !== currentViewId) {
      // 🔥 特殊逻辑：仅对任务类型使用决策服务
      if (objectType === 'task') {
        const sourceViewKey = viewMetadata.value.id
        const targetViewKey = targetZoneId

        const sourceDate = sourceViewKey.startsWith('daily::') ? sourceViewKey.split('::')[1] : null
        const targetDate = targetViewKey.startsWith('daily::') ? targetViewKey.split('::')[1] : null

        if (sourceDate && targetDate) {
          // ⚠️ 使用 getTodayDateString() 获取本地日期，符合 TIME_CONVENTION.md
          const today = getTodayDateString()
          const decision = makeDragDecision(
            draggedObject as any as TaskCard,
            sourceDate,
            targetDate,
            today
          )

          console.log('🔍 [useInteractDrag] Drag decision:', decision)

          if (decision.keepSourceElement) {
            return applyCompactFlag(currentItems)
          }
        }
      }

      // 否则移除源元素（标准行为）
      return currentItems.filter((item) => getObjectId(item) !== draggedId)
    }

    // 其他情况：显示原始列表
    return currentItems
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
   * 重要：我们必须从原始的 items.value 中查找对象，
   * 因为 displayItems 可能已经被预览状态修改了
   */
  const getDragData = (element: HTMLElement): DragData<T> => {
    const objectId = element.getAttribute('data-object-id') || element.getAttribute('data-task-id')
    if (!objectId) {
      throw new Error('Object ID not found on draggable element')
    }

    // 🔥 关键修复：只在原始对象列表中查找
    // 不要在 displayItems 中查找，因为它可能已经被预览状态修改
    const item = items.value.find((item) => getObjectId(item) === objectId)

    if (!item) {
      console.error('Object lookup failed:', {
        objectId,
        objectType,
        originalItemIds: items.value.map(getObjectId),
        displayItemIds: displayItems.value.map(getObjectId),
        viewId: viewMetadata.value.id,
        message:
          'Object not found in original items list. This might indicate a timing issue with DOM updates.',
      })
      throw new Error(`Object not found: ${objectId}. Check if object exists in original list.`)
    }

    const index = items.value.indexOf(item)

    return {
      type: objectType,
      data: item,
      sourceView: viewMetadata.value,
      index,
      // 🔥 V2: 传递灵活的上下文数据
      sourceContext: {
        itemIds: displayItems.value.map(getObjectId),
        displayItems: displayItems.value,
        // 向后兼容：也提供 taskIds 和 displayTasks 字段
        taskIds: displayItems.value.map(getObjectId),
        displayTasks: displayItems.value,
        viewKey: viewMetadata.value.id,
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
    displayItems,
    isDragging,
    isReceiving,

    // 工具方法
    initializeDragDrop,
    cleanupDragDrop,

    // 调试信息
    getDebugInfo: () => ({
      viewId: viewMetadata.value.id,
      itemCount: items.value.length,
      displayItemCount: displayItems.value.length,
      objectType,
      isDragging: isDragging.value,
      isReceiving: isReceiving.value,
      previewState: dragPreviewState.value,
    }),
  }
}

// ==================== 类型导出 ====================

export type UseInteractDragReturn = ReturnType<typeof useInteractDrag>

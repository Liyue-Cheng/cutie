/**
 * useTimelineCardDrag - 时间卡片拖放 Composable
 *
 * 为 TimelineCard 提供拖放预览功能：
 * - 注册为拖放区域
 * - 监听拖放状态
 * - 在任务拖动悬浮时显示预览
 */

import { computed, onMounted, onBeforeUnmount, type Ref } from 'vue'
import { interactManager, dragPreviewState } from '@/infra/drag-interact'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'

export interface UseTimelineCardDragOptions {
  /** 日期字符串 YYYY-MM-DD */
  date: string

  /** 原始任务列表 */
  originalTasks: Ref<TaskCard[]>

  /** 容器元素引用 */
  containerRef: Ref<HTMLElement | null>

  /** 拖放处理函数（预留接口，不实际执行） */
  onDrop?: (session: any) => Promise<void>
}

export function useTimelineCardDrag(options: UseTimelineCardDragOptions) {
  const { date, originalTasks, containerRef, onDrop } = options

  // 生成唯一的 zone ID
  const zoneId = `timeline::${date}`

  /**
   * 显示的任务列表（包含预览逻辑）
   */
  const displayTasks = computed<TaskCard[]>(() => {
    const preview = dragPreviewState.value
    const currentTasks = originalTasks.value

    // 没有预览状态 → 显示原始列表
    if (!preview) {
      return currentTasks
    }

    const { draggedObject, objectType, targetZoneId } = preview.raw

    // 只处理任务对象
    if (objectType !== 'task') {
      return currentTasks
    }

    // 如果拖动目标是当前时间卡片，显示预览
    if (targetZoneId === zoneId) {
      const draggedTask = draggedObject as TaskCard

      // 检查任务是否已经在当前列表中（避免重复）
      const isAlreadyInList = currentTasks.some((task) => task.id === draggedTask.id)
      if (isAlreadyInList) {
        return currentTasks
      }

      // 在列表末尾添加预览任务
      return [
        ...currentTasks,
        {
          ...draggedTask,
          _isPreview: true, // 标记为预览状态
        } as TaskCard & { _isPreview?: boolean },
      ]
    }

    // 其他情况显示原始列表
    return currentTasks
  })

  /**
   * 是否正在接收拖放
   */
  const isReceiving = computed(() => {
    const preview = dragPreviewState.value
    return (
      preview !== null && preview.raw.objectType === 'task' && preview.raw.targetZoneId === zoneId
    )
  })

  /**
   * 初始化拖放功能
   */
  const initializeDropzone = () => {
    if (!containerRef.value) {
      logger.warn(
        LogTags.COMPONENT_TIMELINE,
        'Container ref is null, skipping dropzone initialization',
        { date }
      )
      return
    }

    // 注册为拖放区
    interactManager.registerDropzone(containerRef.value, {
      zoneId,
      type: 'kanban', // 使用 kanban 类型，与现有系统兼容
      onDrop:
        onDrop ||
        (() => {
          // 默认处理：只打印日志（预留接口）
          logger.info(LogTags.COMPONENT_TIMELINE, 'Task dropped on timeline card (preview only)', {
            date,
            zoneId,
          })
          return Promise.resolve()
        }),
    })

    logger.debug(LogTags.COMPONENT_TIMELINE, 'Timeline card dropzone registered', {
      date,
      zoneId,
    })
  }

  /**
   * 清理拖放功能
   */
  const cleanupDropzone = () => {
    if (containerRef.value) {
      interactManager.unregisterDropzone(containerRef.value)
      logger.debug(LogTags.COMPONENT_TIMELINE, 'Timeline card dropzone unregistered', {
        date,
        zoneId,
      })
    }
  }

  // ==================== 生命周期 ====================

  onMounted(() => {
    // 延迟初始化，确保 DOM 已渲染
    setTimeout(() => {
      initializeDropzone()
    }, 0)
  })

  onBeforeUnmount(() => {
    cleanupDropzone()
  })

  // ==================== 返回 API ====================

  return {
    // 响应式状态
    displayTasks,
    isReceiving,
    zoneId,

    // 工具方法
    initializeDropzone,
    cleanupDropzone,

    // 调试信息
    getDebugInfo: () => ({
      date,
      zoneId,
      originalTaskCount: originalTasks.value.length,
      displayTaskCount: displayTasks.value.length,
      isReceiving: isReceiving.value,
      previewState: dragPreviewState.value,
    }),
  }
}

// ==================== 类型导出 ====================

export type UseTimelineCardDragReturn = ReturnType<typeof useTimelineCardDrag>

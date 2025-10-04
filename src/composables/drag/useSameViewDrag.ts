/**
 * useSameViewDrag - 同看板内拖放逻辑
 *
 * 封装同一个看板内的拖放排序功能，提供实时预览
 */

import { ref, computed } from 'vue'
import type { TaskCard } from '@/types/dtos'

/**
 * 同看板拖放 Composable
 * @param getTasksFn - 获取当前任务列表的函数
 */
export function useSameViewDrag(getTasksFn: () => TaskCard[]) {
  // ==================== 状态 ====================

  /** 被拖动的任务ID */
  const draggedTaskId = ref<string | null>(null)

  /** 当前悬停的目标索引 */
  const draggedOverIndex = ref<number | null>(null)

  /** 节流控制 */
  let lastDragOverTime = 0
  const DRAG_THROTTLE_MS = 50

  // ==================== 计算属性 ====================

  /**
   * 是否正在拖动
   */
  const isDragging = computed(() => draggedTaskId.value !== null)

  /**
   * 实时重排后的任务列表（仅视觉预览）
   */
  const reorderedTasks = computed(() => {
    const tasks = [...getTasksFn()]

    if (!draggedTaskId.value || draggedOverIndex.value === null) {
      return tasks
    }

    const draggedIndex = tasks.findIndex((t) => t.id === draggedTaskId.value)
    if (draggedIndex === -1 || draggedIndex === draggedOverIndex.value) {
      return tasks
    }

    // 实时重排（仅视觉）
    const [draggedTask] = tasks.splice(draggedIndex, 1)
    if (draggedTask) {
      tasks.splice(draggedOverIndex.value, 0, draggedTask)
    }

    return tasks
  })

  // ==================== 操作方法 ====================

  /**
   * 开始拖动
   * @param taskId - 被拖动的任务ID
   */
  function startDrag(taskId: string): void {
    draggedTaskId.value = taskId
    draggedOverIndex.value = null
    lastDragOverTime = 0

    console.log('[useSameViewDrag] 🚀 Drag started:', taskId)
  }

  /**
   * 拖动经过目标位置（带节流）
   * @param targetIndex - 目标索引
   */
  function dragOver(targetIndex: number): void {
    if (!isDragging.value) return

    // 节流：限制执行频率
    const now = Date.now()
    if (now - lastDragOverTime < DRAG_THROTTLE_MS) {
      return
    }
    lastDragOverTime = now

    // 检查任务是否在当前列表中
    const tasks = getTasksFn()
    const draggedIndex = tasks.findIndex((t) => t.id === draggedTaskId.value)
    if (draggedIndex === -1) return

    draggedOverIndex.value = targetIndex
  }

  /**
   * 完成拖动，返回最终顺序
   * @returns 最终的任务ID顺序，如果没有变化返回 null
   */
  function finishDrag(): string[] | null {
    if (!isDragging.value) return null

    console.log('[useSameViewDrag] ✅ Drag finished')

    const finalOrder = reorderedTasks.value.map((t) => t.id)

    // 清理状态
    draggedTaskId.value = null
    draggedOverIndex.value = null
    lastDragOverTime = 0

    return finalOrder
  }

  /**
   * 取消拖动
   */
  function cancelDrag(): void {
    if (!isDragging.value) return

    console.log('[useSameViewDrag] ❌ Drag cancelled')

    draggedTaskId.value = null
    draggedOverIndex.value = null
    lastDragOverTime = 0
  }

  return {
    // 状态（只读）
    isDragging,
    draggedTaskId: computed(() => draggedTaskId.value),
    draggedOverIndex: computed(() => draggedOverIndex.value),
    reorderedTasks,

    // 操作方法
    startDrag,
    dragOver,
    finishDrag,
    cancelDrag,
  }
}

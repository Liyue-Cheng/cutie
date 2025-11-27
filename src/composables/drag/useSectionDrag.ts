/**
 * useSectionDrag - Section 拖放排序 Composable
 *
 * 设计：
 * - 源位置：保留原Section，降低透明度
 * - 幽灵元素：克隆标题栏并添加虚化效果
 * - 目标位置：指示线显示插入位置
 *
 * 使用原生 HTML5 Drag API + 自定义幽灵元素
 */

import { ref, onUnmounted, type Ref } from 'vue'
import type { ProjectSection } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'

export interface UseSectionDragOptions {
  /** Section 列表（响应式引用） */
  sections: Ref<ProjectSection[]>
  /** 重排序回调 */
  onReorder: (sectionId: string, prevId: string | null, nextId: string | null) => Promise<void>
}

export interface UseSectionDragReturn {
  /** 当前正在拖动的 Section */
  draggingSection: Ref<ProjectSection | null>
  /** 当前正在拖动的 Section 索引 */
  draggingIndex: Ref<number>
  /** 目标插入位置索引 */
  dropTargetIndex: Ref<number | null>
  /** 拖动开始事件处理器（需要传入标题栏元素） */
  onDragStart: (
    section: ProjectSection,
    index: number,
    event: DragEvent,
    headerElement?: HTMLElement | null
  ) => void
  /** 拖动经过事件处理器 */
  onSectionDragOver: (event: DragEvent, index: number) => void
  /** 拖动离开事件处理器 */
  onSectionDragLeave: (event: DragEvent) => void
  /** 容器拖动经过事件处理器（用于处理末尾位置） */
  onContainerDragOver: (event: DragEvent) => void
  /** 拖动结束事件处理器 */
  onDragEnd: () => void
}

export function useSectionDrag(options: UseSectionDragOptions): UseSectionDragReturn {
  const { sections, onReorder } = options

  // ========== 状态 ==========
  const draggingSection = ref<ProjectSection | null>(null)
  const draggingIndex = ref<number>(-1)
  const dropTargetIndex = ref<number | null>(null)

  // 私有状态
  let ghostElement: HTMLElement | null = null
  let mouseOffset = { x: 0, y: 0 }
  let documentDragOverHandler: ((e: DragEvent) => void) | null = null
  let documentDragEndHandler: (() => void) | null = null

  // ========== 幽灵元素管理 ==========

  /**
   * 创建标题栏克隆作为幽灵元素（带虚化效果）
   */
  function createGhost(
    section: ProjectSection,
    event: DragEvent,
    headerElement?: HTMLElement | null
  ) {
    // 移除旧的幽灵元素
    removeGhost()

    // 如果提供了标题栏元素，克隆它
    if (headerElement) {
      ghostElement = headerElement.cloneNode(true) as HTMLElement
      ghostElement.className = 'section-drag-ghost section-drag-ghost-cloned'

      // 获取原始尺寸
      const rect = headerElement.getBoundingClientRect()
      ghostElement.style.width = `${rect.width}px`

      // 计算鼠标偏移量
      mouseOffset = {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
      }
    } else {
      // 兜底：创建简化的标题栏幽灵元素
      ghostElement = document.createElement('div')
      ghostElement.className = 'section-drag-ghost'
      ghostElement.innerHTML = `
        <span class="ghost-icon">📁</span>
        <span class="ghost-title">${escapeHtml(section.title)}</span>
      `

      // 获取拖动源元素的位置
      const target = event.target as HTMLElement
      const sectionHeader = target.closest('.task-bar-header') || target
      const rect = sectionHeader.getBoundingClientRect()

      mouseOffset = {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
      }
    }

    // 设置初始位置
    ghostElement.style.left = `${event.clientX - mouseOffset.x}px`
    ghostElement.style.top = `${event.clientY - mouseOffset.y}px`

    // 添加到 body
    document.body.appendChild(ghostElement)

    // 隐藏默认拖拽图像
    const emptyImg = new Image()
    emptyImg.src = 'data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs='
    event.dataTransfer?.setDragImage(emptyImg, 0, 0)

    logger.debug(LogTags.DRAG_CROSS_VIEW, '[SectionDrag] Ghost created', {
      title: section.title,
      cloned: !!headerElement,
    })
  }

  /**
   * 更新幽灵元素位置
   */
  function updateGhostPosition(event: DragEvent) {
    if (!ghostElement) return

    // 防止位置为 0（拖拽结束时会收到 clientX/Y = 0 的事件）
    if (event.clientX === 0 && event.clientY === 0) return

    ghostElement.style.left = `${event.clientX - mouseOffset.x}px`
    ghostElement.style.top = `${event.clientY - mouseOffset.y}px`
  }

  /**
   * 移除幽灵元素
   */
  function removeGhost() {
    if (ghostElement) {
      ghostElement.remove()
      ghostElement = null
    }
  }

  // ========== 事件处理 ==========

  /**
   * 拖动开始
   */
  function onDragStart(
    section: ProjectSection,
    index: number,
    event: DragEvent,
    headerElement?: HTMLElement | null
  ) {
    // 设置状态
    draggingSection.value = section
    draggingIndex.value = index

    // 设置拖拽数据
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move'
      event.dataTransfer.setData('text/plain', section.id)
    }

    // 创建幽灵元素
    createGhost(section, event, headerElement)

    // 添加全局事件监听器
    documentDragOverHandler = (e: DragEvent) => {
      e.preventDefault()
      updateGhostPosition(e)
    }
    documentDragEndHandler = () => onDragEnd()

    document.addEventListener('dragover', documentDragOverHandler)
    document.addEventListener('dragend', documentDragEndHandler)

    logger.info(LogTags.DRAG_CROSS_VIEW, '[SectionDrag] Drag started', {
      sectionId: section.id,
      title: section.title,
      index,
    })
  }

  /**
   * 拖动经过 Section
   */
  function onSectionDragOver(event: DragEvent, index: number) {
    event.preventDefault()

    // 如果没有正在拖动的元素，忽略
    if (draggingIndex.value === -1) return

    // 如果悬停在自己上面，清除指示器
    if (index === draggingIndex.value) {
      dropTargetIndex.value = null
      return
    }

    // 计算插入位置（在元素上半部分还是下半部分）
    const target = event.currentTarget as HTMLElement
    const rect = target.getBoundingClientRect()
    const midY = rect.top + rect.height / 2

    if (event.clientY < midY) {
      // 插入到当前元素之前
      dropTargetIndex.value = index
    } else {
      // 插入到当前元素之后
      dropTargetIndex.value = index + 1
    }
  }

  /**
   * 拖动离开 Section
   */
  function onSectionDragLeave(_event: DragEvent) {
    // 不立即清除，避免闪烁
    // dropTargetIndex 会在 onSectionDragOver 中更新
  }

  /**
   * 容器拖动经过（用于处理末尾位置）
   */
  function onContainerDragOver(event: DragEvent) {
    event.preventDefault()

    // 如果没有正在拖动的元素，忽略
    if (draggingIndex.value === -1) return

    // 如果 dropTargetIndex 还没有被设置（没有悬停在任何 section 上）
    // 检查是否在最后一个 section 的下方
    const container = event.currentTarget as HTMLElement
    const lastSection = container.querySelector('.task-section:last-child')

    if (lastSection) {
      const rect = lastSection.getBoundingClientRect()
      if (event.clientY > rect.bottom) {
        dropTargetIndex.value = sections.value.length
      }
    }
  }

  /**
   * 拖动结束
   */
  function onDragEnd() {
    // 清理全局事件监听器
    if (documentDragOverHandler) {
      document.removeEventListener('dragover', documentDragOverHandler)
      documentDragOverHandler = null
    }
    if (documentDragEndHandler) {
      document.removeEventListener('dragend', documentDragEndHandler)
      documentDragEndHandler = null
    }

    // 执行重排序
    if (draggingSection.value && dropTargetIndex.value !== null) {
      const fromIndex = draggingIndex.value
      let toIndex = dropTargetIndex.value

      // 调整索引（如果从前往后拖，目标索引需要减1）
      if (fromIndex < toIndex) {
        toIndex -= 1
      }

      // 只有位置真正改变才执行
      if (fromIndex !== toIndex) {
        // 计算重排后的列表
        const reorderedSections = [...sections.value]
        const [moved] = reorderedSections.splice(fromIndex, 1)
        if (moved) {
          reorderedSections.splice(toIndex, 0, moved)

          // 计算前后邻居
          const prevSection = toIndex > 0 ? reorderedSections[toIndex - 1] : null
          const nextSection =
            toIndex < reorderedSections.length - 1 ? reorderedSections[toIndex + 1] : null

          logger.info(LogTags.DRAG_CROSS_VIEW, '[SectionDrag] Reorder', {
            sectionId: draggingSection.value.id,
            fromIndex,
            toIndex,
            prevId: prevSection?.id ?? null,
            nextId: nextSection?.id ?? null,
          })

          // 调用回调
          onReorder(draggingSection.value.id, prevSection?.id ?? null, nextSection?.id ?? null)
        }
      }
    }

    // 清理状态
    removeGhost()
    draggingSection.value = null
    draggingIndex.value = -1
    dropTargetIndex.value = null

    logger.debug(LogTags.DRAG_CROSS_VIEW, '[SectionDrag] Drag ended')
  }

  // ========== 清理 ==========

  onUnmounted(() => {
    removeGhost()
    if (documentDragOverHandler) {
      document.removeEventListener('dragover', documentDragOverHandler)
    }
    if (documentDragEndHandler) {
      document.removeEventListener('dragend', documentDragEndHandler)
    }
  })

  return {
    // 状态
    draggingSection,
    draggingIndex,
    dropTargetIndex,

    // 事件处理器
    onDragStart,
    onSectionDragOver,
    onSectionDragLeave,
    onContainerDragOver,
    onDragEnd,
  }
}

// ========== 工具函数 ==========

/**
 * 转义 HTML 特殊字符
 */
function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

import type { Directive } from 'vue'
import { useDraggable } from '../useDraggable'
import type { DraggableOptions } from '../types'

/**
 * 创建一个修改过的 PointerEvent，将 target 设置为指定元素
 */
function createModifiedPointerEvent(
  originalEvent: PointerEvent,
  targetElement: HTMLElement
): PointerEvent {
  const modifiedEvent = new PointerEvent(originalEvent.type, {
    pointerId: originalEvent.pointerId,
    bubbles: originalEvent.bubbles,
    cancelable: originalEvent.cancelable,
    clientX: originalEvent.clientX,
    clientY: originalEvent.clientY,
    screenX: originalEvent.screenX,
    screenY: originalEvent.screenY,
    button: originalEvent.button,
    buttons: originalEvent.buttons,
    relatedTarget: originalEvent.relatedTarget,
  })

  // 手动设置 target 为指定元素
  Object.defineProperty(modifiedEvent, 'target', {
    value: targetElement,
    writable: false,
  })

  return modifiedEvent
}

/**
 * c-draggable 指令
 * 使元素可拖拽
 *
 * 使用方式：
 * <div v-c-draggable="{ data: item, dataType: 'task' }">...</div>
 */
export const cDraggable: Directive<HTMLElement, DraggableOptions> = {
  mounted(el, binding) {
    const options = binding.value
    if (!options || !options.data || !options.dataType) {
      console.warn('c-draggable: 需要提供 data 和 dataType')
      return
    }

    const { startDrag } = useDraggable(options)

    // 添加拖拽样式类
    el.classList.add('draggable')

    // 设置用户选择为 none，避免拖拽时选中文本
    el.style.userSelect = 'none'
    el.style.webkitUserSelect = 'none'

    // 添加指针事件监听器
    const handlePointerDown = (event: PointerEvent) => {
      // 阻止默认行为，避免文本选择等
      event.preventDefault()

      // 创建修改过的事件，确保 target 指向拖拽元素本身
      const modifiedEvent = createModifiedPointerEvent(event, el)

      // 启动拖拽
      startDrag(modifiedEvent)
    }

    el.addEventListener('pointerdown', handlePointerDown)

    // 保存清理函数到元素上，供 unmounted 使用
    ;(el as any).__dragCleanup = () => {
      el.removeEventListener('pointerdown', handlePointerDown)
      el.classList.remove('draggable')
      el.style.userSelect = ''
      el.style.webkitUserSelect = ''
    }

    // 保存初始选项用于后续比较
    ;(el as any).__dragOptions = { ...options }
  },

  beforeUpdate(el, binding) {
    // 只有当绑定值真正发生变化时才执行更新逻辑
    const newOptions = binding.value
    const oldOptions = (el as any).__dragOptions

    // 深度比较选项是否发生变化
    if (
      oldOptions &&
      oldOptions.data === newOptions?.data &&
      oldOptions.dataType === newOptions?.dataType &&
      oldOptions.ghostComponent === newOptions?.ghostComponent &&
      JSON.stringify(oldOptions.ghostProps) === JSON.stringify(newOptions?.ghostProps)
    ) {
      // 选项未变化，跳过更新
      return
    }

    // 验证新选项
    if (!newOptions || !newOptions.data || !newOptions.dataType) {
      console.warn('c-draggable: 需要提供 data 和 dataType')
      return
    }

    // 清理旧的监听器
    if ((el as any).__dragCleanup) {
      ;(el as any).__dragCleanup()
    }

    // 重新设置
    const { startDrag } = useDraggable(newOptions)

    el.classList.add('draggable')
    el.style.userSelect = 'none'
    el.style.webkitUserSelect = 'none'

    const handlePointerDown = (event: PointerEvent) => {
      event.preventDefault()

      // 创建修改过的事件，确保 target 指向拖拽元素本身
      const modifiedEvent = createModifiedPointerEvent(event, el)

      startDrag(modifiedEvent)
    }

    el.addEventListener('pointerdown', handlePointerDown)
    ;(el as any).__dragCleanup = () => {
      el.removeEventListener('pointerdown', handlePointerDown)
      el.classList.remove('draggable')
      el.style.userSelect = ''
      el.style.webkitUserSelect = ''
    }

    // 保存当前选项用于下次比较
    ;(el as any).__dragOptions = { ...newOptions }
  },

  unmounted(el) {
    // 清理事件监听器和样式
    if ((el as any).__dragCleanup) {
      ;(el as any).__dragCleanup()
      delete (el as any).__dragCleanup
    }

    // 清理选项缓存
    delete (el as any).__dragOptions
  },
}

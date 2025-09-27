import type { Directive } from 'vue'
import { watch } from 'vue'
import { useDroppable } from '../useDroppable'
import type { DroppableOptions } from '../types'

/**
 * c-droppable 指令
 * 使元素成为可放置区域
 *
 * 使用方式：
 * <div v-c-droppable="{ acceptedDataTypes: ['task'], onDrop: handleDrop }">...</div>
 */
export const cDroppable: Directive<HTMLElement, DroppableOptions> = {
  mounted(el, binding) {
    const options = binding.value
    if (!options || !options.acceptedDataTypes || options.acceptedDataTypes.length === 0) {
      console.warn('c-droppable: 需要提供 acceptedDataTypes')
      return
    }

    const { registerDropzone, unregisterDropzone, isOver, isValidDropTarget } =
      useDroppable(options)

    // 注册放置区
    registerDropzone(el)

    // 添加放置区样式类
    el.classList.add('droppable')

    // 使用 Vue 的响应式系统来监听状态变化并更新样式
    const updateClasses = () => {
      // 移除之前的状态类
      el.classList.remove('drag-over', 'drag-valid-target')

      // 添加当前状态类
      if (isOver.value) {
        el.classList.add('drag-over')
      }
      if (isValidDropTarget.value) {
        el.classList.add('drag-valid-target')
      }
    }

    // 初始更新
    updateClasses()

    // 使用 watch 替代 requestAnimationFrame 轮询，提高性能
    const stopWatcher = watch([isOver, isValidDropTarget], updateClasses, { 
      immediate: false,
      flush: 'post' // 在 DOM 更新后执行
    })

    // 保存清理函数
    ;(el as any).__dropCleanup = () => {
      unregisterDropzone(el)
      el.classList.remove('droppable', 'drag-over', 'drag-valid-target')
      stopWatcher() // 停止 watcher
    }

    // 保存初始选项用于后续比较
    ;(el as any).__dropOptions = { ...options }
  },

  beforeUpdate(el, binding) {
    // 只有当绑定值真正发生变化时才执行更新逻辑
    const newOptions = binding.value
    const oldOptions = (el as any).__dropOptions

    // 深度比较选项是否发生变化
    if (
      oldOptions &&
      JSON.stringify(oldOptions.acceptedDataTypes) ===
        JSON.stringify(newOptions?.acceptedDataTypes) &&
      oldOptions.onDrop === newOptions?.onDrop &&
      oldOptions.onDragEnter === newOptions?.onDragEnter &&
      oldOptions.onDragOver === newOptions?.onDragOver &&
      oldOptions.onDragLeave === newOptions?.onDragLeave
    ) {
      // 选项未变化，跳过更新
      return
    }

    // 验证新选项
    if (!newOptions || !newOptions.acceptedDataTypes || newOptions.acceptedDataTypes.length === 0) {
      console.warn('c-droppable: 需要提供 acceptedDataTypes')
      return
    }

    // 清理旧的注册
    if ((el as any).__dropCleanup) {
      ;(el as any).__dropCleanup()
    }

    // 重新注册
    const { registerDropzone, unregisterDropzone, isOver, isValidDropTarget } =
      useDroppable(newOptions)

    registerDropzone(el)
    el.classList.add('droppable')

    const updateClasses = () => {
      el.classList.remove('drag-over', 'drag-valid-target')

      if (isOver.value) {
        el.classList.add('drag-over')
      }
      if (isValidDropTarget.value) {
        el.classList.add('drag-valid-target')
      }
    }

    updateClasses()

    // 使用 watch 替代 requestAnimationFrame 轮询，提高性能
    const stopWatcher2 = watch([isOver, isValidDropTarget], updateClasses, { 
      immediate: false,
      flush: 'post' // 在 DOM 更新后执行
    })
    ;(el as any).__dropCleanup = () => {
      unregisterDropzone(el)
      el.classList.remove('droppable', 'drag-over', 'drag-valid-target')
      stopWatcher2() // 停止 watcher
    }

    // 保存当前选项用于下次比较
    ;(el as any).__dropOptions = { ...newOptions }
  },

  unmounted(el) {
    // 清理注册和样式
    if ((el as any).__dropCleanup) {
      ;(el as any).__dropCleanup()
      delete (el as any).__dropCleanup
    }

    // 清理选项缓存
    delete (el as any).__dropOptions
  },
}

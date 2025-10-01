import { readonly, shallowRef, markRaw } from 'vue'
import type { Component } from 'vue'

// ... interface ContextMenuState ...

interface ContextMenuState {
  show: boolean
  x: number
  y: number
  component: Component | null
  props: Record<string, any>
}

const state = shallowRef<ContextMenuState>({
  show: false,
  x: 0,
  y: 0,
  component: null,
  props: {},
})

// 模块级别的清理函数，用于存储上一次添加的监听器的移除逻辑
let cleanupListeners: () => void = () => {}

const manager = {
  state: readonly(state),

  show(component: Component, props: Record<string, any> = {}, event?: MouseEvent) {
    event?.preventDefault()

    // --- BUG修复关键点 1 ---
    // 在设置新状态之前，先调用上一次的清理函数，确保window是干净的。
    cleanupListeners()

    state.value = {
      show: true,
      x: event?.clientX ?? 0,
      y: event?.clientY ?? 0,
      component: markRaw(component),
      props,
    }

    const hideOnClickOutside = () => {
      // 在这个函数里只做hide，不要再手动remove listener
      // 因为清理工作会由manager.hide()统一处理
      manager.hide()
    }

    // --- BUG修复关键点 2 ---
    // 定义本次show操作的清理逻辑
    cleanupListeners = () => {
      window.removeEventListener('click', hideOnClickOutside)
      window.removeEventListener('contextmenu', hideOnClickOutside)
    }

    setTimeout(() => {
      window.addEventListener('click', hideOnClickOutside)
      // 注意：这里不再使用 { once: true }，因为我们的清理是手动的
      window.addEventListener('contextmenu', hideOnClickOutside)
    }, 0)
  },

  hide() {
    if (state.value.show) {
      state.value = { ...state.value, show: false }

      // --- BUG修复关键点 3 ---
      // hide的时候，总是调用清理函数
      cleanupListeners()
      // 清理后，将清理函数重置为空，防止重复调用
      cleanupListeners = () => {}
    }
  },
}

export function useContextMenu() {
  return manager
}

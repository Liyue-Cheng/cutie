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

    // 计算初始位置
    let x = event?.clientX ?? 0
    let y = event?.clientY ?? 0

    // 🎯 屏幕边缘检测和位置调整
    // 预估菜单尺寸（可以根据实际菜单调整）
    const MENU_WIDTH = 200
    const MENU_HEIGHT = 300
    const PADDING = 8 // 距离边缘的安全距离

    const viewportWidth = window.innerWidth
    const viewportHeight = window.innerHeight

    // 检查右边缘
    if (x + MENU_WIDTH + PADDING > viewportWidth) {
      x = viewportWidth - MENU_WIDTH - PADDING
    }

    // 检查底部边缘
    if (y + MENU_HEIGHT + PADDING > viewportHeight) {
      y = viewportHeight - MENU_HEIGHT - PADDING
    }

    // 检查左边缘
    if (x < PADDING) {
      x = PADDING
    }

    // 检查顶部边缘
    if (y < PADDING) {
      y = PADDING
    }

    state.value = {
      show: true,
      x,
      y,
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

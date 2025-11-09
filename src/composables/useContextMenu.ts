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

// 存储菜单元素的引用，用于判断点击是否在菜单内部
let menuElement: HTMLElement | null = null

const manager = {
  state: readonly(state),

  show(component: Component, props: Record<string, any> = {}, event?: MouseEvent) {
    event?.preventDefault()

    // --- BUG修复关键点 1 ---
    // 在设置新状态之前，先调用上一次的清理函数，确保window是干净的。
    cleanupListeners()

    // 🎯 使用原始鼠标位置，边缘检测由 ContextMenuHost 根据实际尺寸处理
    state.value = {
      show: true,
      x: event?.clientX ?? 0,
      y: event?.clientY ?? 0,
      component: markRaw(component),
      props,
    }

    const hideOnClickOutside = (event: Event) => {
      // 🎯 检查点击是否在菜单内部
      const target = event.target as Node
      if (menuElement && menuElement.contains(target)) {
        // 点击在菜单内部，允许事件正常传播（菜单项会自己处理）
        return
      }
      
      // 🎯 点击在菜单外部：阻止事件传播，只关闭菜单
      // 使用捕获阶段确保在事件到达目标元素之前就拦截
      event.stopPropagation()
      event.preventDefault()
      
      // 关闭菜单
      manager.hide()
    }

    // --- BUG修复关键点 2 ---
    // 定义本次show操作的清理逻辑
    cleanupListeners = () => {
      window.removeEventListener('click', hideOnClickOutside, true)
      window.removeEventListener('contextmenu', hideOnClickOutside, true)
    }

    setTimeout(() => {
      // 🎯 使用捕获阶段 ({ capture: true }) 来优先拦截点击事件
      // 这样可以在事件到达目标元素之前就拦截并关闭菜单
      window.addEventListener('click', hideOnClickOutside, true)
      window.addEventListener('contextmenu', hideOnClickOutside, true)
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
      
      // 清理菜单元素引用
      menuElement = null
    }
  },
  
  /**
   * 设置菜单元素的引用（由 ContextMenuHost 调用）
   * 用于判断点击是否在菜单内部
   */
  setMenuElement(element: HTMLElement | null) {
    menuElement = element
  },
}

export function useContextMenu() {
  return manager
}

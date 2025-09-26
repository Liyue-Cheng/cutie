/**
 * Vue-Draxis - Vue 3 通用拖放管理器
 * 统一导出入口
 */

// 类型定义
export type * from './types'

// 核心协调器
export { manager as dragManager } from './drag-coordinator'

// Composables
export { useDraggable } from './useDraggable'
export { useDroppable } from './useDroppable'
export { useDragCreator } from './useDragCreator'

// 指令
export { cDraggable } from './directives/c-draggable'
export { cDroppable } from './directives/c-droppable'

// 组件
export { default as DragRenderer } from '../../components/DragRenderer.vue'

/**
 * Vue-Draxis 插件安装函数
 * 用于全局注册指令
 */
import type { App } from 'vue'
import { cDraggable } from './directives/c-draggable'
import { cDroppable } from './directives/c-droppable'

export function installVueDraxis(app: App) {
  // 注册全局指令
  app.directive('cDraggable', cDraggable)
  app.directive('cDroppable', cDroppable)
}

/**
 * 默认导出插件对象
 */
export default {
  install: installVueDraxis,
}

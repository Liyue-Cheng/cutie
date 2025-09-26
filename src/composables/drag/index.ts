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
export { vDraggable } from './directives/v-draggable'
export { vDroppable } from './directives/v-droppable'

// 组件
export { default as DragRenderer } from '../../components/DragRenderer.vue'

/**
 * Vue-Draxis 插件安装函数
 * 用于全局注册指令
 */
import type { App } from 'vue'
import { vDraggable } from './directives/v-draggable'
import { vDroppable } from './directives/v-droppable'

export function installVueDraxis(app: App) {
  // 注册全局指令
  app.directive('draggable', vDraggable)
  app.directive('droppable', vDroppable)
}

/**
 * 默认导出插件对象
 */
export default {
  install: installVueDraxis,
}

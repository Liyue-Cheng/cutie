import { defineStore } from 'pinia'
import * as core from './core'
import * as view from './view-operations'
import * as events from './event-handlers'

export const useTemplateStore = defineStore('template', () => {
  return {
    // ============================================================
    // STATE (寄存器) - 只读访问
    // ============================================================
    templates: core.templates,

    // ============================================================
    // GETTERS (导线 + 多路复用器) - 计算属性和选择器
    // ============================================================
    allTemplates: core.allTemplates,
    getTemplateById: core.getTemplateById,
    generalTemplates: core.generalTemplates,
    recurrenceTemplates: core.recurrenceTemplates,

    // ============================================================
    // MUTATIONS (寄存器写入) - 纯数据操作
    // ============================================================
    addOrUpdateTemplate_mut: core.addOrUpdateTemplate_mut,
    removeTemplate_mut: core.removeTemplate_mut,
    clearAllTemplates_mut: core.clearAllTemplates_mut,

    // ============================================================
    // DMA (Direct Memory Access) - 绕过指令流水线的数据传输
    // ============================================================
    fetchAllTemplates: view.fetchAllTemplates,

    // ============================================================
    // EVENT HANDLING (SSE 事件处理)
    // ============================================================
    initEventSubscriptions: events.initEventSubscriptions,
  }
})

// 导出类型定义
export type {
  CreateTemplatePayload,
  UpdateTemplatePayload,
  CreateTaskFromTemplatePayload,
  CreateTemplateFromTaskPayload,
} from './types'

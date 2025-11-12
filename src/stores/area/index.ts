/**
 * Area Store (RTL 架构)
 *
 * 架构说明：
 * - core.ts: State + Getters + Mutations
 * - view.ts: DMA 数据加载
 * - event-handlers.ts: SSE 事件处理
 */

import { defineStore } from 'pinia'
import * as core from './core'
import * as view from './view'
import * as events from './event-handlers'

export type { Area } from './core'

export const useAreaStore = defineStore('area', () => {
  return {
    // ========== STATE (寄存器) ==========
    areas: core.areas,

    // ========== GETTERS (传输线) ==========
    allAreas: core.allAreas,
    rootAreas: core.rootAreas,
    getChildAreas: core.getChildAreas,
    getAreaById: core.getAreaById,

    // ========== MUTATIONS (逻辑门) ==========
    addOrUpdate_mut: core.addOrUpdate_mut,
    addOrUpdateBatch_mut: core.addOrUpdateBatch_mut,
    replaceAll_mut: core.replaceAll_mut,
    remove_mut: core.remove_mut,
    clear_mut: core.clear_mut,

    // ========== DMA (数据加载) ==========
    fetchAll: view.fetchAll,
    fetchById: view.fetchById,

    // ========== EVENT HANDLING ==========
    initEventSubscriptions: events.initEventSubscriptions,
  }
})

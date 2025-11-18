/**
 * Project Store - 主入口
 *
 * RTL 硬件设计模式：
 * - Core: State + Getters + Mutations (纯数据操作)
 * - Event Handlers: SSE 事件处理（外部输入）
 */

import { defineStore } from 'pinia'
import * as core from './core'
import * as eventHandlers from './event-handlers'

export const useProjectStore = defineStore('project', () => {
  // ============================================================
  // State & Getters (从 core 导出)
  // ============================================================

  const {
    // State
    projects,
    sections,

    // Getters
    allProjects,
    activeProjects,
    completedProjects,
    getProjectById,
    getProjectsByArea,
    getSectionsByProject,
    getSectionById,
    getProjectStatsRealtime, // 🔥 实时统计

    // Mutations
    addOrUpdateProject_mut,
    addOrUpdateProjectsBatch_mut,
    replaceAllProjects_mut,
    removeProject_mut,
    addOrUpdateSection_mut,
    replaceProjectSections_mut,
    removeSection_mut,
    clearAll_mut,
  } = core

  // ============================================================
  // Event Handlers
  // ============================================================

  const {
    initEventSubscriptions,
    handleProjectCreated,
    handleProjectUpdated,
    handleProjectDeleted,
    handleSectionCreated,
    handleSectionUpdated,
    handleSectionDeleted,
  } = eventHandlers

  // ============================================================
  // 返回公共 API
  // ============================================================

  return {
    // State
    projects,
    sections,

    // Getters
    allProjects,
    activeProjects,
    completedProjects,
    getProjectById,
    getProjectsByArea,
    getSectionsByProject,
    getSectionById,
    getProjectStatsRealtime, // 🔥 实时统计

    // Mutations
    addOrUpdateProject_mut,
    addOrUpdateProjectsBatch_mut,
    replaceAllProjects_mut,
    removeProject_mut,
    addOrUpdateSection_mut,
    replaceProjectSections_mut,
    removeSection_mut,
    clearAll_mut,

    // Event Handlers
    initEventSubscriptions,
    handleProjectCreated,
    handleProjectUpdated,
    handleProjectDeleted,
    handleSectionCreated,
    handleSectionUpdated,
    handleSectionDeleted,
  }
})

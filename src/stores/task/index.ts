import { defineStore } from 'pinia'
import { createTaskCore } from './core'
import { createViewOperations } from './view-operations'
import { createCrudOperations } from './crud-operations'
import { createEventHandlers } from './event-handlers'

/**
 * Task Store V2.0 - 模块化重构版本
 *
 * 架构原则：
 * - State: 只存储最原始、最规范化的数据
 * - Actions: 负责执行操作、调用API、修改State
 * - Getters: 只负责从State中读取和计算数据，不修改State
 *
 * 模块化结构：
 * - core: 核心状态管理和计算属性
 * - view-operations: 视图相关的 API 调用
 * - crud-operations: 增删改查操作
 * - event-handlers: SSE 事件处理
 */

export const useTaskStore = defineStore('task', () => {
  // 创建核心状态管理
  const core = createTaskCore()

  // 创建视图操作
  const viewOps = createViewOperations(core)

  // 创建 CRUD 操作
  const crudOps = createCrudOperations(core)

  // 创建事件处理器
  const eventHandlers = createEventHandlers(core, crudOps)

  return {
    // ============================================================
    // STATE & GETTERS - 从核心模块导出
    // ============================================================

    // State
    tasks: core.tasks,
    isLoading: core.isLoading,
    error: core.error,

    // Getters - 所有视图的数据源
    allTasks: core.allTasks,
    stagingTasks: core.stagingTasks, // ✅ 动态过滤
    plannedTasks: core.plannedTasks, // ✅ 动态过滤
    incompleteTasks: core.incompleteTasks, // ✅ 动态过滤
    completedTasks: core.completedTasks,
    scheduledTasks: core.scheduledTasks, // @deprecated
    getTaskById: core.getTaskById,
    getTasksByProject: core.getTasksByProject,
    getTasksByArea: core.getTasksByArea,

    // ============================================================
    // ACTIONS - 从各个操作模块导出
    // ============================================================

    // 基础状态操作
    addOrUpdateTasks: core.addOrUpdateTasks,
    addOrUpdateTask: core.addOrUpdateTask,
    removeTask: core.removeTask,

    // 视图操作
    fetchAllTasks: viewOps.fetchAllTasks,
    fetchAllIncompleteTasks: viewOps.fetchAllIncompleteTasks,
    fetchPlannedTasks: viewOps.fetchPlannedTasks,
    fetchStagingTasks: viewOps.fetchStagingTasks,
    searchTasks: viewOps.searchTasks,

    // CRUD 操作
    createTask: crudOps.createTask,
    updateTask: crudOps.updateTask,
    fetchTaskDetail: crudOps.fetchTaskDetail,
    deleteTask: crudOps.deleteTask,
    completeTask: crudOps.completeTask,
    reopenTask: crudOps.reopenTask,

    // 日程管理操作
    addSchedule: crudOps.addSchedule,
    updateSchedule: crudOps.updateSchedule,
    deleteSchedule: crudOps.deleteSchedule,
    returnToStaging: crudOps.returnToStaging,

    // 事件处理
    initEventSubscriptions: eventHandlers.initEventSubscriptions,
  }
})

// 导出类型定义
export * from '@/stores/task/types'

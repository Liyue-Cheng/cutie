import { defineStore } from 'pinia'
import { createTaskCore } from './core'
import { createLoaders } from './loaders'
import { createMutations } from './mutations'
import { createEventHandlers } from './event-handlers'

/**
 * Task Store V4.0 - 纯状态容器版本
 *
 * 架构演进：
 * V1: 所有逻辑在 Store 中 (API + State)
 * V2: 模块化拆分 (core/crud/view/events)
 * V3: 职责分离 (Handler调用API, Store只管数据)
 * V4: 彻底清理，Store = 寄存器 + 导线 + 多路复用器 ← 当前版本
 *
 * RTL 架构原则：
 * - State: 寄存器 (registers)，只存储数据
 * - Mutations: 寄存器写入操作 (_mut 后缀)
 * - Getters: 导线 (wires) 和多路复用器 (_Mux 后缀)
 * - ❌ 不包含 API 调用（由 Command Handler 负责）
 * - ❌ 不包含业务逻辑（由 Command Handler 负责）
 *
 * 数据流：
 * 组件 → commandBus → Handler → API → Store.mutation
 *                                    ↓
 *                               组件响应式更新
 *
 * 类比 CPU：
 * - Store = 寄存器堆 (Register File)
 * - Mutations = 寄存器写端口 (Write Port)
 * - Getters/Mux = 寄存器读端口 + 多路复用器 (Read Port + Multiplexer)
 * - Handler = 执行单元 (Execution Unit)
 * - CommandBus = 指令译码器 (Instruction Decoder)
 */

export const useTaskStore = defineStore('task', () => {
  // 创建核心状态管理
  const core = createTaskCore()

  // 创建纯数据操作方法
  const mutations = createMutations(core)

  // 创建数据加载器（仅用于初始化数据加载）
  const loaders = createLoaders(core)

  // 创建事件处理器（SSE 事件处理）
  const eventHandlers = createEventHandlers(core)

  return {
    // ============================================================
    // STATE (寄存器) - 只读访问
    // ============================================================
    tasks: core.tasks,
    isLoading: core.isLoading,
    error: core.error,

    // ============================================================
    // GETTERS (导线 + 多路复用器) - 计算属性和选择器
    // ============================================================

    // 计算属性（导线 - wires）
    allTasks: core.allTasks,
    stagingTasks: core.stagingTasks,
    plannedTasks: core.plannedTasks,
    incompleteTasks: core.incompleteTasks,
    completedTasks: core.completedTasks,
    archivedTasks: core.archivedTasks,

    // 选择器（多路复用器 - Mux）
    getTaskById_Mux: core.getTaskById_Mux,
    getTasksByDate_Mux: core.getTasksByDate_Mux,
    getTasksByProject_Mux: core.getTasksByProject_Mux,
    getTasksByArea_Mux: core.getTasksByArea_Mux,
    getTasksByViewKey_Mux: core.getTasksByViewKey_Mux,

    // ============================================================
    // MUTATIONS (寄存器写入) - 纯数据操作
    // ============================================================

    // 任务操作
    addOrUpdateTask_mut: mutations.addOrUpdateTask_mut,
    batchAddOrUpdateTasks_mut: mutations.batchAddOrUpdateTasks_mut,
    removeTask_mut: mutations.removeTask_mut,
    batchRemoveTasks_mut: mutations.batchRemoveTasks_mut,
    patchTask_mut: mutations.patchTask_mut,
    clearAllTasks_mut: mutations.clearAllTasks_mut,

    // ============================================================
    // DMA (Direct Memory Access) - 绕过指令流水线的数据传输
    // ============================================================

    // DMA 传输方法（应用启动时批量加载数据）
    // ❌ fetchAllTasks_DMA: 已删除 - 避免循环任务导致的无限数据
    fetchAllIncompleteTasks_DMA: loaders.fetchAllIncompleteTasks_DMA,
    fetchPlannedTasks_DMA: loaders.fetchPlannedTasks_DMA,
    fetchStagingTasks_DMA: loaders.fetchStagingTasks_DMA,
    fetchDailyTasks_DMA: loaders.fetchDailyTasks_DMA,
    refreshDailyTasks_DMA: loaders.refreshDailyTasks_DMA,
    fetchTaskDetail_DMA: loaders.fetchTaskDetail_DMA,
    searchTasks_DMA: loaders.searchTasks_DMA,

    // ============================================================
    // EVENT HANDLING (SSE 事件处理)
    // ============================================================

    initEventSubscriptions: eventHandlers.initEventSubscriptions,
  }
})

// 导出类型定义
export * from '@/stores/task/types'

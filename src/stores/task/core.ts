import { ref, computed } from 'vue'
import type { TaskCard, TaskDetail } from '@/types/dtos'
import { updateMapItem, removeMapItem, createLoadingState } from '@/stores/shared'

/**
 * Task Store 核心状态管理
 *
 * 职责：
 * - 管理任务数据的单一数据源
 * - 提供基础的状态操作方法
 * - 提供计算属性和过滤器
 */

/**
 * 创建任务核心状态
 */
export function createTaskCore() {
  // ============================================================
  // STATE - 只存储最原始、最规范化的数据
  // ============================================================

  /**
   * 任务映射表 (单一数据源)
   * key: task_id
   * value: TaskCard | TaskDetail (总是保存当前最完整的信息)
   *
   * 说明：TaskDetail extends TaskCard，所以可以安全地存储两种类型
   * 当获取详情时，会用 TaskDetail 覆盖原有的 TaskCard
   */
  const tasks = ref(new Map<string, TaskCard | TaskDetail>())

  /**
   * 加载状态管理
   */
  const { isLoading, error, withLoading } = createLoadingState()

  // ============================================================
  // GETTERS - 动态过滤（所有视图的数据源）
  // ============================================================

  /**
   * 基础数组缓存层（性能优化）
   * ✅ 只转换一次 Map → Array，所有其他 getter 复用此数组
   */
  const allTasksArray = computed(() => {
    return Array.from(tasks.value.values())
  })

  /**
   * 获取所有任务（数组形式）
   */
  const allTasks = computed(() => {
    return allTasksArray.value
  })

  /**
   * Staging 任务（未安排且未完成）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   */
  const stagingTasks = computed(() => {
    return allTasksArray.value.filter(
      (task) => task.schedule_status === 'staging' && !task.is_completed
    )
  })

  /**
   * Planned 任务（已安排且未完成）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   */
  const plannedTasks = computed(() => {
    return allTasksArray.value.filter(
      (task) => task.schedule_status === 'scheduled' && !task.is_completed
    )
  })

  /**
   * 未完成的任务（所有状态）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   */
  const incompleteTasks = computed(() => {
    return allTasksArray.value.filter((task) => !task.is_completed)
  })

  /**
   * 已完成的任务
   * ✅ 性能优化：复用 allTasksArray
   */
  const completedTasks = computed(() => {
    return allTasksArray.value.filter((task) => task.is_completed)
  })

  /**
   * 已安排的任务（包括已完成和未完成）
   * @deprecated 使用 plannedTasks（只含未完成）
   */
  const scheduledTasks = computed(() => {
    return allTasksArray.value.filter((task) => task.schedule_status === 'scheduled')
  })

  /**
   * 根据 ID 获取任务（返回当前最完整的信息）
   */
  function getTaskById(id: string): TaskCard | TaskDetail | undefined {
    return tasks.value.get(id)
  }

  /**
   * 获取指定日期的任务列表（响应式）
   * ✅ 单一数据源：从 TaskStore 过滤，自动响应变化
   * ✅ 性能优化：复用 allTasksArray
   */
  const getTasksByDate = computed(() => (date: string) => {
    return allTasksArray.value.filter(
      (task) =>
        task.schedules?.some((schedule) => schedule.scheduled_day === date) ?? false
    )
  })

  /**
   * 根据项目 ID 获取任务列表
   * ✅ 性能优化：复用 allTasksArray
   */
  const getTasksByProject = computed(() => {
    return (projectId: string) => {
      return allTasksArray.value.filter((task) => task.project_id === projectId)
    }
  })

  /**
   * 根据区域 ID 获取任务列表
   * ✅ 性能优化：复用 allTasksArray
   */
  const getTasksByArea = computed(() => {
    return (areaId: string) => {
      return allTasksArray.value.filter((task) => task.area_id === areaId)
    }
  })

  // ============================================================
  // ACTIONS - 基础状态操作
  // ============================================================

  /**
   * 批量添加或更新任务（单一数据源）
   * 使用 shared 工具进行状态更新
   */
  function addOrUpdateTasks(newTasks: (TaskCard | TaskDetail)[]) {
    for (const task of newTasks) {
      if (!task || !task.id) {
        console.warn('[TaskCore] Skipping task without ID', task)
        continue
      }

      // 正确的做法：直接用服务器返回的权威数据进行设置
      // tasks.value 是一个响应式 Map，调用 .set() 会被 Vue 侦测到
      // Vue 会自动将新设置的 task 对象转换为响应式代理
      updateMapItem(tasks, task.id, task)
    }
  }

  /**
   * 添加或更新单个任务
   */
  function addOrUpdateTask(task: TaskCard | TaskDetail) {
    addOrUpdateTasks([task])
  }

  /**
   * 从 state 中移除任务
   */
  function removeTask(id: string) {
    removeMapItem(tasks, id)
  }

  return {
    // State
    tasks,
    isLoading,
    error,
    withLoading,

    // Getters
    allTasks,
    allTasksArray,
    stagingTasks,
    plannedTasks,
    incompleteTasks,
    completedTasks,
    scheduledTasks,
    getTaskById,
    getTasksByDate,
    getTasksByProject,
    getTasksByArea,

    // Actions
    addOrUpdateTasks,
    addOrUpdateTask,
    removeTask,
  }
}

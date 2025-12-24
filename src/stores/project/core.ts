/**
 * Project Store - Core (RTL 架构)
 *
 * 职责：管理 Project 和 ProjectSection 数据的核心状态和 mutations
 *
 * RTL 架构：
 * - Register (寄存器): 纯响应式状态，只读
 * - Transmission (传输线): Getters，计算属性
 * - Logic (逻辑门): Mutations，纯数据操作（_mut 后缀）
 */

import { ref, computed } from 'vue'
import type { ProjectCard, ProjectSection } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'

// ============================================================
// STATE (寄存器)
// ============================================================

export const projects = ref(new Map<string, ProjectCard>())
export const sections = ref(new Map<string, ProjectSection>())

// ============================================================
// GETTERS (传输线)
// ============================================================

export const allProjects = computed(() => {
  return Array.from(projects.value.values()).sort(
    (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
  )
})

export const activeProjects = computed(() => {
  return allProjects.value.filter((p) => p.status === 'ACTIVE')
})

export const completedProjects = computed(() => {
  return allProjects.value.filter((p) => p.status === 'COMPLETED')
})

export const getProjectById = computed(() => {
  return (id: string): ProjectCard | undefined => {
    return projects.value.get(id)
  }
})

export const getProjectsByArea = computed(() => {
  return (areaId: string): ProjectCard[] => {
    return allProjects.value.filter((p) => p.area_id === areaId)
  }
})

export const getSectionsByProject = computed(() => {
  return (projectId: string): ProjectSection[] => {
    const projectSections = Array.from(sections.value.values()).filter(
      (s) => s.project_id === projectId
    )
    // 按 sort_order 排序
    return projectSections.sort((a, b) => {
      if (!a.sort_order && !b.sort_order) return 0
      if (!a.sort_order) return 1
      if (!b.sort_order) return -1
      return a.sort_order.localeCompare(b.sort_order)
    })
  }
})

export const getSectionById = computed(() => {
  return (id: string): ProjectSection | undefined => {
    return sections.value.get(id)
  }
})

/**
 * 前端实时计算项目统计
 *
 * 基于 task store 中的任务数据响应式计算项目的任务统计
 * 任务变化时统计会自动更新，无需手动维护
 *
 * ⚠️ 重要：
 * - 过滤已删除、EXPIRE 类型过期的循环任务
 * - 对循环任务去重（每个循环规则只计一个未完成任务）
 */
export const getProjectStatsRealtime = computed(() => {
  const taskStore = useTaskStore()

  return (projectId: string): { total: number; completed: number } => {
    const allTasks = Array.from(taskStore.tasks.values())
    const today = new Date().toISOString().split('T')[0]!

    // 1. 基础过滤：项目匹配 + 未删除 + 未归档
    const projectTasks = allTasks.filter(
      (task) => task.project_id === projectId && !task.is_deleted && !task.is_archived
    )

    // 2. 过滤 EXPIRE 类型的过期循环任务
    const filteredTasks = projectTasks.filter(
      (task) => !taskStore.isExpiredRecurringTask(task, today)
    )

    // 3. 对循环任务去重（每个循环规则只计一个未完成任务）
    const deduplicatedTasks = taskStore.deduplicateRecurringTasks(filteredTasks)

    // 4. 统计完成和总数
    const completed = deduplicatedTasks.filter((task) => task.is_completed).length
    const total = deduplicatedTasks.length

    return { total, completed }
  }
})

// ============================================================
// MUTATIONS (逻辑门) - Projects
// ============================================================

/**
 * 添加或更新单个 Project
 */
export function addOrUpdateProject_mut(project: ProjectCard) {
  const newMap = new Map(projects.value)
  newMap.set(project.id, project)
  projects.value = newMap
}

/**
 * 批量添加或更新 Projects
 */
export function addOrUpdateProjectsBatch_mut(newProjects: ProjectCard[]) {
  const newMap = new Map(projects.value)
  for (const project of newProjects) {
    newMap.set(project.id, project)
  }
  projects.value = newMap
}

/**
 * 替换所有 Projects（用于初始加载）
 */
export function replaceAllProjects_mut(newProjects: ProjectCard[]) {
  const newMap = new Map<string, ProjectCard>()
  for (const project of newProjects) {
    newMap.set(project.id, project)
  }
  projects.value = newMap
}

/**
 * 删除单个 Project
 */
export function removeProject_mut(id: string) {
  const newMap = new Map(projects.value)
  newMap.delete(id)
  projects.value = newMap
}

// ============================================================
// MUTATIONS (逻辑门) - Sections
// ============================================================

/**
 * 添加或更新单个 Section
 */
export function addOrUpdateSection_mut(section: ProjectSection) {
  const newMap = new Map(sections.value)
  newMap.set(section.id, section)
  sections.value = newMap
}

/**
 * 批量替换项目的所有 Sections
 */
export function replaceProjectSections_mut(newSections: ProjectSection[]) {
  const newMap = new Map(sections.value)
  // 删除相同project_id的旧sections，添加新sections
  if (newSections.length > 0) {
    const firstSection = newSections[0]
    if (firstSection) {
      const projectId = firstSection.project_id
      Array.from(newMap.values())
        .filter((s) => s.project_id === projectId)
        .forEach((s) => newMap.delete(s.id))
    }
  }
  for (const section of newSections) {
    newMap.set(section.id, section)
  }
  sections.value = newMap
}

/**
 * 删除单个 Section
 */
export function removeSection_mut(id: string) {
  const newMap = new Map(sections.value)
  newMap.delete(id)
  sections.value = newMap
}

/**
 * 清空所有数据
 */
export function clearAll_mut() {
  projects.value = new Map()
  sections.value = new Map()
}

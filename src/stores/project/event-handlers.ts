/**
 * Project Store - SSE Event Handlers
 *
 * 职责：处理来自后端的 SSE 事件，更新本地状态
 */

import type { ProjectCard, ProjectSection, TaskCard, TimeBlockView } from '@/types/dtos'
import * as core from './core'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 删除项目事件的载荷类型
 */
interface DeleteProjectPayload {
  id: string
  side_effects?: {
    deleted_tasks?: TaskCard[]
    deleted_time_blocks?: TimeBlockView[]
  }
}

/**
 * 初始化事件订阅（v4.0: 通过 INT 中断管理器）
 */
export function initEventSubscriptions() {
  import('@/cpu/interrupt/InterruptHandler').then(({ interruptHandler }) => {
    // Project 事件
    interruptHandler.on('project.created', (event) => {
      handleProjectCreated(event.payload as ProjectCard)
    })

    interruptHandler.on('project.updated', (event) => {
      handleProjectUpdated(event.payload as ProjectCard)
    })

    interruptHandler.on('project.deleted', (event) => {
      handleProjectDeleted(event.payload as DeleteProjectPayload)
    })

    // ProjectSection 事件
    interruptHandler.on('project_section.created', (event) => {
      handleSectionCreated(event.payload as ProjectSection)
    })

    interruptHandler.on('project_section.updated', (event) => {
      handleSectionUpdated(event.payload as ProjectSection)
    })

    interruptHandler.on('project_section.deleted', (event) => {
      handleSectionDeleted(event.payload as { id: string; project_id: string })
    })

    logger.info(LogTags.STORE_PROJECT, 'Project event subscriptions initialized (v4.0 - via INT)')
  })
}

/**
 * 处理 project.created 事件
 */
export function handleProjectCreated(payload: ProjectCard) {
  logger.debug(LogTags.STORE_PROJECT, 'SSE: project.created', { projectId: payload.id })
  core.addOrUpdateProject_mut(payload)
}

/**
 * 处理 project.updated 事件
 */
export function handleProjectUpdated(payload: ProjectCard) {
  logger.debug(LogTags.STORE_PROJECT, 'SSE: project.updated', { projectId: payload.id })
  core.addOrUpdateProject_mut(payload)
}

/**
 * 处理 project.deleted 事件
 *
 * 1. 删除项目本身
 * 2. 删除关联的 sections（由 removeProject_mut 处理）
 * 3. 删除关联的 tasks
 * 4. 删除关联的 time_blocks
 */
export function handleProjectDeleted(payload: DeleteProjectPayload) {
  logger.debug(LogTags.STORE_PROJECT, 'SSE: project.deleted', {
    projectId: payload.id,
    deletedTasksCount: payload.side_effects?.deleted_tasks?.length ?? 0,
    deletedTimeBlocksCount: payload.side_effects?.deleted_time_blocks?.length ?? 0,
  })

  // 1. 删除项目（会同时删除关联的 sections）
  core.removeProject_mut(payload.id)

  // 2. 删除关联的 tasks
  if (payload.side_effects?.deleted_tasks) {
    import('../task').then(({ useTaskStore }) => {
      const taskStore = useTaskStore()
      for (const task of payload.side_effects!.deleted_tasks!) {
        taskStore.removeTask_mut(task.id)
      }
    })
  }

  // 3. 删除关联的 time_blocks
  if (payload.side_effects?.deleted_time_blocks) {
    import('../timeblock').then(({ useTimeBlockStore }) => {
      const timeBlockStore = useTimeBlockStore()
      for (const block of payload.side_effects!.deleted_time_blocks!) {
        timeBlockStore.removeTimeBlock_mut(block.id)
      }
    })
  }
}

/**
 * 处理 project_section.created 事件
 */
export function handleSectionCreated(payload: ProjectSection) {
  logger.debug(LogTags.STORE_PROJECT, 'SSE: project_section.created', { sectionId: payload.id })
  core.addOrUpdateSection_mut(payload)
}

/**
 * 处理 project_section.updated 事件
 */
export function handleSectionUpdated(payload: ProjectSection) {
  logger.debug(LogTags.STORE_PROJECT, 'SSE: project_section.updated', { sectionId: payload.id })
  core.addOrUpdateSection_mut(payload)
}

/**
 * 处理 project_section.deleted 事件
 */
export function handleSectionDeleted(payload: { id: string; project_id: string }) {
  logger.debug(LogTags.STORE_PROJECT, 'SSE: project_section.deleted', { sectionId: payload.id })
  core.removeSection_mut(payload.id)
}

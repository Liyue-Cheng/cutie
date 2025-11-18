/**
 * Project Store - SSE Event Handlers
 *
 * 职责：处理来自后端的 SSE 事件，更新本地状态
 */

import type { ProjectCard, ProjectSection } from '@/types/dtos'
import * as core from './core'
import { logger, LogTags } from '@/infra/logging/logger'

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
      handleProjectDeleted(event.payload as { id: string })
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
 */
export function handleProjectDeleted(payload: { id: string }) {
  logger.debug(LogTags.STORE_PROJECT, 'SSE: project.deleted', { projectId: payload.id })
  core.removeProject_mut(payload.id)
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

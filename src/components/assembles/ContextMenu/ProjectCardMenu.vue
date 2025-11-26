<template>
  <ContextMenu>
    <MenuItem @click="handleAction('edit')">编辑项目</MenuItem>
    <MenuItem divider @click="handleAction('add-section')">新增节</MenuItem>
    <MenuItem divider variant="danger" @click="handleAction('delete')">删除项目</MenuItem>
  </ContextMenu>
</template>

<script setup lang="ts">
import ContextMenu from '@/components/assembles/ContextMenu/shared/CuteContextMenu.vue'
import MenuItem from '@/components/assembles/ContextMenu/shared/CuteMenuItem.vue'
import type { ProjectCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'

const props = defineProps<{
  project: ProjectCard
  onEdit?: () => void
  onAddSection?: () => void
}>()

const emit = defineEmits(['close'])

const handleAction = async (action: 'edit' | 'add-section' | 'delete') => {
  if (action === 'edit') {
    if (props.onEdit) {
      props.onEdit()
    }
    logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Project edit requested', {
      projectId: props.project.id,
    })
  } else if (action === 'add-section') {
    if (props.onAddSection) {
      props.onAddSection()
    }
    logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Project add section requested', {
      projectId: props.project.id,
    })
  } else if (action === 'delete') {
    try {
      await pipeline.dispatch('project.delete', { id: props.project.id })
      logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Project deleted', {
        projectId: props.project.id,
        name: props.project.name,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Failed to delete project',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  }

  emit('close')
}
</script>

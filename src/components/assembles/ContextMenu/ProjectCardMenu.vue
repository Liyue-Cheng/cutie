<template>
  <ContextMenu>
    <MenuItem @click="handleAction('edit')">{{ $t('project.action.edit') }}</MenuItem>
    <MenuItem divider @click="handleAction('add-section')">{{ $t('project.action.addSection') }}</MenuItem>
    <MenuItem divider variant="danger" @click="handleAction('delete')">{{ $t('project.action.delete') }}</MenuItem>
  </ContextMenu>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
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

const { t } = useI18n()

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

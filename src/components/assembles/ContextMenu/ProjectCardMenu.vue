<template>
  <ContextMenu>
    <!-- 主要操作 -->
    <MenuItem icon="Pencil" @click="handleAction('edit')">{{ $t('project.action.edit') }}</MenuItem>
    <MenuItem icon="FolderPlus" @click="handleAction('add-section')">{{ $t('project.action.addSection') }}</MenuItem>
    <!-- 危险操作 -->
    <MenuItem divider icon="Trash2" variant="danger" @click="handleAction('delete')">{{ $t('project.action.delete') }}</MenuItem>
  </ContextMenu>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import ContextMenu from '@/components/assembles/ContextMenu/shared/CuteContextMenu.vue'
import MenuItem from '@/components/assembles/ContextMenu/shared/CuteMenuItem.vue'
import type { ProjectCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { dialog } from '@/composables/useDialog'

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
    // 确认删除
    const confirmed = await dialog.confirm(t('project.confirm.delete'))
    if (!confirmed) {
      emit('close')
      return
    }

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

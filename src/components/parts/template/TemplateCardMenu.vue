<template>
  <CuteContextMenu>
    <CuteMenuItem @click="handleAction('edit')">编辑模板</CuteMenuItem>
    <CuteMenuDivider />
    <CuteMenuItem variant="danger" @click="handleAction('delete')">删除模板</CuteMenuItem>
  </CuteContextMenu>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import CuteContextMenu from '@/components/parts/CuteContextMenu.vue'
import CuteMenuItem from '@/components/parts/CuteMenuItem.vue'
import CuteMenuDivider from '@/components/parts/CuteMenuDivider.vue'
import type { Template } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'

const props = defineProps<{
  template: Template
  onOpenEditor?: () => void
}>()

const emit = defineEmits(['close'])

const handleAction = async (action: 'edit' | 'delete') => {
  if (action === 'delete') {
    try {
      await pipeline.dispatch('template.delete', { id: props.template.id })
      logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Template deleted', {
        templateId: props.template.id,
        title: props.template.title,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Failed to delete template',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'edit') {
    if (props.onOpenEditor) {
      props.onOpenEditor()
    }
    logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Template action', {
      action,
      templateId: props.template.id,
    })
  }

  emit('close')
}
</script>

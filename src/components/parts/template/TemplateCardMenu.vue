<template>
  <div class="context-menu">
    <button class="menu-button" @click="handleAction('edit')">编辑模板</button>
    <div class="divider"></div>
    <button class="menu-button delete" @click="handleAction('delete')">删除模板</button>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import type { Template } from '@/types/dtos'
import { useTemplateStore } from '@/stores/template'
import { logger, LogTags } from '@/infra/logging/logger'

const props = defineProps<{
  template: Template
  onOpenEditor?: () => void
}>()

const emit = defineEmits(['close'])

const templateStore = useTemplateStore()

const handleAction = async (action: 'edit' | 'delete') => {
  if (action === 'delete') {
    try {
      await templateStore.deleteTemplate(props.template.id)
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

<style scoped>
.context-menu {
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
  border-radius: 4px;
  background-color: #fff;
  display: flex;
  flex-direction: column;
  padding: 4px;
  min-width: 140px;
}

.menu-button {
  background: none;
  border: none;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 14px;
  color: #333;
  border-radius: 4px;
  transition: background-color 0.2s;
  text-align: left;
  width: 100%;
}

.menu-button:hover {
  background-color: #f5f5f5;
}

.menu-button.delete {
  color: #d03050;
}

.menu-button.delete:hover {
  background-color: #ffe8ee;
}

.divider {
  width: 100%;
  height: 1px;
  background-color: #e0e0e0;
  margin: 4px 0;
}
</style>

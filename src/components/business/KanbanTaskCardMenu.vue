<template>
  <n-card class="context-menu" content-style="padding: 5px;">
    <n-button text @click="handleAction('edit')">编辑任务</n-button>
    <n-divider vertical />
    <n-button text @click="handleAction('delete')">删除任务</n-button>
  </n-card>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import { NCard, NButton, NDivider } from 'naive-ui'
import type { Task } from '@/types/models'

const props = defineProps<{
  task: Task
}>()

const emit = defineEmits(['close'])

const handleAction = (action: 'edit' | 'delete') => {
  console.log(`Action: ${action} on task:`, props.task)
  // Here you would typically emit an event or call a store action
  // For example: taskStore.deleteTask(props.task.id);
  emit('close') // Close the context menu after action
}
</script>

<style scoped>
.context-menu {
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
  border-radius: 4px;
  background-color: #fff;
  display: inline-block;
}
</style>

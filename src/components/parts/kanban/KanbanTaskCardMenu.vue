<template>
  <n-card class="context-menu" content-style="padding: 5px;">
    <n-button text @click="handleAction('edit')">编辑任务</n-button>
    <n-divider vertical />
    <n-button text type="error" @click="handleAction('delete')">删除任务</n-button>
  </n-card>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import { NCard, NButton, NDivider } from 'naive-ui'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'

const props = defineProps<{
  task: TaskCard
}>()

const emit = defineEmits(['close'])

const taskStore = useTaskStore()

const handleAction = async (action: 'edit' | 'delete') => {
  if (action === 'delete') {
    try {
      await taskStore.deleteTask(props.task.id)
      console.log(`任务 "${props.task.title}" 已删除`)
    } catch (error) {
      console.error('删除任务失败:', error)
    }
  } else if (action === 'edit') {
    console.log(`Action: ${action} on task:`, props.task)
    // TODO: 实现编辑功能
  }

  emit('close')
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

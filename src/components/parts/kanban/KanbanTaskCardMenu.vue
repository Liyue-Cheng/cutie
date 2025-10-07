<template>
  <div class="context-menu">
    <button class="menu-button" @click="handleAction('edit')">编辑任务</button>
    <div class="divider"></div>
    <button v-if="!task.is_archived" class="menu-button" @click="handleAction('archive')">
      归档任务
    </button>
    <button v-else class="menu-button" @click="handleAction('unarchive')">取消归档</button>
    <div class="divider"></div>
    <button class="menu-button delete" @click="handleAction('delete')">删除任务</button>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { useTaskOperations } from '@/composables/useTaskOperations'

const props = defineProps<{
  task: TaskCard
}>()

const emit = defineEmits(['close'])

const taskOps = useTaskOperations()

const handleAction = async (action: 'edit' | 'delete' | 'archive' | 'unarchive') => {
  if (action === 'delete') {
    try {
      const success = await taskOps.deleteTask(props.task.id)
      if (success) {
        console.log(`任务 "${props.task.title}" 已删除`)
      }
    } catch (error) {
      console.error('删除任务失败:', error)
    }
  } else if (action === 'archive') {
    try {
      const success = await taskOps.archiveTask(props.task.id)
      if (success) {
        console.log(`任务 "${props.task.title}" 已归档`)
      }
    } catch (error) {
      console.error('归档任务失败:', error)
    }
  } else if (action === 'unarchive') {
    try {
      const success = await taskOps.unarchiveTask(props.task.id)
      if (success) {
        console.log(`任务 "${props.task.title}" 已取消归档`)
      }
    } catch (error) {
      console.error('取消归档任务失败:', error)
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

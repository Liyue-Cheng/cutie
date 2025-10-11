<script setup lang="ts">
import { computed, watch } from 'vue'
import { useUIStore } from '@/stores/ui'
import { useRecurrenceStore } from '@/stores/recurrence'
import RecurrenceEditDialog from './RecurrenceEditDialog.vue'
import type { TaskRecurrence } from '@/types/dtos'

/**
 * 全局循环规则编辑对话框
 *
 * 监听 UI Store 的状态，在任何地方都可以通过 uiStore.openRecurrenceEditDialog() 打开
 */

const uiStore = useUIStore()
const recurrenceStore = useRecurrenceStore()

// 当前正在编辑的循环规则
const editingRecurrence = computed<TaskRecurrence | null>(() => {
  const recurrenceId = uiStore.recurrenceEditDialogId
  if (!recurrenceId) return null
  return recurrenceStore.getRecurrenceById(recurrenceId) || null
})

// 监听 UI Store 状态变化，自动加载循环规则数据
watch(
  () => uiStore.recurrenceEditDialogId,
  async (recurrenceId) => {
    if (recurrenceId) {
      // 确保循环规则数据已加载
      await recurrenceStore.fetchAllRecurrences()
    }
  }
)

function handleClose() {
  uiStore.closeRecurrenceEditDialog()
}

function handleSuccess() {
  // 编辑成功，对话框会自动关闭
  uiStore.closeRecurrenceEditDialog()
}
</script>

<template>
  <!-- 使用 Teleport 将对话框渲染到 body 下，确保 z-index 正确 -->
  <Teleport to="body">
    <RecurrenceEditDialog
      :recurrence="editingRecurrence"
      :open="uiStore.isRecurrenceEditDialogOpen"
      @close="handleClose"
      @success="handleSuccess"
    />
  </Teleport>
</template>

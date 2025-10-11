<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRecurrenceStore } from '@/stores/recurrence'
import { useViewStore } from '@/stores/view'
import { useTemplateStore } from '@/stores/template'
import RecurrenceRuleCard from './RecurrenceRuleCard.vue'
import RecurrenceEditDialog from './RecurrenceEditDialog.vue'
import type { TaskRecurrence } from '@/types/dtos'

const recurrenceStore = useRecurrenceStore()
const viewStore = useViewStore()
const templateStore = useTemplateStore()

// 编辑对话框状态
const showEditDialog = ref(false)
const editingRecurrenceId = ref<string | null>(null)

// 当前正在编辑的循环规则
const editingRecurrence = computed<TaskRecurrence | null>(() => {
  if (!editingRecurrenceId.value) return null
  return recurrenceStore.getRecurrenceById(editingRecurrenceId.value) || null
})

onMounted(async () => {
  // 加载所有模板和循环规则
  await Promise.all([templateStore.fetchAllTemplates(), recurrenceStore.fetchAllRecurrences()])
})

async function handleToggleActive(id: string, currentStatus: boolean) {
  try {
    await recurrenceStore.updateRecurrence(id, { is_active: !currentStatus })
    await viewStore.refreshAllMountedDailyViews()
  } catch (error) {
    console.error('Failed to toggle recurrence:', error)
    alert('操作失败，请重试')
  }
}

function handleEdit(id: string) {
  editingRecurrenceId.value = id
  showEditDialog.value = true
}

async function handleDelete(id: string) {
  try {
    await recurrenceStore.deleteRecurrence(id)
    await viewStore.refreshAllMountedDailyViews()
  } catch (error) {
    console.error('Failed to delete recurrence:', error)
    alert('删除失败，请重试')
  }
}

function handleEditDialogClose() {
  showEditDialog.value = false
  editingRecurrenceId.value = null
}

function handleEditSuccess() {
  // 编辑成功，对话框会自动关闭
  console.log('Recurrence updated successfully')
}
</script>

<template>
  <div class="recurrence-board">
    <div class="board-header">
      <h2>循环任务</h2>
      <div class="count-badge">{{ recurrenceStore.allRecurrences.length }}</div>
    </div>

    <div v-if="recurrenceStore.allRecurrences.length === 0" class="empty-state">
      <div class="empty-icon">🔄</div>
      <p class="empty-text">暂无循环任务规则</p>
      <p class="empty-hint">在任务卡片菜单中选择"设置为循环"来创建</p>
    </div>

    <div v-else class="recurrence-list">
      <RecurrenceRuleCard
        v-for="recurrence in recurrenceStore.allRecurrences"
        :key="recurrence.id"
        :recurrence="recurrence"
        @toggle-active="handleToggleActive"
        @edit="handleEdit"
        @delete="handleDelete"
      />
    </div>

    <!-- 编辑对话框 -->
    <RecurrenceEditDialog
      :recurrence="editingRecurrence"
      :open="showEditDialog"
      @close="handleEditDialogClose"
      @success="handleEditSuccess"
    />
  </div>
</template>

<style scoped>
.recurrence-board {
  padding: 20px;
  background: #f8f9fa;
  min-height: 100vh;
}

.board-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.board-header h2 {
  margin: 0;
  font-size: 1.8em;
  color: #333;
}

.count-badge {
  background: #007aff;
  color: white;
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 0.9em;
  font-weight: 600;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  background: white;
  border-radius: 12px;
  border: 2px dashed #ddd;
}

.empty-icon {
  font-size: 4em;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-text {
  font-size: 1.2em;
  color: #666;
  margin: 0 0 8px;
}

.empty-hint {
  font-size: 0.9em;
  color: #999;
  margin: 0;
}

.recurrence-list {
  display: flex;
  flex-direction: column;
}
</style>

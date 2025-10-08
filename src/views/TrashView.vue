<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useTrashStore } from '@/stores/trash'
import KanbanTaskCard from '@/components/parts/kanban/KanbanTaskCard.vue'
import type { TaskCard } from '@/types/dtos'

const trashStore = useTrashStore()

onMounted(async () => {
  console.log('[TrashView] Loading trash...')
  try {
    await trashStore.fetchTrash()
    console.log('[TrashView] Loaded', trashStore.trashedTaskCount, 'deleted tasks')
  } catch (error) {
    console.error('[TrashView] Failed to fetch trash:', error)
  }
})

// 任务总数
const taskCount = computed(() => trashStore.trashedTaskCount)

// 格式化删除时间
function formatDeletedAt(deletedAt: string | null): string {
  if (!deletedAt) return '未知时间'

  const date = new Date(deletedAt)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return '刚刚'
  if (diffMins < 60) return `${diffMins} 分钟前`
  if (diffHours < 24) return `${diffHours} 小时前`
  if (diffDays < 7) return `${diffDays} 天前`

  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
}

// 恢复任务
async function handleRestore(task: TaskCard) {
  if (!confirm(`确定要恢复任务"${task.title}"吗？`)) return

  try {
    await trashStore.restoreTask(task.id)
    console.log('[TrashView] Task restored:', task.id)
  } catch (error) {
    console.error('[TrashView] Failed to restore task:', error)
    alert('恢复失败，请重试')
  }
}

// 彻底删除任务
async function handlePermanentlyDelete(task: TaskCard) {
  if (!confirm(`确定要彻底删除任务"${task.title}"吗？此操作不可恢复！`)) return

  try {
    await trashStore.permanentlyDeleteTask(task.id)
    console.log('[TrashView] Task permanently deleted:', task.id)
  } catch (error) {
    console.error('[TrashView] Failed to permanently delete task:', error)
    alert('删除失败，请重试')
  }
}

// 清空回收站
async function handleEmptyTrash() {
  if (!confirm('确定要清空回收站吗？这将彻底删除所有任务，此操作不可恢复！')) {
    return
  }

  try {
    const deletedCount = await trashStore.emptyTrash({ olderThanDays: 0 })
    console.log('[TrashView] Trash emptied, deleted count:', deletedCount)
    alert(`已清空回收站，删除了 ${deletedCount} 个任务`)
  } catch (error) {
    console.error('[TrashView] Failed to empty trash:', error)
    alert('清空失败，请重试')
  }
}

// 禁止打开编辑器
function handleOpenEditor() {
  // 回收站中的任务不允许编辑
  console.log('[TrashView] Cannot edit deleted tasks')
}
</script>

<template>
  <div class="trash-column">
    <div class="column-header">
      <div class="header-title">
        <h3>回收站</h3>
        <span class="task-count">{{ taskCount }}</span>
      </div>
      <button v-if="taskCount > 0" class="empty-trash-btn" @click="handleEmptyTrash">
        清空回收站
      </button>
    </div>

    <div class="column-content">
      <div v-if="trashStore.trashedTaskCount === 0" class="empty-state">
        <p>回收站是空的</p>
      </div>

      <div v-else class="tasks-list">
        <div v-for="task in trashStore.allTrashedTasks" :key="task.id" class="task-wrapper">
          <!-- 删除时间指示器 -->
          <div class="deleted-time-indicator">
            <span class="deleted-time-text">删除于 {{ formatDeletedAt(task.deleted_at) }}</span>
          </div>

          <!-- 任务卡片 -->
          <div class="task-card-container">
            <KanbanTaskCard
              :task="task"
              :view-metadata="{
                type: 'custom',
                id: 'trash',
                config: {
                  filter: (t) => t.is_deleted,
                  metadata: { isTrash: true },
                },
              }"
              :class="{ 'deleted-task': true }"
              @open-editor="handleOpenEditor"
            />
          </div>

          <!-- 操作按钮 -->
          <div class="task-actions">
            <button class="action-btn restore-btn" @click="handleRestore(task)">
              <span class="btn-icon">↩</span>
              <span class="btn-text">恢复</span>
            </button>
            <button class="action-btn delete-btn" @click="handlePermanentlyDelete(task)">
              <span class="btn-icon">🗑</span>
              <span class="btn-text">彻底删除</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.trash-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
}

.column-header {
  padding: 1.5rem 1.5rem 1rem;
  border-bottom: 1px solid var(--color-border-default);
  background-color: var(--color-background-content);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  margin-bottom: 0.8rem;
}

.header-title h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2.4rem;
  height: 2.4rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-tertiary);
  background-color: var(--color-background-hover);
  border-radius: 1.2rem;
}

.empty-trash-btn {
  padding: 0.6rem 1.2rem;
  font-size: 1.2rem;
  font-weight: 500;
  color: #fff;
  background-color: #ef4444;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.empty-trash-btn:hover {
  background-color: #dc2626;
}

.column-content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 2rem;
}

.empty-state p {
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
  text-align: center;
}

.tasks-list {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.task-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

/* 删除时间指示器 */
.deleted-time-indicator {
  display: flex;
  align-items: center;
  padding: 0 0.8rem;
}

.deleted-time-text {
  font-size: 1.1rem;
  color: var(--color-text-tertiary);
  font-weight: 500;
}

/* 任务卡片容器 */
.task-card-container {
  position: relative;
}

.deleted-task {
  opacity: 0.7;
  pointer-events: none; /* 禁止点击 */
  cursor: default;
}

/* 操作按钮 */
.task-actions {
  display: flex;
  gap: 0.8rem;
  padding: 0 0.8rem;
}

.action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  padding: 0.6rem 1rem;
  font-size: 1.2rem;
  font-weight: 500;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-icon {
  font-size: 1.4rem;
}

.btn-text {
  font-size: 1.2rem;
}

.restore-btn {
  color: #fff;
  background-color: #10b981;
}

.restore-btn:hover {
  background-color: #059669;
}

.delete-btn {
  color: #fff;
  background-color: #ef4444;
}

.delete-btn:hover {
  background-color: #dc2626;
}

.action-btn:active {
  transform: scale(0.98);
}
</style>

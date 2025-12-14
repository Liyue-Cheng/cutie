<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTrashStore } from '@/stores/trash'
import KanbanTaskCard from '@/components/assembles/tasks/kanban/KanbanTaskCard.vue'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import { dialog } from '@/composables/useDialog'

const { t } = useI18n()
const trashStore = useTrashStore()

onMounted(async () => {
  logger.info(LogTags.VIEW_TRASH, 'Loading trash...')
  try {
    await trashStore.fetchTrash()
    logger.info(LogTags.VIEW_TRASH, 'Loaded deleted tasks', { count: trashStore.trashedTaskCount })
  } catch (error) {
    logger.error(
      LogTags.VIEW_TRASH,
      'Failed to fetch trash',
      error instanceof Error ? error : new Error(String(error))
    )
  }
})

// 任务总数
const taskCount = computed(() => trashStore.trashedTaskCount)

// 格式化删除时间
function formatDeletedAt(deletedAt: string | null): string {
  if (!deletedAt) return t('time.unknown')

  const date = new Date(deletedAt)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return t('time.justNow')
  if (diffMins < 60) return t('time.minutesAgo', { n: diffMins })
  if (diffHours < 24) return t('time.hoursAgo', { n: diffHours })
  if (diffDays < 7) return t('time.daysAgo', { n: diffDays })

  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
}

// 恢复任务
async function handleRestore(task: TaskCard) {
  const confirmed = await dialog.confirm(t('confirm.restoreTask', { title: task.title }))
  if (!confirmed) return

  try {
    await trashStore.restoreTask(task.id)
    logger.info(LogTags.VIEW_TRASH, 'Task restored', { taskId: task.id })
  } catch (error) {
    logger.error(
      LogTags.VIEW_TRASH,
      'Failed to restore task',
      error instanceof Error ? error : new Error(String(error)),
      { taskId: task.id }
    )
    await dialog.alert(t('message.error.restoreFailed'))
  }
}

// 彻底删除任务
async function handlePermanentlyDelete(task: TaskCard) {
  const confirmed = await dialog.confirm(t('confirm.permanentDeleteTask', { title: task.title }), {
    danger: true,
  })
  if (!confirmed) return

  try {
    await trashStore.permanentlyDeleteTask(task.id)
    logger.info(LogTags.VIEW_TRASH, 'Task permanently deleted', { taskId: task.id })
  } catch (error) {
    logger.error(
      LogTags.VIEW_TRASH,
      'Failed to permanently delete task',
      error instanceof Error ? error : new Error(String(error)),
      { taskId: task.id }
    )
    await dialog.alert(t('message.error.deleteFailed'))
  }
}

// 清空回收站
async function handleEmptyTrash() {
  const confirmed = await dialog.confirm(t('confirm.emptyTrash'), { danger: true })
  if (!confirmed) return

  try {
    const deletedCount = await trashStore.emptyTrash({ olderThanDays: 0 })
    logger.info(LogTags.VIEW_TRASH, 'Trash emptied', { deletedCount })
    await dialog.alert(t('message.success.trashEmptied', { count: deletedCount }))
  } catch (error) {
    logger.error(
      LogTags.VIEW_TRASH,
      'Failed to empty trash',
      error instanceof Error ? error : new Error(String(error))
    )
    await dialog.alert(t('message.error.emptyTrashFailed'))
  }
}

// 禁止打开编辑器
function handleOpenEditor() {
  // 回收站中的任务不允许编辑
  logger.debug(LogTags.VIEW_TRASH, 'Cannot edit deleted tasks')
}
</script>

<template>
  <div class="trash-column">
    <div class="column-header">
      <div class="header-title">
        <h3>{{ $t('trash.title') }}</h3>
        <span class="task-count">{{ taskCount }}</span>
      </div>
      <button v-if="taskCount > 0" class="empty-trash-btn" @click="handleEmptyTrash">
        {{ $t('trash.action.empty') }}
      </button>
    </div>

    <div class="column-content">
      <div v-if="trashStore.trashedTaskCount === 0" class="empty-state">
        <p>{{ $t('trash.empty.message') }}</p>
      </div>

      <div v-else class="tasks-list">
        <div v-for="task in trashStore.allTrashedTasks" :key="task.id" class="task-wrapper">
          <!-- 删除时间指示器 -->
          <div class="deleted-time-indicator">
            <span class="deleted-time-text">{{ $t('trash.label.deletedAt', { time: formatDeletedAt(task.deleted_at) }) }}</span>
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
              <span class="btn-text">{{ $t('trash.action.restore') }}</span>
            </button>
            <button class="action-btn delete-btn" @click="handlePermanentlyDelete(task)">
              <span class="btn-icon">🗑</span>
              <span class="btn-text">{{ $t('trash.action.permanentDelete') }}</span>
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
  color: var(--color-button-danger-text, #f0f);
  background-color: var(--color-button-danger-bg, #f0f);
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.empty-trash-btn:hover {
  background-color: var(--color-button-danger-hover, #f0f);
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
  color: var(--color-button-primary-text, #f0f);
  background-color: var(--color-button-primary-bg, #f0f);
}

.restore-btn:hover {
  background-color: var(--color-button-primary-hover, #f0f);
}

.delete-btn {
  color: var(--color-button-danger-text, #f0f);
  background-color: var(--color-button-danger-bg, #f0f);
}

.delete-btn:hover {
  background-color: var(--color-button-danger-hover, #f0f);
}

.action-btn:active {
  transform: scale(0.98);
}
</style>

<template>
  <div class="context-menu">
    <button class="menu-button" @click="handleAction('edit')">编辑任务</button>

    <!-- 循环任务相关操作 -->
    <template v-if="isRecurringTask">
      <div class="divider"></div>
      <div class="menu-section-title">Task recurrence:</div>
      <button class="menu-button" @click="handleAction('stop-repeating')">
        <CuteIcon name="Square" :size="14" />
        Stop repeating
      </button>
      <button class="menu-button" @click="handleAction('change-frequency')">
        <CuteIcon name="RefreshCw" :size="14" />
        Change repeat frequency
      </button>
      <button class="menu-button" @click="handleAction('update-all-instances')">
        <CuteIcon name="Copy" :size="14" />
        Update all incomplete instances to match this task
      </button>
      <button class="menu-button delete" @click="handleAction('delete-all-instances')">
        <CuteIcon name="Trash2" :size="14" />
        Delete all incomplete instances and stop repeating
      </button>
    </template>

    <div class="divider"></div>
    <button class="menu-button" @click="handleAction('return-to-staging')">
      <CuteIcon name="RotateCcw" :size="14" />
      返回暂存区
    </button>
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
import { defineProps, defineEmits, computed } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { pipeline } from '@/cpu'
import { useRecurrenceOperations } from '@/composables/useRecurrenceOperations'
import { logger, LogTags } from '@/infra/logging/logger'
import CuteIcon from '@/components/parts/CuteIcon.vue'

const props = defineProps<{
  task: TaskCard
}>()

const emit = defineEmits(['close'])

// 循环规则相关操作暂时保留 composable（等后续统一迁移）
const recurrenceOps = useRecurrenceOperations()

// 检查是否为循环任务
const isRecurringTask = computed(() => {
  return !!(props.task.recurrence_id && props.task.recurrence_original_date)
})

type ActionType =
  | 'edit'
  | 'delete'
  | 'archive'
  | 'unarchive'
  | 'return-to-staging'
  | 'stop-repeating'
  | 'change-frequency'
  | 'update-all-instances'
  | 'delete-all-instances'

const handleAction = async (action: ActionType) => {
  // ✅ 使用命令总线处理任务操作
  if (action === 'delete') {
    try {
      await pipeline.dispatch('task.delete', { id: props.task.id })
      logger.info(LogTags.COMPONENT_KANBAN, 'Task deleted', { taskTitle: props.task.title })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to delete task',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'archive') {
    try {
      await pipeline.dispatch('task.archive', { id: props.task.id })
      logger.info(LogTags.COMPONENT_KANBAN, 'Task archived', { taskTitle: props.task.title })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to archive task',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'unarchive') {
    try {
      await pipeline.dispatch('task.unarchive', { id: props.task.id })
      logger.info(LogTags.COMPONENT_KANBAN, 'Task unarchived', { taskTitle: props.task.title })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to unarchive task',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'return-to-staging') {
    try {
      await pipeline.dispatch('task.return_to_staging', { id: props.task.id })
      logger.info(LogTags.COMPONENT_KANBAN, 'Task returned to staging', {
        taskTitle: props.task.title,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to return task to staging',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'edit') {
    logger.debug(LogTags.COMPONENT_KANBAN, 'Task action', { action, taskId: props.task.id })
    // TODO: 实现编辑功能
  } else if (action === 'stop-repeating') {
    if (!props.task.recurrence_id || !props.task.recurrence_original_date) return

    try {
      await recurrenceOps.stopRepeating(
        props.task.recurrence_id,
        props.task.recurrence_original_date
      )
      logger.info(LogTags.COMPONENT_KANBAN, 'Stopped repeating task', {
        taskTitle: props.task.title,
        recurrenceId: props.task.recurrence_id,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to stop repeating task',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'change-frequency') {
    if (!props.task.recurrence_id) return

    try {
      await recurrenceOps.openEditDialog(props.task.recurrence_id)
      logger.info(LogTags.COMPONENT_KANBAN, 'Opening recurrence edit dialog', {
        recurrenceId: props.task.recurrence_id,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to open recurrence edit dialog',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'update-all-instances') {
    if (!props.task.recurrence_id) return

    try {
      logger.info(LogTags.COMPONENT_KANBAN, 'Updating all instances to match task', {
        props,
      })
      await recurrenceOps.updateAllInstances(props.task.recurrence_id, props.task)
      logger.info(LogTags.COMPONENT_KANBAN, 'Updated all instances to match task', {
        taskTitle: props.task.title,
        recurrenceId: props.task.recurrence_id,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to update all instances',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'delete-all-instances') {
    if (!props.task.recurrence_id) return

    const confirmed = confirm(
      `确定删除所有未完成的循环任务实例并停止重复吗？\n` +
        `这将删除所有未来的"${props.task.title}"任务。\n` +
        `此操作不可撤销。`
    )

    if (confirmed) {
      try {
        await recurrenceOps.deleteAllInstancesAndStop(props.task.recurrence_id)
        logger.info(LogTags.COMPONENT_KANBAN, 'Deleted all instances and stopped repeating', {
          taskTitle: props.task.title,
          recurrenceId: props.task.recurrence_id,
        })
      } catch (error) {
        logger.error(
          LogTags.COMPONENT_KANBAN,
          'Failed to delete all instances and stop repeating',
          error instanceof Error ? error : new Error(String(error))
        )
      }
    }
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

.divider {
  width: 100%;
  height: 1px;
  background-color: #e0e0e0;
  margin: 4px 0;
}

.menu-section-title {
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 600;
  color: #666;
  text-transform: uppercase;
  letter-spacing: 0.5px;
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
  display: flex;
  align-items: center;
  gap: 8px;
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
</style>

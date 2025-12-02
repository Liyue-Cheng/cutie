<template>
  <ContextMenu>
    <MenuItem @click="handleAction('edit')">{{ $t('task.action.edit') }}</MenuItem>

    <!-- 循环任务相关操作 -->
    <template v-if="isRecurringTask">
      <MenuSection divider :title="$t('recurrence.menuSection')">
        <MenuItem icon="Square" @click="handleAction('stop-repeating')">
          {{ $t('recurrence.action.stop') }}
        </MenuItem>
        <MenuItem icon="RefreshCw" @click="handleAction('change-frequency')">
          {{ $t('recurrence.action.changeFrequency') }}
        </MenuItem>
        <MenuItem icon="Copy" @click="handleAction('update-all-instances')">
          {{ $t('recurrence.action.updateAll') }}
        </MenuItem>
        <MenuItem icon="Trash2" variant="danger" @click="handleAction('delete-all-instances')">
          {{ $t('recurrence.action.deleteAll') }}
        </MenuItem>
      </MenuSection>
    </template>

    <MenuItem divider icon="RotateCcw" @click="handleAction('return-to-staging')">
      {{ $t('task.action.returnToStaging') }}
    </MenuItem>

    <!-- 取消今日排期（只在日期视图显示） -->
    <MenuItem
      v-if="showCancelSchedule"
      divider
      icon="CalendarX"
      @click="handleAction('cancel-today-schedule')"
    >
      {{ $t('task.action.cancelTodaySchedule') }}
    </MenuItem>

    <MenuItem v-if="!task.is_archived" divider @click="handleAction('archive')">
      {{ $t('task.action.archive') }}
    </MenuItem>
    <MenuItem v-else divider @click="handleAction('unarchive')">{{ $t('task.action.unarchive') }}</MenuItem>

    <MenuItem divider variant="danger" @click="handleAction('delete')">{{ $t('task.action.delete') }}</MenuItem>
  </ContextMenu>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { TaskCard } from '@/types/dtos'
import { pipeline } from '@/cpu'
import { useRecurrenceOperations } from '@/composables/useRecurrenceOperations'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/infra/logging/logger'
import ContextMenu from '@/components/assembles/ContextMenu/shared/CuteContextMenu.vue'
import MenuItem from '@/components/assembles/ContextMenu/shared/CuteMenuItem.vue'
import MenuSection from '@/components/assembles/ContextMenu/shared/CuteMenuSection.vue'

const props = defineProps<{
  task: TaskCard
  viewKey?: string
}>()

const emit = defineEmits(['close'])

const { t } = useI18n()
// UI Store
const uiStore = useUIStore()

// 循环规则相关操作暂时保留 composable（等后续统一迁移）
const recurrenceOps = useRecurrenceOperations()

// 检查是否为循环任务
const isRecurringTask = computed(() => {
  return !!(props.task.recurrence_id && props.task.recurrence_original_date)
})

// 检查是否显示"取消今日排期"选项
// 只在日期视图（viewKey 为 daily::YYYY-MM-DD 格式）中显示
const showCancelSchedule = computed(() => {
  if (!props.viewKey) return false
  return props.viewKey.startsWith('daily::')
})

// 获取当前日期
const currentDate = computed(() => {
  if (props.viewKey && props.viewKey.startsWith('daily::')) {
    return props.viewKey.split('::')[1]
  }
  return ''
})

type ActionType =
  | 'edit'
  | 'delete'
  | 'archive'
  | 'unarchive'
  | 'return-to-staging'
  | 'cancel-today-schedule'
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
  } else if (action === 'cancel-today-schedule') {
    try {
      const dateToCancel = currentDate.value
      if (!dateToCancel) {
        logger.warn(LogTags.COMPONENT_KANBAN, 'No date to cancel schedule for')
        return
      }

      await pipeline.dispatch('schedule.delete', {
        task_id: props.task.id,
        scheduled_day: dateToCancel,
      })
      logger.info(LogTags.COMPONENT_KANBAN, 'Cancelled today schedule', {
        taskTitle: props.task.title,
        date: dateToCancel,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to cancel today schedule',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  } else if (action === 'edit') {
    logger.debug(LogTags.COMPONENT_KANBAN, 'Opening task editor', { taskId: props.task.id })
    // ✅ 打开任务编辑框
    uiStore.openEditor(props.task.id, props.viewKey)
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
      t('confirm.deleteAllRecurrenceInstances', { title: props.task.title })
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

// 修改 confirm 函数调用
function getConfirmMessage() {
  if (props.task.recurrence_id) {
    return t('confirm.deleteAllRecurrenceInstances', { title: props.task.title })
  }
  return ''
}
</script>

<template>
  <TimeBlockCreateDialog
    :show="uiStore.isTimeBlockCreateDialogOpen"
    :position="timeBlockDialogPosition"
    @confirm="handleTimeBlockCreate"
    @cancel="handleTimeBlockDialogCancel"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import TimeBlockCreateDialog from '@/components/organisms/TimeBlockCreateDialog.vue'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { dialog } from '@/composables/useDialog'

const uiStore = useUIStore()

// 创建对话框位置（根据 UI Store 中的锚点信息计算）
const timeBlockDialogPosition = computed(() => {
  const context = uiStore.timeBlockCreateContext as {
    anchorTop?: number
    anchorLeft?: number
  } | null

  if (!context || context.anchorTop == null || context.anchorLeft == null) {
    return undefined
  }

  return {
    top: context.anchorTop,
    left: context.anchorLeft,
  }
})

function handleTimeBlockDialogCancel() {
  uiStore.closeTimeBlockCreateDialog()
}

async function handleTimeBlockCreate(data: { type: 'task' | 'event'; title: string; description?: string }) {
  const context = uiStore.timeBlockCreateContext
  if (!context) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'No context available for time block creation',
      new Error('Context is null')
    )
    return
  }

  try {
    if (data.type === 'task') {
      const taskCard = await pipeline.dispatch('task.create', {
        title: data.title,
        glance_note: data.description || null,
        estimated_duration: 60,
      })

      await pipeline.dispatch('time_block.create_from_task', {
        task_id: taskCard.id,
        start_time: context.startISO,
        end_time: context.endISO,
        start_time_local: context.startTimeLocal,
        end_time_local: context.endTimeLocal,
        time_type: 'FLOATING',
        creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        is_all_day: context.isAllDay,
      })

      logger.info(LogTags.COMPONENT_CALENDAR, 'Created task with time block from calendar', {
        title: data.title,
        taskId: taskCard.id,
        startISO: context.startISO,
        endISO: context.endISO,
      })
    } else {
      await pipeline.dispatch('time_block.create', {
        title: data.title,
        description: data.description || null,
        start_time: context.startISO,
        end_time: context.endISO,
        start_time_local: context.startTimeLocal,
        end_time_local: context.endTimeLocal,
        time_type: 'FLOATING',
        creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        is_all_day: context.isAllDay,
      })

      logger.info(LogTags.COMPONENT_CALENDAR, 'Created time block from calendar', {
        title: data.title,
        startISO: context.startISO,
        endISO: context.endISO,
        isAllDay: context.isAllDay,
      })
    }

    uiStore.closeTimeBlockCreateDialog()
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'Failed to create from calendar',
      error instanceof Error ? error : new Error(String(error)),
      { type: data.type, title: data.title }
    )

    let errorMessage = '创建失败，请重试'
    if (error instanceof Error) {
      errorMessage = error.message
    } else if (typeof error === 'string') {
      errorMessage = error
    }
    await dialog.alert(`创建失败: ${errorMessage}`)
  }
}
</script>



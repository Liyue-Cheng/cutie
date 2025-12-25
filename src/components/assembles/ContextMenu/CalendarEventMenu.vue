<template>
  <ContextMenu>
    <!-- 循环时间块相关操作 -->
    <template v-if="isRecurringTimeBlock">
      <MenuSection :title="$t('recurrence.timeBlockMenuSection')">
        <!-- 停止和继续是互斥的：有 end_date 时显示继续，否则显示停止 -->
        <MenuItem v-if="!isRecurrenceStopped" icon="Square" @click="handleStopRepeating">
          {{ $t('recurrence.action.stop') }}
        </MenuItem>
        <MenuItem v-else icon="Play" @click="handleResumeRepeating">
          {{ $t('recurrence.action.continue') }}
        </MenuItem>
        <MenuItem icon="RefreshCw" @click="handleChangeFrequency">
          {{ $t('recurrence.action.changeFrequency') }}
        </MenuItem>
      </MenuSection>
    </template>

    <!-- 危险操作 -->
    <MenuItem :divider="isRecurringTimeBlock" icon="Trash2" variant="danger" @click="handleDelete">
      {{ $t('timeBlock.action.delete') }}
    </MenuItem>
  </ContextMenu>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import ContextMenu from '@/components/assembles/ContextMenu/shared/CuteContextMenu.vue'
import MenuItem from '@/components/assembles/ContextMenu/shared/CuteMenuItem.vue'
import MenuSection from '@/components/assembles/ContextMenu/shared/CuteMenuSection.vue'
import type { EventApi } from '@fullcalendar/core'
import { pipeline } from '@/cpu'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useTimeBlockRecurrenceOperations } from '@/composables/useTimeBlockRecurrenceOperations'
import { getTimeBlockRecurrenceById } from '@/cpu/isa/timeblock-recurrence-isa'

const props = defineProps<{
  event: EventApi | { id: string }
}>()

const emit = defineEmits(['close'])

const { t } = useI18n()
const timeBlockStore = useTimeBlockStore()
const recurrenceOps = useTimeBlockRecurrenceOperations()

// 获取时间块ID
const timeBlockId = computed(() => {
  return typeof props.event.id === 'string' ? props.event.id : String(props.event.id)
})

// 获取完整的时间块数据
const timeBlock = computed(() => {
  return timeBlockStore.getTimeBlockById(timeBlockId.value)
})

// 检查是否为循环时间块
const isRecurringTimeBlock = computed(() => {
  return !!(timeBlock.value?.recurrence_id && timeBlock.value?.recurrence_original_date)
})

// 检查循环是否已停止（有 end_date 表示已停止）
const isRecurrenceStopped = computed(() => {
  if (!timeBlock.value?.recurrence_id) return false
  const recurrence = getTimeBlockRecurrenceById(timeBlock.value.recurrence_id)
  return !!recurrence?.end_date
})

// 删除单个时间块
const handleDelete = async () => {
  await pipeline.dispatch('time_block.delete', { id: timeBlockId.value })
  emit('close')
}

// 停止循环（在当前时间块的日期停止）
const handleStopRepeating = async () => {
  if (!timeBlock.value?.recurrence_id || !timeBlock.value?.recurrence_original_date) return

  try {
    // 使用当前时间块的原始日期作为停止日期
    await recurrenceOps.stopRepeating(
      timeBlock.value.recurrence_id,
      timeBlock.value.recurrence_original_date
    )
  } catch (error) {
    console.error('Failed to stop repeating:', error)
  }
  emit('close')
}

// 继续循环
const handleResumeRepeating = async () => {
  if (!timeBlock.value?.recurrence_id) return

  try {
    await recurrenceOps.resumeRecurrence(timeBlock.value.recurrence_id)
  } catch (error) {
    console.error('Failed to resume repeating:', error)
  }
  emit('close')
}

// 修改重复频率
const handleChangeFrequency = () => {
  if (!timeBlock.value?.recurrence_id) return
  recurrenceOps.openEditDialog(timeBlock.value.recurrence_id)
  emit('close')
}
</script>

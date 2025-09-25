<template>
  <n-card class="context-menu" content-style="padding: 5px;">
    <n-button text @click="handleDelete">
      <template #icon>
        <CuteIcon name="Trash2" />
      </template>
      删除事件
    </n-button>
  </n-card>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import { NCard, NButton } from 'naive-ui'
import CuteIcon from '@/components/ui/CuteIcon.vue'
import type { EventApi } from '@fullcalendar/core'
import { useActivityStore } from '@/stores/activity'

const props = defineProps<{
  event: EventApi
}>()

const emit = defineEmits(['close'])
const activityStore = useActivityStore()

const handleDelete = async () => {
  await activityStore.deleteActivity(props.event.id)
  emit('close') // Close the context menu
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

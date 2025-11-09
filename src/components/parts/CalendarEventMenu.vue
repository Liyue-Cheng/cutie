<template>
  <CuteContextMenu>
    <CuteMenuItem icon="Trash2" variant="danger" @click="handleDelete">
      删除事件
    </CuteMenuItem>
  </CuteContextMenu>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import CuteContextMenu from '@/components/parts/CuteContextMenu.vue'
import CuteMenuItem from '@/components/parts/CuteMenuItem.vue'
import type { EventApi } from '@fullcalendar/core'
import { pipeline } from '@/cpu'

const props = defineProps<{
  event: EventApi
}>()

const emit = defineEmits(['close'])

const handleDelete = async () => {
  await pipeline.dispatch('time_block.delete', { id: props.event.id })
  emit('close')
}
</script>

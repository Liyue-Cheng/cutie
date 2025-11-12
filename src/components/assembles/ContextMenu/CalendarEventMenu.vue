<template>
  <ContextMenu>
    <MenuItem icon="Trash2" variant="danger" @click="handleDelete">
      删除事件
    </MenuItem>
  </ContextMenu>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import ContextMenu from '@/components/assembles/ContextMenu/shared/CuteContextMenu.vue'
import MenuItem from '@/components/assembles/ContextMenu/shared/CuteMenuItem.vue'
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

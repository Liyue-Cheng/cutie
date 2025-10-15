<template>
  <div class="context-menu">
    <button class="menu-button delete" @click="handleDelete">
      <CuteIcon name="Trash2" />
      <span>删除事件</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import type { EventApi } from '@fullcalendar/core'
import { pipeline } from '@/cpu'

const props = defineProps<{
  event: EventApi
}>()

const emit = defineEmits(['close'])

const handleDelete = async () => {
  await pipeline.dispatch('timeblock.delete', { id: props.event.id })
  emit('close') // Close the context menu
}
</script>

<style scoped>
.context-menu {
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
  border-radius: 4px;
  background-color: #fff;
  display: inline-block;
  padding: 5px;
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

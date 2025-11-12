<script setup lang="ts">
import CuteIcon from '@/components/parts/CuteIcon.vue'

interface Props {
  title: string
  areaColor: string
}

defineProps<Props>()
</script>

<template>
  <div class="calendar-timeblock-event-content">
    <CuteIcon name="Circle" :size="12" :color="areaColor" class="timeblock-area-icon" />
    <span class="timeblock-title">{{ title }}</span>
  </div>
</template>

<style scoped>
.calendar-timeblock-event-content {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.1rem 0.2rem 0.1rem 0.1rem;
  width: 100%;
  box-sizing: border-box;
  pointer-events: auto;
}

.timeblock-area-icon {
  flex: 0 0 auto;
}

.timeblock-title {
  flex: 1 1 auto;
  font-size: 1.2rem;
  line-height: 1.4rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text-primary, #575279);
}
</style>

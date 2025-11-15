<script setup lang="ts">
import { computed } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'

type Props = {
  title: string
  isOverdue?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '任务',
  isOverdue: false,
})

const iconColor = computed(() => (props.isOverdue ? 'var(--color-deadline-overdue)' : 'var(--color-text-tertiary)'))
const titleClass = computed(() => ({ overdue: props.isOverdue }))
</script>

<template>
  <div class="calendar-due-date-event-content">
    <CuteIcon name="Flag" :size="14" :color="iconColor" class="due-date-icon" />
    <span class="due-date-title" :class="titleClass">{{ title }}</span>
  </div>
</template>

<style scoped>
.calendar-due-date-event-content {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.1rem 0.2rem 0.1rem 0.1rem;
  width: 100%;
  box-sizing: border-box;
  pointer-events: auto;
}

.due-date-icon {
  flex: 0 0 auto;
}

.due-date-title {
  flex: 1 1 auto;
  font-size: 1.2rem;
  line-height: 1.4rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text-primary);
}

.due-date-title.overdue {
  color: var(--color-danger);
  font-weight: 600;
}
</style>

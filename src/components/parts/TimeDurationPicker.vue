<script setup lang="ts">
/**
 * TimeDurationPicker - 时间预期时长选择器
 *
 * 用于选择任务的预期时长
 * 提供预设的时间选项和清除功能
 */
import { computed } from 'vue'

interface Props {
  /** 当前已选择的时长（分钟），null表示未设置 */
  modelValue: number | null
  /** 是否显示今日标签 */
  showTodayLabel?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showTodayLabel: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: number | null]
  close: []
}>()

// 预设时间选项（分钟）
const timeOptions = [
  { label: '5 min', value: 5 },
  { label: '10 min', value: 10 },
  { label: '15 min', value: 15 },
  { label: '20 min', value: 20 },
  { label: '25 min', value: 25 },
  { label: '30 min', value: 30 },
  { label: '45 min', value: 45 },
  { label: '1 hr', value: 60 },
  { label: '1.5 hr', value: 90 },
  { label: '2 hr', value: 120 },
  { label: '3 hr', value: 180 },
  { label: '4 hr', value: 240 },
]

// 格式化显示当前选择的时长
const formattedDuration = computed(() => {
  if (props.modelValue === null || props.modelValue === 0) {
    return 'tiny'
  }

  const minutes = props.modelValue
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60

  if (hours > 0 && mins > 0) {
    return `${hours}:${mins.toString().padStart(2, '0')}`
  } else if (hours > 0) {
    return `${hours}:00`
  } else {
    return `${mins} min`
  }
})

function selectDuration(value: number) {
  emit('update:modelValue', value)
  emit('close')
}

function clearDuration() {
  emit('update:modelValue', null)
  emit('close')
}
</script>

<template>
  <div class="time-duration-picker" @click.stop>
    <!-- 标题 -->
    <div class="picker-header">
      <span v-if="showTodayLabel" class="label">Planned (today):</span>
      <span class="current-value">{{ formattedDuration }}</span>
    </div>

    <!-- 时间选项列表 -->
    <div class="time-options">
      <button
        v-for="option in timeOptions"
        :key="option.value"
        class="time-option"
        :class="{ active: modelValue === option.value }"
        @click="selectDuration(option.value)"
      >
        {{ option.label }}
      </button>
    </div>

    <!-- 清除按钮 -->
    <button class="clear-button" @click="clearDuration">Clear planned</button>
  </div>
</template>

<style scoped>
.time-duration-picker {
  display: flex;
  flex-direction: column;
  width: 16rem;
  max-height: 40rem;
  background: white;
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  box-shadow: 0 4px 16px rgb(0 0 0 / 15%);
  overflow: hidden;
}

.picker-header {
  padding: 1.2rem 1.4rem;
  border-bottom: 1px solid var(--color-border-default);
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.label {
  font-size: 1.1rem;
  color: var(--color-text-secondary);
}

.current-value {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.time-options {
  flex: 1;
  overflow-y: auto;
  padding: 0.4rem 0;
}

.time-option {
  width: 100%;
  padding: 1rem 1.4rem;
  border: none;
  background: transparent;
  text-align: left;
  font-size: 1.3rem;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: background-color 0.15s;
}

.time-option:hover {
  background-color: var(--color-bg-hover, #f5f5f5);
}

.time-option.active {
  background-color: var(--color-primary-light, #e3f2fd);
  color: var(--color-primary, #1976d2);
  font-weight: 500;
}

.clear-button {
  width: 100%;
  padding: 1.2rem 1.4rem;
  border: none;
  border-top: 1px solid var(--color-border-default);
  background: white;
  text-align: left;
  font-size: 1.3rem;
  color: var(--color-primary, #1976d2);
  cursor: pointer;
  transition: background-color 0.15s;
}

.clear-button:hover {
  background-color: var(--color-bg-hover, #f5f5f5);
}
</style>

<script setup lang="ts">
// CuteCheckbox - 单模式复选框（仅完成状态）
// 基于 CuteDualModeCheckbox 的简化封装
// 样式与 CuteDualModeCheckbox 完全一致（圆角方形）

import { computed } from 'vue'
import CuteDualModeCheckbox from './CuteDualModeCheckbox.vue'

interface Props {
  checked?: boolean
  size?: 'small' | 'large' | string
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  checked: false,
  size: 'small',
  disabled: false,
})

const emit = defineEmits<{
  'update:checked': [value: boolean]
}>()

// 将 checked (boolean) 映射到 state (CheckboxState)
const state = computed(() => (props.checked ? 'completed' : null))

// 处理状态变化：只处理 completed 和 null
const handleStateChange = (newState: 'completed' | 'present' | null) => {
  emit('update:checked', newState === 'completed')
}
</script>

<template>
  <CuteDualModeCheckbox
    :state="state"
    :size="size"
    :class="{ 'checkbox-disabled': disabled }"
    disable-long-press
    @update:state="handleStateChange"
  />
</template>

<style scoped>
.checkbox-disabled {
  opacity: 0.5;
  pointer-events: none;
  cursor: not-allowed;
}
</style>

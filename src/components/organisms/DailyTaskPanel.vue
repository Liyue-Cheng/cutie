<template>
  <div class="daily-task-panel">
    <TaskList
      :key="viewKey"
      :title="taskListTitle"
      :view-key="viewKey"
      :show-add-input="true"
      :default-collapsed="false"
      :collapsible="false"
      fill-remaining-space
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'
import { getTodayDateString } from '@/infra/utils/dateUtils'

// Props
interface Props {
  modelValue?: string // 当前日期 YYYY-MM-DD
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: () => getTodayDateString(),
})

// 计算 view-key，格式为 daily::YYYY-MM-DD
const viewKey = computed(() => `daily::${props.modelValue}`)

// 是否是今天
const isToday = computed(() => props.modelValue === getTodayDateString())

// 格式化日期显示（作为 TaskList 标题）
const taskListTitle = computed(() => {
  const date = new Date(props.modelValue)
  const month = date.getMonth() + 1
  const day = date.getDate()
  const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  const weekday = weekdays[date.getDay()]

  if (isToday.value) {
    return `${month}月${day}日 ${weekday} · 今天`
  }
  return `${month}月${day}日 ${weekday}`
})
</script>

<style scoped>
.daily-task-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>

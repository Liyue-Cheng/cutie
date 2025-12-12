<template>
  <div class="daily-task-panel">
    <TwoRowLayout>
      <template #top>
        <div class="daily-header">
          <div class="header-left">
            <span class="date-text">{{ $t('toolbar.dailyTasks') }}</span>
          </div>
        </div>
      </template>
      <template #bottom>
        <div class="task-list-wrapper">
          <TaskList
            :key="viewKey"
            :title="taskListTitle"
            :view-key="viewKey"
            :show-add-input="true"
            :default-collapsed="false"
            :collapsible="false"
            :hide-header="false"
            fill-remaining-space
          />
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
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

/* 标题栏样式 - 与 TaskList 标题对齐 */
.daily-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;

  /* TwoRowLayout top-row 有 padding: 0 1rem，TaskList header 有 padding: 1rem 1.6rem */

  /* 需要补充 0.6rem 使总 padding 达到 1.6rem */
  padding-left: 0.6rem;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.date-text {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
  white-space: nowrap;
}

.task-list-wrapper {
  height: 100%;
  padding: 1rem;
  overflow-y: auto;
}
</style>

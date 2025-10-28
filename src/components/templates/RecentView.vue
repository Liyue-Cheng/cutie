<template>
  <div class="recent-view">
    <TwoRowLayout>
      <template #top>
        <div class="recent-controls">
          <!-- 今天按钮 -->
          <button class="control-btn today-btn" @click="goToToday" title="回到今天">
            <CuteIcon name="Calendar" :size="16" />
            <span>今天</span>
          </button>

          <!-- 日期选择器 -->
          <div class="date-picker-wrapper">
            <input type="date" v-model="selectedDate" class="date-input" @change="onDateChange" />
          </div>

          <!-- 天数选择器 -->
          <div class="day-count-selector">
            <button
              v-for="count in dayCountOptions"
              :key="count"
              class="day-count-btn"
              :class="{ active: dayCount === count }"
              @click="setDayCount(count)"
            >
              {{ count }}天
            </button>
          </div>
        </div>
      </template>
      <template #bottom>
        <div class="task-list">
          <!-- 动态生成的日期任务栏 -->
          <TaskBar
            v-for="dateInfo in dateList"
            :key="dateInfo.viewKey"
            :title="dateInfo.label"
            :view-key="dateInfo.viewKey"
          />
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import TaskBar from '@/components/parts/TaskBar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { useTaskStore } from '@/stores/task'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

// Props
interface Props {
  modelValue?: number // 天数（支持 v-model）
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: 3,
})

// Emits
const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const taskStore = useTaskStore()

// ==================== "最近"视图状态 ====================
// 确保 selectedDate 始终是有效的日期字符串
const getValidDateString = (): string => {
  const dateStr = getTodayDateString()
  if (dateStr) return dateStr
  // 兜底：使用当前日期（ISO 8601 格式）
  const fallback = new Date().toISOString().split('T')[0]
  return fallback as string
}

const selectedDate = ref<string>(getValidDateString()) // 选择的起始日期
const dayCount = ref(props.modelValue) // 显示的天数
const dayCountOptions = [1, 3, 5, 7] // 可选的天数选项

// 监听 props 变化
watch(
  () => props.modelValue,
  (newValue) => {
    dayCount.value = newValue
  }
)

// 生成日期列表
interface DateInfo {
  date: string
  viewKey: string
  label: string
}

const dateList = computed<DateInfo[]>(() => {
  const list: DateInfo[] = []
  const today = getValidDateString()
  const startDate = selectedDate.value

  for (let i = 0; i < dayCount.value; i++) {
    const date = new Date(startDate)
    date.setDate(date.getDate() + i)
    const isoString = date.toISOString()
    const dateString = isoString.split('T')[0] as string // ISO 8601 格式总是 YYYY-MM-DDTHH:mm:ss.sssZ

    // 生成友好的日期标签
    const label = formatDateLabel(dateString, today)

    list.push({
      date: dateString,
      viewKey: `daily::${dateString}`,
      label,
    })
  }

  return list
})

// 格式化日期标签
function formatDateLabel(dateString: string, today: string): string {
  const date = new Date(dateString)
  const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  const weekday = weekdays[date.getDay()]

  // 检查是否是今天
  if (dateString === today) {
    return `今天 ${weekday}`
  }

  // 检查是否是昨天
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)
  const yesterdayString = yesterday.toISOString().split('T')[0]
  if (dateString === yesterdayString) {
    return `昨天 ${weekday}`
  }

  // 检查是否是明天
  const tomorrow = new Date(today)
  tomorrow.setDate(tomorrow.getDate() + 1)
  const tomorrowString = tomorrow.toISOString().split('T')[0]
  if (dateString === tomorrowString) {
    return `明天 ${weekday}`
  }

  // 否则显示月-日 周X
  const month = date.getMonth() + 1
  const day = date.getDate()
  return `${month}月${day}日 ${weekday}`
}

// 回到今天
function goToToday() {
  selectedDate.value = getValidDateString()
  logger.info(LogTags.VIEW_HOME, 'Navigate to today', { date: selectedDate.value })
}

// 日期变化
function onDateChange() {
  logger.info(LogTags.VIEW_HOME, 'Date changed', {
    date: selectedDate.value,
    dayCount: dayCount.value,
  })

  // 预加载选择日期范围的任务
  loadDateRangeTasks()
}

// 设置天数
function setDayCount(count: number) {
  dayCount.value = count
  emit('update:modelValue', count) // 通知父组件
  logger.info(LogTags.VIEW_HOME, 'Day count changed', { dayCount: count })

  // 预加载日期范围的任务
  loadDateRangeTasks()
}

// 预加载日期范围的任务
async function loadDateRangeTasks() {
  for (const dateInfo of dateList.value) {
    await taskStore.fetchDailyTasks_DMA(dateInfo.date)
  }
}

// 初始化
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing RecentView component...')

  // 加载日期范围的任务
  await loadDateRangeTasks()
})
</script>

<style scoped>
.recent-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* ==================== 最近视图控制栏 ==================== */
.recent-controls {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 1.2rem;
  padding: 1.2rem 1.6rem;
  background-color: transparent;
}

/* 今天按钮 */
.control-btn {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.8rem 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.control-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.control-btn:active {
  transform: scale(0.98);
}

/* 日期选择器 */
.date-picker-wrapper {
  flex: 1;
  min-width: 16rem;
}

.date-input {
  width: 100%;
  padding: 0.8rem 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.date-input:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.date-input:focus {
  outline: none;
  border-color: var(--color-primary, #4a90e2);
  box-shadow: 0 0 0 2px rgb(74 144 226 / 10%);
}

/* 天数选择器 */
.day-count-selector {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  padding: 0.2rem;
}

.day-count-btn {
  padding: 0.6rem 1rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: transparent;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.day-count-btn:hover {
  color: var(--color-text-primary);
  background-color: rgb(0 0 0 / 5%);
}

.day-count-btn.active {
  color: white;
  background-color: var(--color-primary, #4a90e2);
  font-weight: 600;
}

.day-count-btn:active {
  transform: scale(0.96);
}

/* 任务列表 */
.task-list {
  height: 100%;
  overflow-y: auto;
}
</style>

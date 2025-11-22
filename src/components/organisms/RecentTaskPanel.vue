<template>
  <div class="recent-view">
    <TwoRowLayout>
      <template #top>
        <div class="recent-controls">
          <div class="controls-left">
            <!-- 左右导航按钮 -->
            <button class="control-btn nav-btn" @click="navigatePrevious" title="上一天">
              <CuteIcon name="ChevronLeft" :size="16" />
            </button>
            <button class="control-btn nav-btn" @click="navigateNext" title="下一天">
              <CuteIcon name="ChevronRight" :size="16" />
            </button>

            <!-- 今天/日历合并按钮 -->
            <div class="combined-btn-wrapper">
              <!-- 左半边：今天 -->
              <button class="combined-btn-left" @click="goToToday" title="回到今天">
                <span>今天</span>
              </button>
              <!-- 右半边：日历选择器 -->
              <button class="combined-btn-right" @click="toggleDatePicker" title="选择日期">
                <CuteIcon name="CalendarDays" :size="16" />
              </button>
              <input
                ref="dateInputRef"
                type="date"
                v-model="selectedDate"
                class="date-input-hidden"
                @change="onDateChange"
              />
            </div>

            <!-- 天数选择器（CuteDropdown） -->
            <CuteDropdown
              v-model="dayCount"
              :options="dayCountOptions.map((c) => ({ value: c, label: `${c}天` }))"
              @change="onDayCountChange"
            >
              <template #trigger>
                <button class="day-count-trigger">
                  <span>{{ dayCount }}天</span>
                  <CuteIcon name="ChevronDown" :size="14" />
                </button>
              </template>
            </CuteDropdown>
          </div>

          <!-- 右侧筛选菜单 -->
          <div class="controls-right">
            <CuteDropdown :close-on-select="false">
              <template #trigger>
                <button class="filter-btn">
                  <CuteIcon :name="'Filter' as any" :size="16" />
                  <span>筛选</span>
                  <CuteIcon name="ChevronDown" :size="14" />
                </button>
              </template>
              <CuteDropdownItem @click.prevent>
                <label class="filter-option">
                  <CuteCheckbox
                    :checked="showDailyRecurringTasks"
                    size="small"
                    @update:checked="
                      (val) => {
                        showDailyRecurringTasks = val
                        onFilterChange()
                      }
                    "
                  />
                  <span>显示每日循环任务</span>
                </label>
              </CuteDropdownItem>
            </CuteDropdown>
          </div>
        </div>
      </template>
      <template #bottom>
        <div class="task-list">
          <!-- 动态生成的日期任务栏 -->
          <TaskList
            v-for="(dateInfo, index) in dateList"
            :key="dateInfo.viewKey"
            :title="dateInfo.label"
            :view-key="dateInfo.viewKey"
            :fill-remaining-space="index === dateList.length - 1"
            :hide-daily-recurring-tasks="!showDailyRecurringTasks"
          />
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteDropdown from '@/components/parts/CuteDropdown.vue'
import CuteDropdownItem from '@/components/parts/CuteDropdownItem.vue'
import { useTaskStore } from '@/stores/task'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString, parseDateString, toDateString } from '@/infra/utils/dateUtils'

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
  'date-change': [date: string]
}>()

const taskStore = useTaskStore()

// ==================== "最近"视图状态 ====================
// 确保 selectedDate 始终是有效的日期字符串
const getValidDateString = (): string => {
  const dateStr = getTodayDateString()
  if (dateStr) return dateStr
  // 兜底：使用当前日期（ISO 8601 格式）
  return toDateString(new Date())
}

const selectedDate = ref<string>(getValidDateString()) // 选择的起始日期
const dayCount = ref(props.modelValue) // 显示的天数
const dayCountOptions = [1, 3, 5, 7] // 可选的天数选项
const dateInputRef = ref<HTMLInputElement | null>(null) // 日期输入框引用

// 筛选菜单状态
const showDailyRecurringTasks = ref(true) // 默认显示每日循环任务

// 监听 props 变化
watch(
  () => props.modelValue,
  (newValue) => {
    dayCount.value = newValue
  }
)

// 监听 selectedDate 变化，通知父组件
watch(selectedDate, (newDate) => {
  emit('date-change', newDate)
})

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
    const dateString = shiftDate(startDate, i)

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
  const date = parseDateString(dateString)
  const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  const weekday = weekdays[date.getDay()]

  // 检查是否是今天
  if (dateString === today) {
    return `今天 ${weekday}`
  }

  // 检查是否是昨天
  const yesterdayString = shiftDate(today, -1)
  if (dateString === yesterdayString) {
    return `昨天 ${weekday}`
  }

  // 检查是否是明天
  const tomorrowString = shiftDate(today, 1)
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

// 导航到上一天
function navigatePrevious() {
  selectedDate.value = shiftDate(selectedDate.value, -dayCount.value)
  logger.info(LogTags.VIEW_HOME, 'Navigate previous', { date: selectedDate.value })
  loadDateRangeTasks()
}

// 导航到下一天
function navigateNext() {
  selectedDate.value = shiftDate(selectedDate.value, dayCount.value)
  logger.info(LogTags.VIEW_HOME, 'Navigate next', { date: selectedDate.value })
  loadDateRangeTasks()
}

// 切换日期选择器
function toggleDatePicker() {
  if (dateInputRef.value) {
    dateInputRef.value.showPicker()
  }
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

// 下拉菜单变化处理
function onDayCountChange() {
  emit('update:modelValue', dayCount.value) // 通知父组件
  logger.info(LogTags.VIEW_HOME, 'Day count changed', { dayCount: dayCount.value })
  loadDateRangeTasks()
}

// 筛选选项变化
function onFilterChange() {
  logger.info(LogTags.VIEW_HOME, 'Filter changed', {
    showDailyRecurringTasks: showDailyRecurringTasks.value,
  })
  // 筛选状态已通过 prop 传递给 TaskList 组件
}

// 预加载日期范围的任务
async function loadDateRangeTasks() {
  const dates = dateList.value.map((info) => info.date)
  if (dates.length === 0) {
    return
  }

  const sortedDates = [...dates].sort()
  const startDate = sortedDates[0]!
  const endDate = sortedDates[sortedDates.length - 1]!

  try {
    await taskStore.fetchDailyTasksRange_DMA(startDate, endDate)
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Failed to preload recent view date range',
      error instanceof Error ? error : new Error(String(error)),
      { startDate, endDate }
    )
  }
}

// 工具函数：在日期字符串基础上偏移指定天数
function shiftDate(baseDate: string, offsetDays: number): string {
  const date = parseDateString(baseDate)
  date.setDate(date.getDate() + offsetDays)
  return toDateString(date)
}

// 初始化
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing RecentTaskPanel component...')

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
  justify-content: space-between;
  gap: 1.2rem;
  padding: 1.2rem 1.6rem;
  background-color: transparent;
}

.controls-left {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.controls-right {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

/* 今天按钮 */
.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
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

/* 合并按钮（今天 + 日历） */
.combined-btn-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  height: 3.6rem;
}

.combined-btn-left,
.combined-btn-right {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 3.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.combined-btn-left {
  gap: 0.6rem;
  padding: 0 1.2rem;
  border-radius: 0.6rem 0 0 0.6rem;
  border-right: none;
}

.combined-btn-right {
  width: 3.6rem;
  padding: 0;
  border-radius: 0 0.6rem 0.6rem 0;
  border-left: 1px solid var(--color-border-default);
}

.combined-btn-left:hover,
.combined-btn-right:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
  z-index: 1;
}

.combined-btn-left:active,
.combined-btn-right:active {
  transform: scale(0.98);
}

.date-input-hidden {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  width: 0;
  height: 0;
}

/* 导航按钮 */
.nav-btn {
  width: 3.6rem;
  padding: 0;
}

/* 天数选择器触发器 */
.day-count-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  outline: none;
  min-width: 8rem;
  white-space: nowrap;
}

.day-count-trigger:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  border-color: var(--color-border-hover, var(--color-border-default));
}

/* ==================== 筛选下拉菜单 ==================== */
.filter-btn {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  min-width: 10rem;
}

.filter-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.filter-btn:active {
  transform: scale(0.98);
}

.filter-option {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  width: 100%;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  cursor: pointer;
  user-select: none;
}

.filter-option span {
  user-select: none;
}

/* 任务列表 */
.task-list {
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

/* 最后一个TaskList延展到底部，避免拖动到底部空白区域时闪烁 */
.task-list > :deep(:last-child) {
  flex: 1;
  min-height: auto;
}
</style>

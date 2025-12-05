<template>
  <div class="recent-view">
    <TwoRowLayout>
      <template #top>
        <div class="recent-controls">
          <div class="controls-left">
            <!-- 日期标题区域 -->
            <div class="date-title-wrapper">
              <!-- 日期标题（可点击） -->
              <div class="date-title" @click="toggleDatePanel">
                <span class="date-text">{{ currentDateDisplay }}</span>
              </div>
            </div>

            <!-- 日期导航面板 -->
            <div v-if="showDatePanel" class="date-nav-panel" ref="datePanelRef">
              <div class="panel-header">
                <span class="panel-title">选择日期</span>
              </div>

              <div class="panel-body">
                <!-- 导航控制 -->
                <div class="nav-row">
                  <button class="panel-nav-btn" @click="navigatePrevious" title="上一页">
                    <CuteIcon name="ChevronLeft" :size="16" />
                  </button>
                  <span class="current-range">{{ dateRangeDisplay }}</span>
                  <button class="panel-nav-btn" @click="navigateNext" title="下一页">
                    <CuteIcon name="ChevronRight" :size="16" />
                  </button>
                </div>

                <!-- 日期输入 -->
                <div class="date-input-row">
                  <label class="date-label">起始日期</label>
                  <input
                    type="date"
                    v-model="selectedDate"
                    class="date-input"
                    @change="onDateChange"
                  />
                </div>

                <!-- 今天按钮 -->
                <button class="today-btn" @click="goToToday">
                  <CuteIcon name="Calendar" :size="16" />
                  <span>回到今天</span>
                </button>
              </div>
            </div>
          </div>

          <!-- 右侧设置菜单（天数 + 筛选） -->
          <div class="controls-right">
            <CuteDropdown :close-on-select="false" :max-height="'none'">
              <template #trigger>
                <button class="icon-btn" title="设置">
                  <CuteIcon name="Settings" :size="18" />
                </button>
              </template>

              <!-- 天数选择（横向排布） -->
              <CuteDropdownItem disabled>
                <div class="day-count-row">
                  <span class="menu-section-label">天数</span>
                  <div class="day-count-buttons">
                    <button
                      v-for="option in dayCountOptions"
                      :key="option.value"
                      class="day-count-btn"
                      :class="{ active: dayCount === option.value }"
                      @click.stop="onDayCountChange(option.value)"
                    >
                      {{ option.value }}
                    </button>
                  </div>
                </div>
              </CuteDropdownItem>

              <!-- 分隔线 -->
              <CuteDropdownItem disabled>
                <div class="menu-divider"></div>
              </CuteDropdownItem>

              <!-- 筛选选项 -->
              <CuteDropdownItem @click.prevent>
                <label class="filter-option">
                  <CuteCheckbox
                    :checked="showCompletedTasks"
                    size="small"
                    @update:checked="
                      (val) => {
                        showCompletedTasks = val
                        onFilterChange()
                      }
                    "
                  />
                  <span>显示已完成任务</span>
                </label>
              </CuteDropdownItem>
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
            :hide-completed="!showCompletedTasks"
            :show-estimated-duration="false"
            :title-color="dateInfo.isToday ? 'var(--color-text-accent)' : undefined"
          />
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
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
const showDatePanel = ref(false) // 日期导航面板显示状态
const datePanelRef = ref<HTMLElement | null>(null) // 日期面板引用

// 筛选菜单状态
const showCompletedTasks = ref(true) // 默认显示已完成任务
const showDailyRecurringTasks = ref(true) // 默认显示每日循环任务

// 天数选项
const dayCountOptions = [
  { value: 1 },
  { value: 3 },
  { value: 5 },
]

// 当前日期显示（用于触发按钮）
const currentDateDisplay = computed(() => {
  // 直接使用日期范围显示
  return dateRangeDisplay.value
})

// 日期范围显示（用于面板内）
const dateRangeDisplay = computed(() => {
  const startDate = parseDateString(selectedDate.value)
  const endDate = parseDateString(shiftDate(selectedDate.value, dayCount.value - 1))
  const startMonth = startDate.getMonth() + 1
  const startDay = startDate.getDate()
  const endMonth = endDate.getMonth() + 1
  const endDay = endDate.getDate()

  if (startMonth === endMonth) {
    return `${startMonth}月${startDay}日 - ${endDay}日`
  }
  return `${startMonth}月${startDay}日 - ${endMonth}月${endDay}日`
})

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
  isToday: boolean
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
      isToday: dateString === today,
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
  showDatePanel.value = false
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

// 切换日期面板
function toggleDatePanel() {
  showDatePanel.value = !showDatePanel.value
}

// 点击外部关闭面板
function handleClickOutside(event: MouseEvent) {
  if (datePanelRef.value && !datePanelRef.value.contains(event.target as Node)) {
    const trigger = (event.target as Element).closest('.date-title')
    if (!trigger) {
      showDatePanel.value = false
    }
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

// 筛选选项变化
function onFilterChange() {
  logger.info(LogTags.VIEW_HOME, 'Filter changed', {
    showCompletedTasks: showCompletedTasks.value,
    showDailyRecurringTasks: showDailyRecurringTasks.value,
  })
  // 筛选状态已通过 prop 传递给 TaskList 组件
}

// 天数变化
function onDayCountChange(value: number) {
  dayCount.value = value
  emit('update:modelValue', value)
  logger.info(LogTags.VIEW_HOME, 'Day count changed', { dayCount: value })
  loadDateRangeTasks()
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

  // 添加点击外部关闭面板的监听器
  document.addEventListener('click', handleClickOutside)

  // 加载日期范围的任务
  await loadDateRangeTasks()
})

// 清理
onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
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
  position: relative;
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.controls-right {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

/* ==================== 日期标题样式 ==================== */
.date-title-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.date-title {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.date-title:hover {
  opacity: 0.7;
}

.date-title:active {
  opacity: 0.5;
}

.date-text {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
  white-space: nowrap;
}

/* ==================== 日期导航面板 ==================== */
.date-nav-panel {
  position: absolute;
  top: calc(100% + 0.8rem);
  left: 0;
  z-index: 100;
  min-width: 24rem;
  background-color: var(--color-background-primary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  overflow: hidden;
}

.panel-header {
  padding: 1.2rem 1.6rem;
  border-bottom: 1px solid var(--color-border-light, #f0f);
}

.panel-title {
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

.panel-body {
  padding: 1.2rem 1.6rem;
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

/* 导航行 */
.nav-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.panel-nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.panel-nav-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover, #f0f);
}

.panel-nav-btn:active {
  transform: scale(0.95);
}

.current-range {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

/* 日期输入行 */
.date-input-row {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.date-label {
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary, #f0f);
  line-height: 1.4;
}

.date-input {
  width: 100%;
  height: 3.6rem;
  padding: 0 1rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.date-input:hover {
  border-color: var(--color-border-hover, #f0f);
}

.date-input:focus {
  outline: none;
  border-color: var(--color-accent-primary, #f0f);
}

/* 今天按钮 */
.today-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.today-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover, #f0f);
}

.today-btn:active {
  transform: scale(0.98);
}

/* ==================== 筛选图标按钮 ==================== */
.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.6rem;
  height: 3.6rem;
  padding: 0;
  color: var(--color-text-secondary, #f0f);
  background-color: transparent;
  border: 1px solid transparent;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.icon-btn:hover {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-default, #f0f);
}

.icon-btn:active {
  transform: scale(0.95);
}

.filter-option {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  width: 100%;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  cursor: pointer;
  user-select: none;
}

.filter-option span {
  user-select: none;
}

/* 菜单分组标签 */
.menu-section-label {
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-tertiary, #f0f);
}

/* 菜单分隔线 */
.menu-divider {
  height: 1px;
  background-color: var(--color-border-light, #f0f);
  margin: 0.4rem 0;
}

/* 天数选择行 */
.day-count-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.2rem;
  width: 100%;
}

.day-count-buttons {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.day-count-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 2.8rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-secondary, #f0f);
  background-color: transparent;
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.day-count-btn:hover {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-hover, #f0f);
}

.day-count-btn.active {
  color: var(--color-text-on-accent, #f0f);
  background-color: var(--color-accent-primary, #f0f);
  border-color: var(--color-accent-primary, #f0f);
}

/* 任务列表 */
.task-list {
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  padding: 1rem;
}

/* 最后一个TaskList延展到底部，避免拖动到底部空白区域时闪烁 */
.task-list > :deep(:last-child) {
  flex: 1;
  min-height: auto;
}
</style>

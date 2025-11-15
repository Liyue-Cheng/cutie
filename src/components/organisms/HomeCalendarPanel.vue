<template>
  <div class="home-calendar-panel">
    <TwoRowLayout>
      <template #top>
        <div class="calendar-controls">
          <!-- 左侧：年月显示 -->
          <div v-if="currentRightPaneView === 'calendar'" class="calendar-year-month">
            {{ calendarYearMonth }}
          </div>

          <!-- 中间：占位 -->
          <div class="spacer"></div>

          <!-- 右侧控制组 -->
          <div class="controls-right">
            <!-- 缩放按钮（仅日历视图显示） -->
            <button
              v-if="currentRightPaneView === 'calendar'"
              class="zoom-btn"
              @click="cycleZoom"
              title="切换缩放"
            >
              {{ calendarZoom }}x
            </button>

            <!-- 月视图筛选菜单 -->
            <CuteDropdown
              v-if="currentRightPaneView === 'calendar' && effectiveCalendarViewType === 'month'"
              :close-on-select="false"
            >
              <template #trigger>
                <button class="filter-btn">
                  <span>筛选</span>
                  <CuteIcon name="ChevronDown" :size="14" />
                </button>
              </template>
              <CuteDropdownItem @click.prevent>
                <label class="filter-option">
                  <CuteCheckbox
                    :checked="monthViewFilters.showRecurringTasks"
                    size="small"
                    @update:checked="(val) => (monthViewFilters.showRecurringTasks = val)"
                  />
                  <span>循环任务</span>
                </label>
              </CuteDropdownItem>
              <CuteDropdownItem @click.prevent>
                <label class="filter-option">
                  <CuteCheckbox
                    :checked="monthViewFilters.showScheduledTasks"
                    size="small"
                    @update:checked="(val) => (monthViewFilters.showScheduledTasks = val)"
                  />
                  <span>已排期任务</span>
                </label>
              </CuteDropdownItem>
              <CuteDropdownItem @click.prevent>
                <label class="filter-option">
                  <CuteCheckbox
                    :checked="monthViewFilters.showDueDates"
                    size="small"
                    @update:checked="(val) => (monthViewFilters.showDueDates = val)"
                  />
                  <span>截止日期</span>
                </label>
              </CuteDropdownItem>
              <CuteDropdownItem @click.prevent>
                <label class="filter-option">
                  <CuteCheckbox
                    :checked="monthViewFilters.showAllDayEvents"
                    size="small"
                    @update:checked="(val) => (monthViewFilters.showAllDayEvents = val)"
                  />
                  <span>全天事件</span>
                </label>
              </CuteDropdownItem>
            </CuteDropdown>

            <!-- 视图选择下拉菜单 -->
            <CuteDropdown v-model="currentRightPaneView">
              <template #trigger>
                <button class="view-selector-btn">
                  <span>{{ viewSelectorLabel }}</span>
                  <CuteIcon name="ChevronDown" :size="14" />
                </button>
              </template>
              <CuteDropdownItem
                v-for="view in viewOptions"
                :key="view.value"
                :label="view.label"
                :active="currentRightPaneView === view.value"
                @click="currentRightPaneView = view.value"
              />
            </CuteDropdown>
          </div>
        </div>
      </template>

      <template #bottom>
        <!-- 日历视图 -->
        <div v-if="currentRightPaneView === 'calendar'" class="calendar-wrapper">
          <CuteCalendar
            ref="calendarRef"
            :current-date="currentCalendarDate"
            :view-type="effectiveCalendarViewType"
            :zoom="calendarZoom"
            :days="calendarDays"
            :month-view-filters="monthViewFilters"
          />
        </div>
        <!-- 时间线视图 -->
        <DoubleRowTimeline
          v-else-if="currentRightPaneView === 'timeline'"
          :current-month="currentCalendarDate.slice(0, 7)"
          :month-view-filters="monthViewFilters"
        />
        <!-- Staging 视图 -->
        <StagingList v-else-if="currentRightPaneView === 'staging'" />
        <!-- Upcoming 视图 -->
        <UpcomingList v-else-if="currentRightPaneView === 'upcoming'" />
        <!-- Templates 视图 -->
        <TemplateList v-else-if="currentRightPaneView === 'templates'" />
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import CuteCalendar from '@/components/assembles/calender/CuteCalendar.vue'
import DoubleRowTimeline from '@/components/parts/timeline/DoubleRowTimeline.vue'
import StagingList from '@/components/assembles/tasks/list/StagingList.vue'
import UpcomingList from '@/components/assembles/tasks/list/UpcomingList.vue'
import TemplateList from '@/components/assembles/template/TemplateList.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteDropdown from '@/components/parts/CuteDropdown.vue'
import CuteDropdownItem from '@/components/parts/CuteDropdownItem.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

// Props
interface Props {
  currentCalendarDate?: string
  calendarDays?: 1 | 3 | 5 | 7
  leftViewType?: 'recent' | 'staging'
}

const props = withDefaults(defineProps<Props>(), {
  currentCalendarDate: () => getTodayDateString(),
  calendarDays: 3,
  leftViewType: 'recent',
})

// Emits
const emit = defineEmits<{
  'calendar-size-update': []
}>()

// ==================== 右栏视图状态 ====================
type RightPaneView = 'calendar' | 'staging' | 'upcoming' | 'templates' | 'timeline'
const currentRightPaneView = ref<RightPaneView>('calendar')

const viewOptions = [
  { value: 'calendar', label: '日历' },
  { value: 'timeline', label: '时间线' },
  { value: 'staging', label: 'Staging' },
  { value: 'upcoming', label: 'Upcoming' },
  { value: 'templates', label: 'Templates' },
] as const

const viewSelectorLabel = computed(() => {
  const option = viewOptions.find((opt) => opt.value === currentRightPaneView.value)
  return option?.label || '日历'
})

// ==================== 日历状态 ====================
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)
const calendarZoom = ref<1 | 2 | 3>(1)

// 月视图筛选状态
const monthViewFilters = ref({
  showRecurringTasks: true,
  showScheduledTasks: true,
  showDueDates: true,
  showAllDayEvents: true,
})

// 根据天数计算视图类型：7天显示本周视图，其他显示多天视图
const calendarViewType = computed(() => {
  return props.calendarDays === 7 ? 'week' : 'day'
})

// 最终的日历视图类型：Staging 视图强制使用月视图
const effectiveCalendarViewType = computed(() => {
  if (props.leftViewType === 'staging') {
    return 'month'
  }
  return calendarViewType.value
})

// 格式化日历年月显示
const calendarYearMonth = computed(() => {
  const dateStr = props.currentCalendarDate
  if (!dateStr) return ''

  const date = new Date(dateStr)
  const year = date.getFullYear()
  const month = date.getMonth() + 1

  return `${year}年${month}月`
})

// 循环切换缩放等级
function cycleZoom() {
  if (calendarZoom.value === 1) {
    calendarZoom.value = 2
  } else if (calendarZoom.value === 2) {
    calendarZoom.value = 3
  } else {
    calendarZoom.value = 1
  }
  logger.debug(LogTags.COMPONENT_HOME_CALENDAR_PANEL, 'Calendar zoom cycled', {
    zoom: calendarZoom.value,
  })
}

// 通知父组件需要更新日历尺寸
function notifyCalendarSizeUpdate() {
  emit('calendar-size-update')
}

// 监听视图切换，通知父组件更新日历尺寸
watch(currentRightPaneView, () => {
  notifyCalendarSizeUpdate()
})

// 暴露方法给父组件
defineExpose({
  calendarRef,
})
</script>

<style scoped>
.home-calendar-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ==================== 控制栏 ==================== */
.calendar-controls {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.2rem;
  padding: 1.2rem 1.6rem;
  background-color: transparent;
}

.controls-right {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

/* 年月显示 */
.calendar-year-month {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
}

/* 占位 */
.spacer {
  flex: 1;
}

/* 缩放按钮 */
.zoom-btn {
  display: flex;
  align-items: center;
  justify-content: center;
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
  min-width: 5.6rem;
}

.zoom-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.zoom-btn:active {
  transform: scale(0.98);
}

/* 筛选按钮 */
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

/* 筛选选项 */
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

/* 视图选择器按钮 */
.view-selector-btn {
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

.view-selector-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.view-selector-btn:active {
  transform: scale(0.98);
}

/* 日历包装器 */
.calendar-wrapper {
  height: 100%;
  width: 100%;
  overflow: hidden;
}
</style>

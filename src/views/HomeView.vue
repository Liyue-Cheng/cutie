<template>
  <div class="home-view">
    <!-- 左栏 -->
    <div class="left-column" :style="{ width: leftPaneWidth + '%' }">
      <RecentTaskPanel
        v-if="currentView === 'recent'"
        v-model="calendarDays"
        @date-change="onRecentDateChange"
      />
      <StagingTaskPanel v-else-if="currentView === 'staging'" />
    </div>

    <!-- 可拖动的分割线 -->
    <div class="divider" @mousedown="startDragging" @dblclick="resetPaneWidth"></div>

    <!-- 右栏 -->
    <div class="right-column">
      <TwoRowLayout>
        <template #top>
          <div class="calendar-header">
            <!-- 最左侧：日历视图专属年月显示 -->
            <div v-if="currentRightPaneView === 'calendar'" class="calendar-year-month">
              {{ calendarYearMonth }}
            </div>

            <!-- 中间：占位 -->
            <div class="spacer"></div>

            <!-- 日历视图专属：缩放按钮（右侧，日历下拉菜单左边） -->
            <button
              v-if="currentRightPaneView === 'calendar'"
              class="zoom-toggle-btn"
              @click="cycleZoom"
            >
              {{ calendarZoom }}x
            </button>

            <!-- 月视图筛选按钮 -->
            <div
              v-if="currentRightPaneView === 'calendar' && effectiveCalendarViewType === 'month'"
              class="filter-dropdown"
            >
              <button class="filter-btn" @click="toggleFilterMenu">
                筛选
                <span class="filter-icon">▼</span>
              </button>
              <div v-if="showFilterMenu" class="filter-menu">
                <label class="filter-item">
                  <input v-model="monthViewFilters.showRecurringTasks" type="checkbox" />
                  <span>循环任务</span>
                </label>
                <label class="filter-item">
                  <input v-model="monthViewFilters.showScheduledTasks" type="checkbox" />
                  <span>已排期任务</span>
                </label>
                <label class="filter-item">
                  <input v-model="monthViewFilters.showDueDates" type="checkbox" />
                  <span>截止日期</span>
                </label>
                <label class="filter-item">
                  <input v-model="monthViewFilters.showAllDayEvents" type="checkbox" />
                  <span>全天事件</span>
                </label>
              </div>
            </div>

            <!-- 最右侧：视图选择下拉菜单 -->
            <select v-model="currentRightPaneView" class="view-selector">
              <option value="calendar">日历</option>
              <option value="timeline">时间线</option>
              <option value="staging">Staging</option>
              <option value="upcoming">Upcoming</option>
              <option value="templates">Templates</option>
            </select>
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

    <!-- 任务编辑器模态框挂载点 -->
    <KanbanTaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import RecentTaskPanel from '@/components/organisms/RecentTaskPanel.vue'
import StagingTaskPanel from '@/components/organisms/StagingTaskPanel.vue'
import StagingList from '@/components/assembles/tasks/list/StagingList.vue'
import UpcomingList from '@/components/assembles/tasks/list/UpcomingList.vue'
import TemplateList from '@/components/assembles/template/TemplateList.vue'
import CuteCalendar from '@/components/assembles/calender/CuteCalendar.vue'
import DoubleRowTimeline from '@/components/parts/timeline/DoubleRowTimeline.vue'
import { useRegisterStore } from '@/stores/register'
import { useUIStore } from '@/stores/ui'
import KanbanTaskEditorModal from '@/components/assembles/tasks/kanban/KanbanTaskEditorModal.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

const route = useRoute()
const registerStore = useRegisterStore()
const uiStore = useUIStore()

// ==================== 视图切换状态 ====================
const currentView = ref<'recent' | 'staging'>('recent') // 当前视图

// ==================== 右栏视图状态 ====================
type RightPaneView = 'calendar' | 'staging' | 'upcoming' | 'templates' | 'timeline'
const currentRightPaneView = ref<RightPaneView>('calendar') // 右栏当前视图

// ==================== 日历天数联动状态 ====================
const calendarDays = ref<1 | 3 | 5 | 7>(3) // 默认显示3天，与 RecentTaskPanel 联动
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)
const calendarZoom = ref<1 | 2 | 3>(1) // 日历缩放倍率

// ==================== 月视图筛选状态 ====================
const monthViewFilters = ref({
  showRecurringTasks: true,
  showScheduledTasks: true,
  showDueDates: true,
  showAllDayEvents: true,
})

const showFilterMenu = ref(false)

function toggleFilterMenu() {
  showFilterMenu.value = !showFilterMenu.value
}

// 点击外部关闭筛选菜单
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.filter-dropdown')) {
    showFilterMenu.value = false
  }
}

// 根据天数计算视图类型：7天显示本周视图，其他显示多天视图
const calendarViewType = computed(() => {
  return calendarDays.value === 7 ? 'week' : 'day'
})

// 最终的日历视图类型：Staging 视图强制使用月视图
const effectiveCalendarViewType = computed(() => {
  if (currentView.value === 'staging') {
    return 'month'
  }
  return calendarViewType.value
})

// 监听路由变化，切换视图
watch(
  () => route.query.view,
  (newView) => {
    if (newView === 'staging') {
      currentView.value = 'staging'
      logger.info(LogTags.VIEW_HOME, 'Switched to Staging view')
    } else {
      currentView.value = 'recent'
      // 切换回 Recent 视图时，确保日历跳转到今天
      const today = getTodayDateString()
      registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, today)
      logger.info(LogTags.VIEW_HOME, 'Switched to Recent view', { date: today })
    }
  },
  { immediate: true }
)

// 处理 RecentTaskPanel 的日期变化
function onRecentDateChange(date: string) {
  // 更新寄存器中的日历日期
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, date)
  logger.debug(LogTags.VIEW_HOME, 'Calendar date synced from RecentTaskPanel', { date })
}

// 循环切换缩放等级
function cycleZoom() {
  if (calendarZoom.value === 1) {
    calendarZoom.value = 2
  } else if (calendarZoom.value === 2) {
    calendarZoom.value = 3
  } else {
    calendarZoom.value = 1
  }
  logger.debug(LogTags.VIEW_HOME, 'Calendar zoom cycled', { zoom: calendarZoom.value })
}

// 初始化
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing Home view with Recent + Calendar...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'home')
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})

// ==================== 日历状态 ====================
const currentCalendarDate = computed(() => {
  return (
    registerStore.readRegister<string>(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME) ||
    getTodayDateString()
  )
})

// 格式化日历年月显示
const calendarYearMonth = computed(() => {
  const dateStr = currentCalendarDate.value
  if (!dateStr) return ''

  const date = new Date(dateStr)
  const year = date.getFullYear()
  const month = date.getMonth() + 1

  return `${year}年${month}月`
})

// ==================== 可拖动分割线逻辑 ====================
const leftPaneWidth = ref(40) // 默认比例 2:3，左栏占 40%
const isDragging = ref(false)
let rafId: number | null = null

function startDragging(e: MouseEvent) {
  isDragging.value = true
  document.addEventListener('mousemove', onDragging)
  document.addEventListener('mouseup', stopDragging)
  e.preventDefault()
}

function onDragging(e: MouseEvent) {
  if (!isDragging.value) return

  const container = document.querySelector('.home-view') as HTMLElement
  if (!container) return

  const containerRect = container.getBoundingClientRect()
  const containerWidth = containerRect.width
  const mouseX = e.clientX - containerRect.left

  // 计算新的左栏宽度百分比
  let newWidth = (mouseX / containerWidth) * 100

  // 限制最小和最大宽度（20% - 80%）
  newWidth = Math.max(20, Math.min(80, newWidth))

  leftPaneWidth.value = newWidth

  // 使用 requestAnimationFrame 实现流畅的实时更新
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
  rafId = requestAnimationFrame(() => {
    updateCalendarSize()
    rafId = null
  })
}

// 更新日历尺寸的辅助函数
function updateCalendarSize() {
  if (calendarRef.value?.calendarRef) {
    const calendarApi = calendarRef.value.calendarRef.getApi()
    if (calendarApi) {
      calendarApi.updateSize()
      // 同步自定义头部的列宽
      nextTick(() => {
        calendarRef.value?.syncColumnWidths()
      })
    }
  }
}

async function stopDragging() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)

  // 清除 requestAnimationFrame
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }

  // 最后确保更新一次日历尺寸
  await nextTick()
  updateCalendarSize()
  logger.debug(LogTags.VIEW_HOME, 'Calendar size updated after pane resize')
}

// 双击重置为默认比例
async function resetPaneWidth() {
  leftPaneWidth.value = 40

  // 通知日历更新尺寸
  await nextTick()
  updateCalendarSize()
  logger.debug(LogTags.VIEW_HOME, 'Calendar size updated after pane reset')
}

// 清理事件监听器和动画帧
onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
})
</script>

<style scoped>
.home-view {
  width: 100%;
  height: 100%;
  display: flex;
  overflow: hidden;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

/* 左右栏 */
.left-column,
.right-column {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: transparent;
}

.left-column {
  flex-shrink: 0;
  position: relative;
}

.right-column {
  flex: 1;
  min-width: 0;
  position: relative;
}

/* 分割线 */
.divider {
  width: 1px;
  height: 100%;
  background-color: var(--color-border-default);
  cursor: col-resize;
  flex-shrink: 0;
  transition: background-color 0.2s;
  position: relative;
  z-index: 10;
}

/* 扩大可点击区域 */
.divider::before {
  content: '';
  position: absolute;
  inset: 0 -4px;
  cursor: col-resize;
}

.divider:hover {
  background-color: var(--color-border-hover, var(--color-border-default));
}

/* 日历头部 */
.calendar-header {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0 1rem;
}

/* 视图选择下拉菜单 */
.view-selector {
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
  outline: none;
}

.view-selector:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.view-selector:focus {
  outline: none;
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

/* 缩放切换按钮 */
.zoom-toggle-btn {
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

.zoom-toggle-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.zoom-toggle-btn:active {
  transform: scale(0.98);
}

/* 日历包装器 */
.calendar-wrapper {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

/* 列内容 */
.column-content {
  padding: 2rem;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.placeholder-text {
  font-size: 1.6rem;
  color: var(--color-text-secondary);
}

/* 筛选下拉菜单 */
.filter-dropdown {
  position: relative;
}

.filter-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
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

.filter-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.filter-icon {
  font-size: 1rem;
  transition: transform 0.2s ease;
}

.filter-menu {
  position: absolute;
  top: calc(100% + 0.4rem);
  right: 0;
  min-width: 16rem;
  background-color: var(--color-background-primary, #fff);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
  padding: 0.8rem 0;
  z-index: 1000;
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem 1.2rem;
  cursor: pointer;
  transition: background-color 0.2s ease;
  user-select: none;
}

.filter-item:hover {
  background-color: var(--color-background-hover, #f5f5f5);
}

.filter-item input[type='checkbox'] {
  width: 1.6rem;
  height: 1.6rem;
  cursor: pointer;
}

.filter-item span {
  font-size: 1.4rem;
  color: var(--color-text-primary);
}
</style>

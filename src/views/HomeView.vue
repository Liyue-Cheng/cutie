<template>
  <div class="home-view">
    <!-- 左栏 -->
    <div class="left-column" :style="{ width: leftPaneWidth + '%' }">
      <RecentView
        v-if="currentView === 'recent'"
        v-model="calendarDays"
        @date-change="onRecentDateChange"
      />
      <StagingView v-else-if="currentView === 'staging'" />
    </div>

    <!-- 可拖动的分割线 -->
    <div class="divider" @mousedown="startDragging" @dblclick="resetPaneWidth"></div>

    <!-- 右栏 -->
    <div class="right-column">
      <TwoRowLayout>
        <template #top>
          <div class="calendar-header">
            <!-- 左侧：视图选择下拉菜单 -->
            <select v-model="currentRightPaneView" class="view-selector">
              <option value="calendar">日历</option>
              <option value="staging">Staging</option>
              <option value="upcoming">Upcoming</option>
              <option value="templates">Templates</option>
            </select>

            <!-- 日历视图专属：年月显示 -->
            <div v-if="currentRightPaneView === 'calendar'" class="calendar-year-month">
              {{ calendarYearMonth }}
            </div>

            <!-- 中间：占位 -->
            <div class="spacer"></div>

            <!-- 日历视图专属：缩放按钮 -->
            <div v-if="currentRightPaneView === 'calendar'" class="calendar-zoom-controls">
              <button
                :class="['zoom-btn', 'zoom-btn-left', { active: calendarZoom === 1 }]"
                @click="calendarZoom = 1"
              >
                1x
              </button>
              <button
                :class="['zoom-btn', 'zoom-btn-middle', { active: calendarZoom === 2 }]"
                @click="calendarZoom = 2"
              >
                2x
              </button>
              <button
                :class="['zoom-btn', 'zoom-btn-right', { active: calendarZoom === 3 }]"
                @click="calendarZoom = 3"
              >
                3x
              </button>
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
            />
          </div>
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
import RecentView from '@/components/templates/RecentView.vue'
import StagingView from '@/components/templates/StagingView.vue'
import StagingList from '@/components/parts/StagingList.vue'
import UpcomingList from '@/components/parts/UpcomingList.vue'
import TemplateList from '@/components/parts/template/TemplateList.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import { useRegisterStore } from '@/stores/register'
import { useUIStore } from '@/stores/ui'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

const route = useRoute()
const registerStore = useRegisterStore()
const uiStore = useUIStore()

// ==================== 视图切换状态 ====================
const currentView = ref<'recent' | 'staging'>('recent') // 当前视图

// ==================== 右栏视图状态 ====================
type RightPaneView = 'calendar' | 'staging' | 'upcoming' | 'templates'
const currentRightPaneView = ref<RightPaneView>('calendar') // 右栏当前视图

// ==================== 日历天数联动状态 ====================
const calendarDays = ref<1 | 3 | 5 | 7>(3) // 默认显示3天，与 RecentView 联动
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)
const calendarZoom = ref<1 | 2 | 3>(1) // 日历缩放倍率

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

// 处理 RecentView 的日期变化
function onRecentDateChange(date: string) {
  // 更新寄存器中的日历日期
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, date)
  logger.debug(LogTags.VIEW_HOME, 'Calendar date synced from RecentView', { date })
}

// 初始化
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing Home view with Recent + Calendar...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'home')
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
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgb(74 144 226 / 10%);
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

/* 缩放按钮组 */
.calendar-zoom-controls {
  display: flex;
  height: 3.6rem;
}

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
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  min-width: 3.6rem;
}

/* 左侧按钮 */
.zoom-btn-left {
  border-radius: 0.6rem 0 0 0.6rem;
  border-right: none;
}

/* 中间按钮 */
.zoom-btn-middle {
  border-radius: 0;
  border-right: none;
  border-left: 1px solid var(--color-border-default);
}

/* 右侧按钮 */
.zoom-btn-right {
  border-radius: 0 0.6rem 0.6rem 0;
  border-left: 1px solid var(--color-border-default);
}

.zoom-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
  z-index: 1;
}

.zoom-btn:active {
  transform: scale(0.98);
}

.zoom-btn.active {
  color: var(--color-primary);
  background-color: var(--color-primary-bg);
  border-color: var(--color-primary);
  font-weight: 600;
  z-index: 2;
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
</style>

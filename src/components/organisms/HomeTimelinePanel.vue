<template>
  <div class="home-timeline-panel">
    <TwoRowLayout>
      <template #top>
        <div class="panel-controls">
          <!-- 左侧：年月标题（可点击展开导航面板） -->
          <div class="controls-left">
            <div class="date-title-wrapper">
              <div class="date-title" @click="toggleNavPanel">
                <span class="date-text">{{ yearMonth }}</span>
              </div>
            </div>

            <!-- 导航面板 -->
            <div v-if="showNavPanel" ref="navPanelRef" class="date-nav-panel">
              <div class="panel-header">
                <span class="panel-title">导航</span>
              </div>
              <div class="panel-body">
                <!-- 导航控制 -->
                <div class="nav-row">
                  <button class="panel-nav-btn" title="上一月" @click="navigatePrevious">
                    <CuteIcon name="ChevronLeft" :size="16" />
                  </button>
                  <span class="current-range">{{ yearMonth }}</span>
                  <button class="panel-nav-btn" title="下一月" @click="navigateNext">
                    <CuteIcon name="ChevronRight" :size="16" />
                  </button>
                </div>

                <!-- 日期输入 -->
                <div class="date-input-row">
                  <label class="date-label">跳转到日期</label>
                  <input
                    type="date"
                    v-model="currentDate"
                    class="date-input"
                    @change="onDatePickerChange"
                  />
                </div>

                <!-- 快捷按钮 -->
                <button class="today-btn" @click="goToToday">
                  <CuteIcon name="Calendar" :size="16" />
                  <span>回到本月</span>
                </button>
              </div>
            </div>
          </div>

          <!-- 中间：占位 -->
          <div class="spacer"></div>

          <!-- 右侧控制组 -->
          <div class="controls-right">
            <!-- 导航按钮组 -->
            <div class="nav-buttons">
              <button class="nav-btn" title="上一月" @click="navigatePrevious">
                <CuteIcon name="ChevronLeft" :size="18" />
              </button>
              <button class="nav-btn" title="下一月" @click="navigateNext">
                <CuteIcon name="ChevronRight" :size="18" />
              </button>
              <button class="nav-btn today-nav-btn" title="回到本月" @click="goToToday">
                <CuteIcon name="Calendar" :size="18" />
              </button>
            </div>
          </div>
        </div>
      </template>

      <template #bottom>
        <DoubleRowTimeline
          :current-month="currentDate.slice(0, 7)"
          :month-view-filters="monthViewFilters"
          :layout-mode="props.layoutMode"
        />
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import DoubleRowTimeline from '@/components/parts/timeline/DoubleRowTimeline.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString, toDateString } from '@/infra/utils/dateUtils'

// Props
interface Props {
  layoutMode?: 'auto' | 'single' | 'double'
}

const props = withDefaults(defineProps<Props>(), {
  layoutMode: 'auto',
})

// 状态
const currentDate = ref<string>(getTodayDateString())
const showNavPanel = ref(false)
const navPanelRef = ref<HTMLElement | null>(null)

// 月视图筛选状态
const monthViewFilters = ref({
  showRecurringTasks: true,
  showScheduledTasks: true,
  showDueDates: true,
  showAllDayEvents: true,
})

// 年月显示
const yearMonth = computed(() => {
  const dateStr = currentDate.value
  if (!dateStr) return ''

  const date = new Date(dateStr)
  const year = date.getFullYear()
  const month = date.getMonth() + 1

  return `${year}年${month}月`
})

// 切换导航面板
function toggleNavPanel() {
  showNavPanel.value = !showNavPanel.value
}

// 点击外部关闭面板
function handleClickOutside(event: MouseEvent) {
  if (navPanelRef.value && !navPanelRef.value.contains(event.target as Node)) {
    const trigger = (event.target as Element).closest('.date-title')
    if (!trigger) {
      showNavPanel.value = false
    }
  }
}

// 导航：上一月
function navigatePrevious() {
  const date = new Date(currentDate.value)
  date.setMonth(date.getMonth() - 1)
  currentDate.value = toDateString(date)
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Timeline navigate previous', { date: currentDate.value })
}

// 导航：下一月
function navigateNext() {
  const date = new Date(currentDate.value)
  date.setMonth(date.getMonth() + 1)
  currentDate.value = toDateString(date)
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Timeline navigate next', { date: currentDate.value })
}

// 跳转到本月
function goToToday() {
  currentDate.value = getTodayDateString()
  showNavPanel.value = false
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Timeline go to today')
}

// 日期选择器变化
function onDatePickerChange() {
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Timeline date picker changed', { date: currentDate.value })
}

// 生命周期
onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.home-timeline-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ==================== 控制栏 ==================== */
.panel-controls {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.2rem;
  padding: 1.2rem 0.8rem 1.2rem 1.6rem;
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
  gap: 0.4rem;
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

/* ==================== 导航面板 ==================== */
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
  border-color: var(--color-border-focus, #f0f);
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

/* ==================== 导航按钮组 ==================== */
.nav-buttons {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.nav-btn {
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

.nav-btn:hover {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-default, #f0f);
}

.nav-btn:active {
  transform: scale(0.95);
}

/* 占位 */
.spacer {
  flex: 1;
}
</style>

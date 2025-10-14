<template>
  <div
    class="calendar-container"
    :class="`zoom-${currentZoom}x`"
    @dragenter="drag.handleDragEnter"
    @dragover="drag.handleDragOver"
    @dragleave="drag.handleDragLeave"
    @drop="drag.handleDrop"
  >
    <!-- 日期显示栏 -->
    <div class="calendar-header">
      <div class="date-display">
        <span class="date-text">{{ formattedDate }}</span>
      </div>
    </div>

    <FullCalendar ref="calendarRef" :options="calendarOptions" />

    <!-- 装饰竖线（跨越 TwoRowLayout 可视区域） -->
    <div
      v-if="
        decorativeLinePosition !== null &&
        decorativeLineTop !== null &&
        decorativeLineHeight !== null
      "
      class="decorative-line"
      :style="{
        left: `${decorativeLinePosition}px`,
        top: `${decorativeLineTop}px`,
        height: `${decorativeLineHeight}px`,
      }"
    ></div>

    <!-- 时间块详情面板 -->
    <TimeBlockDetailPanel
      v-if="selectedTimeBlockId"
      :time-block-id="selectedTimeBlockId"
      @close="selectedTimeBlockId = null"
    />
  </div>
</template>

<script setup lang="ts">
import FullCalendar from '@fullcalendar/vue3'
import { computed, ref, nextTick, watch, onMounted } from 'vue'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useAutoScroll } from '@/composables/calendar/useAutoScroll'
import { useTimePosition } from '@/composables/calendar/useTimePosition'
import { useDecorativeLine } from '@/composables/calendar/useDecorativeLine'
import { useCalendarEvents } from '@/composables/calendar/useCalendarEvents'
import { useCalendarHandlers } from '@/composables/calendar/useCalendarHandlers'
import { useCalendarOptions } from '@/composables/calendar/useCalendarOptions'
import { logger, LogTags } from '@/infra/logging/logger'
import { useCalendarDrag } from '@/composables/calendar/useCalendarDrag'
import TimeBlockDetailPanel from './TimeBlockDetailPanel.vue'

const timeBlockStore = useTimeBlockStore()

// ==================== Props ====================
const props = defineProps<{
  currentDate?: string // YYYY-MM-DD 格式的日期
  zoom?: 1 | 2 | 3 // 缩放倍率
}>()

// 默认缩放倍率为 1
const currentZoom = computed(() => props.zoom ?? 1)

// FullCalendar 引用
const calendarRef = ref<InstanceType<typeof FullCalendar> | null>(null)
const currentDateRef = computed(() => props.currentDate)

// 选中的时间块ID（用于显示详情面板）
const selectedTimeBlockId = ref<string | null>(null)

// ==================== Composables ====================
// 自动滚动
const { handleAutoScroll, stopAutoScroll } = useAutoScroll()

// 时间位置计算
const { getTimeFromDropPosition, clearCache, resetCache } = useTimePosition(calendarRef)

// 装饰线
const decorativeLine = useDecorativeLine(calendarRef, currentDateRef)
decorativeLine.initialize()

// 拖拽功能
const drag = useCalendarDrag(calendarRef, {
  getTimeFromDropPosition,
  clearCache,
  resetCache,
  handleAutoScroll,
  stopAutoScroll,
})
drag.initialize()

// 日历事件数据
const { calendarEvents } = useCalendarEvents(drag.previewEvent)

// 事件处理器
const handlers = useCalendarHandlers(drag.previewEvent, currentDateRef, selectedTimeBlockId)

// 日历配置
const { calendarOptions } = useCalendarOptions(calendarEvents, handlers)

// 装饰线位置（用于模板绑定）
const decorativeLinePosition = decorativeLine.position
const decorativeLineTop = decorativeLine.top
const decorativeLineHeight = decorativeLine.height

// ==================== 日期显示 ====================
// 格式化日期显示
const formattedDate = computed(() => {
  const dateToDisplay = props.currentDate || new Date().toISOString().split('T')[0]
  const date = new Date(dateToDisplay + 'T00:00:00')

  const year = date.getFullYear()
  const month = date.getMonth() + 1
  const day = date.getDate()
  const weekDays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  const weekDay = weekDays[date.getDay()]

  return `${year}年${month}月${day}日 ${weekDay}`
})

// ==================== 日期切换功能 ====================
// 监听 currentDate prop 变化，切换日历显示的日期
watch(
  () => props.currentDate,
  (newDate, oldDate) => {
    // 🔍 检查点3：日历日期同步
    logger.debug(LogTags.COMPONENT_CALENDAR, 'Date changed', { oldDate, newDate })

    if (newDate && calendarRef.value) {
      const calendarApi = calendarRef.value.getApi()
      if (calendarApi) {
        logger.info(LogTags.COMPONENT_CALENDAR, 'Switching to date', { newDate })
        calendarApi.gotoDate(newDate)

        // 🔧 FIX: 清除缓存，强制重新计算位置
        clearCache()

        // 🔍 检查点3：确认切换后的日期
        logger.debug(LogTags.COMPONENT_CALENDAR, 'After gotoDate', {
          currentDate: calendarApi.getDate().toISOString().split('T')[0],
        })
      }
    }
  },
  { immediate: false }
)

// 缩放变化：强制更新日历尺寸并重算装饰线，同时保持当前日期和滚动位置比例
watch(
  () => props.zoom,
  async () => {
    // 保存滚动位置比例（在DOM更新前）
    let scrollRatio = 0
    let scrollerEl: HTMLElement | null = null
    if (calendarRef.value) {
      const el = calendarRef.value.$el as HTMLElement
      scrollerEl = el.querySelector('.fc-scroller-liquid-absolute') as HTMLElement
      if (scrollerEl) {
        const scrollTop = scrollerEl.scrollTop
        const scrollHeight = scrollerEl.scrollHeight
        const clientHeight = scrollerEl.clientHeight
        const maxScroll = scrollHeight - clientHeight
        // 计算滚动比例（0到1之间）
        scrollRatio = maxScroll > 0 ? scrollTop / maxScroll : 0
      }
    }

    await nextTick()
    if (calendarRef.value) {
      try {
        const api = calendarRef.value.getApi()
        // 保存当前日期
        const currentDate = api.getDate()
        // 更新尺寸
        api.updateSize()
        // 恢复到之前的日期
        api.gotoDate(currentDate)

        // 根据比例恢复滚动位置
        await nextTick()
        if (scrollerEl) {
          const newScrollHeight = scrollerEl.scrollHeight
          const newClientHeight = scrollerEl.clientHeight
          const newMaxScroll = newScrollHeight - newClientHeight
          // 按比例计算新的滚动位置
          scrollerEl.scrollTop = newMaxScroll * scrollRatio
        }
      } catch {}
    }
    decorativeLine.updatePosition()
  }
)

onMounted(async () => {
  // 🔍 检查点2：全局 drop 捕获监听（检测是否被内部拦截）
  document.addEventListener(
    'drop',
    (e) => {
      const target = e.target as HTMLElement
      logger.debug(LogTags.COMPONENT_CALENDAR, 'Global drop capture', {
        targetClass: target?.className,
        tagName: target?.tagName,
      })
    },
    true
  ) // 捕获阶段

  // 使用 nextTick 确保DOM完全渲染后再获取数据
  await nextTick()

  try {
    // 🔧 FIX: 加载更大的时间范围（前后各 3 个月），避免切换日历时看不到数据
    const today = new Date()
    const startDate = new Date(today.getFullYear(), today.getMonth() - 3, 1) // 3个月前
    const endDate = new Date(today.getFullYear(), today.getMonth() + 4, 0) // 3个月后（下个月的0号=本月最后一天）

    logger.debug(LogTags.COMPONENT_CALENDAR, 'Loading time blocks for range', {
      startDate: startDate.toISOString(),
      endDate: endDate.toISOString(),
    })
    await timeBlockStore.fetchTimeBlocksForRange(startDate.toISOString(), endDate.toISOString())

    // 如果有初始日期，切换到该日期
    if (props.currentDate && calendarRef.value) {
      const calendarApi = calendarRef.value.getApi()
      if (calendarApi) {
        logger.debug(LogTags.COMPONENT_CALENDAR, 'Setting initial date', {
          currentDate: props.currentDate,
        })
        calendarApi.gotoDate(props.currentDate)
      }
    }

    // 计算装饰竖线位置
    await nextTick()
    decorativeLine.updatePosition()
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'Failed to fetch initial time blocks',
      error instanceof Error ? error : new Error(String(error))
    )
  }
})
</script>

<style>
/*
 * ===============================================
 * FullCalendar 自定义样式
 * ===============================================
 * 
 * 本文件包含对 FullCalendar 组件的所有自定义样式修改，
 * 按功能模块分组，便于维护和理解。
 */

/* ===============================================
 * 0. 日历容器样式
 * =============================================== */
.calendar-container {
  height: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 日历头部固定高度 */
.calendar-header {
  flex-shrink: 0;
  padding: 1rem 1.5rem;
  background: var(--color-background);
  border-bottom: 1px solid var(--color-border);
}

/* FullCalendar 占据剩余空间 */
.calendar-container > :nth-child(2) {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* 预览事件样式 */
.fc-event.preview-event {
  background-color: #bceaee !important;
  color: #fff !important;
  border-color: #357abd !important;
  pointer-events: none !important; /* 允许命中检测到下方的真实事件，避免阻挡 */
}

/* 创建中事件样式 */
.fc-event.creating-event {
  background-color: #bceaee !important;
  color: #fff !important;
  border-color: #357abd !important;
  opacity: 0.8;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.8;
  }

  50% {
    opacity: 1;
  }
}

/* 当前时间指示器样式 */
.fc-timegrid-now-indicator-line {
  border-color: #ff6b6b !important;
  border-width: 2px !important;
  z-index: 10 !important;
}

.fc-timegrid-now-indicator-arrow {
  border-left-color: #ff6b6b !important;
  border-right-color: #ff6b6b !important;
}

/* ===============================================
 * 1. 今日高亮样式
 * =============================================== */
.fc .fc-day-today {
  background-color: transparent !important; /* 移除今日的默认蓝色背景 */
}

/* ===============================================
 * 2. 时间标签样式修复
 * =============================================== */

/* 时间标签垂直居中 */
.fc .fc-timegrid-slot-label {
  transform: translateY(-50%);
}

/* 移除时间槽边框 */
.fc .fc-timegrid-slot-label,
.fc .fc-timegrid-slot-minor {
  border: none !important;
}

/* 为时间标签容器添加上边距，防止 translateY(-50%) 导致的裁切问题 */
.fc .fc-timegrid-slots {
  padding-top: 1rem !important;
}

/* ===============================================
 * 3. 滚动条样式美化
 * =============================================== */

/* 隐藏默认滚动条 */
.fc .fc-scroller::-webkit-scrollbar {
  width: 8px;
  background-color: transparent;
}

/* 滚动条轨道样式 */
.fc .fc-scroller::-webkit-scrollbar-track {
  background-color: transparent;
}

/* 滚动条滑块样式 */
.fc .fc-scroller::-webkit-scrollbar-thumb {
  background-color: var(--color-border-hover);
  border-radius: 4px;
}

/* ===============================================
 * 4. 时间网格分隔线样式
 * =============================================== */
.fc .fc-timegrid-divider {
  padding: 0 !important; /* 增加分隔线区域的内边距 */
  border-bottom: none !important;
  background-color: transparent !important; /* 设置透明背景 */
}

/* ===============================================
 * 5. 边框移除 - 解决多余边框显示问题
 * =============================================== */

/* 移除主网格边框 */
.fc-theme-standard .fc-scrollgrid {
  border: none !important;
}

/* 移除表格单元格右边框 */
.fc-theme-standard td,
.fc-theme-standard th {
  border-right: none !important;
}

/* 移除特定容器的边框 */
.fc .fc-scrollgrid-section-liquid > td {
  border: none !important;
}

/* ===============================================
 * 6. 事件样式自定义
 * =============================================== */

/* 事件边框和视觉效果 */
.fc-event,
.fc-timegrid-event {
  border-color: #ddd !important; /* 设置事件边框为灰色 */
  box-shadow: none !important; /* 移除默认阴影效果 */
}

/* 全天事件内边距 */
.fc-daygrid-event {
  padding: 2px 6px !important; /* 上下2px，左右6px */
  margin: 1px 4px !important; /* 外边距，让事件之间有间隔 */
}

.fc-timegrid-axis-cushion {
  display: none !important;
}

/* 全天事件标题容器 */

.fc-daygrid-day-events {
  padding: 0 !important;
  min-height: 2px !important;
  margin-bottom: 2rem !important;

  /* display: none !important; */
}

/* 全天事件标题文字 */
.fc-daygrid-event .fc-event-title {
  padding: 1px 0 !important; /* 微调文字内边距 */
  line-height: 1.4 !important; /* 调整行高，让文字更舒适 */
}

/* ===============================================
 * 7. 日期显示栏样式
 * =============================================== */

.date-display {
  display: flex;
  align-items: center;
  justify-content: center;
}

.date-text {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
  letter-spacing: 0.5px;
}

/* ===============================================
 * 8. 装饰竖线样式
 * =============================================== */

.decorative-line {
  position: fixed; /* 脱离内层 padding 影响，参照 viewport */
  width: 0.8px;
  background: #d1d1d1;
  pointer-events: none;
  z-index: 5;
}

/* ===============================================
 * 9. 日历缩放样式（调整时间槽高度）
 * =============================================== */

/* 1x 缩放（默认） - 保持 FullCalendar 默认高度 1.5rem */
.calendar-container.zoom-1x .fc .fc-timegrid-slot {
  height: 0.5rem !important; /* 10分钟槽，默认值 */
  min-height: 0.5rem !important;
  max-height: 0.5rem !important;
  line-height: 0.5rem !important;
  font-size: 0 !important;
  padding: 0 !important;
}

/* 同时控制时间标签列，防止其撑高行 */
.calendar-container.zoom-1x .fc .fc-timegrid-slot-label {
  height: 0.6rem !important;
  min-height: 0.6rem !important;
  max-height: 0.6rem !important;
  line-height: 0 !important;
  padding: 0 !important;
}

/* 时间标签文字使用绝对定位，不参与高度计算 */
.calendar-container.zoom-1x .fc .fc-timegrid-slot-label-cushion {
  position: absolute;
  top: 50%;
  transform: translate(-100%, -50%);
  font-size: 1.2rem !important;
  line-height: 1 !important;
  white-space: nowrap;
}

/* 1x 缩放时隐藏半点时间标签 (xx:30) */

.calendar-container.zoom-1x
  .fc
  .fc-timegrid-slot-label[data-time$=':30:00']
  .fc-timegrid-slot-label-cushion {
  display: none !important;
}

/* 1x 缩放时移除半点时间槽的边框 */

.calendar-container.zoom-1x .fc .fc-timegrid-slot-lane[data-time$=':30:00'] {
  border: none !important;
}

/* 2x 缩放 - 每小时约 2倍 */
.calendar-container.zoom-2x .fc .fc-timegrid-slot {
  height: 1.5rem !important; /* 10分钟槽 = 3rem，1小时 = 18rem */
}

/* 3x 缩放 - 每小时约 3倍 */
.calendar-container.zoom-3x .fc .fc-timegrid-slot {
  height: 3rem !important; /* 10分钟槽 = 4.5rem，1小时 = 27rem */
}

/* ===============================================
 * 10. 拖拽悬浮在已有事件上的视觉反馈（简化版：仅显示链子图标）
 * =============================================== */
.fc-event.hover-link-target::after {
  content: '🔗';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 2rem;
  pointer-events: none;
}
</style>

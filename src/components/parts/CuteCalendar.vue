<template>
  <div class="calendar-container" :class="`zoom-${currentZoom}x`">
    <!-- 自定义日期头部 -->
    <div v-if="displayDates.length > 0" class="custom-day-headers">
      <div class="time-axis-placeholder" :style="{ width: timeAxisWidth + 'px' }"></div>
      <div
        v-for="dateInfo in displayDates"
        :key="dateInfo.date"
        class="custom-day-header"
        :data-date="dateInfo.date"
        :class="{ 'is-today': dateInfo.isToday }"
        :style="{ width: dateInfo.width ? dateInfo.width + 'px' : 'auto' }"
      >
        <span class="day-name">{{ dateInfo.dayName }}</span>
        <span class="date-number">{{ dateInfo.dateNumber }}</span>
      </div>
    </div>

    <FullCalendar ref="calendarRef" :options="calendarOptions" />

    <!-- 装饰竖线（已禁用） -->
    <!-- <div
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
    ></div> -->

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
import { computed, ref, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useRegisterStore } from '@/stores/register'
import { useAutoScroll } from '@/composables/calendar/useAutoScroll'
import { useTimePosition } from '@/composables/calendar/useTimePosition'
import { useDecorativeLine } from '@/composables/calendar/useDecorativeLine'
import { useCalendarEvents } from '@/composables/calendar/useCalendarEvents'
import { useCalendarHandlers } from '@/composables/calendar/useCalendarHandlers'
import { useCalendarOptions } from '@/composables/calendar/useCalendarOptions'
import { logger, LogTags } from '@/infra/logging/logger'
import { useCalendarInteractDrag } from '@/composables/calendar/useCalendarInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { interactManager } from '@/infra/drag-interact'
import TimeBlockDetailPanel from './TimeBlockDetailPanel.vue'

const timeBlockStore = useTimeBlockStore()
const registerStore = useRegisterStore()

// ==================== Props ====================
const props = withDefaults(
  defineProps<{
    currentDate?: string // YYYY-MM-DD 格式的日期
    zoom?: 1 | 2 | 3 // 缩放倍率
    viewType?: 'day' | 'week' | 'month' // ✅ 新增：视图类型（单天、周或月视图）
    days?: 1 | 3 | 5 | 7 // 🆕 新增：显示天数（1天、3天、5天或7天）
  }>(),
  {
    viewType: 'day', // 默认单天视图
    days: 1, // 默认显示1天
  }
)

// ==================== Events ====================
const emit = defineEmits<{
  'date-change': [date: string] // 日历显示日期变化事件
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
const { getTimeFromDropPosition, clearCache } = useTimePosition(calendarRef)

// 装饰线
const decorativeLine = useDecorativeLine(calendarRef, currentDateRef)
decorativeLine.initialize()

// 拖拽功能（新的 interact.js 系统）
const drag = useCalendarInteractDrag(calendarRef, {
  getTimeFromDropPosition,
  handleAutoScroll,
  stopAutoScroll,
})
const dragStrategy = useDragStrategy()

// 日历事件数据（传入视图类型）
const viewTypeRef = computed(() => props.viewType)
const { calendarEvents } = useCalendarEvents(drag.previewEvent, viewTypeRef)

// 事件处理器
const handlers = useCalendarHandlers(drag.previewEvent, currentDateRef, selectedTimeBlockId)

// 日历日期变化回调
const handleDatesSet = (dateInfo: { start: Date; end: Date }) => {
  // 🔧 FIX: 使用本地时间而不是 UTC 时间，避免时区偏移
  const date = dateInfo.start
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const dateStr = `${year}-${month}-${day}`

  // ✅ 直接写入寄存器，消除 props drilling
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, dateStr)

  // 更新自定义日期头部
  nextTick(() => {
    updateDisplayDates()
  })

  // 保留事件发射以兼容现有代码（可选）
  emit('date-change', dateStr)
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Calendar date changed and written to register', {
    dateStr,
  })
}

// 日历配置（传递视图类型、天数和日期变化回调）
const { calendarOptions } = useCalendarOptions(
  calendarEvents,
  handlers,
  props.viewType,
  handleDatesSet,
  props.days ?? 1
)

// 装饰线位置（已禁用）
// const decorativeLinePosition = decorativeLine.position
// const decorativeLineTop = decorativeLine.top
// const decorativeLineHeight = decorativeLine.height

// ==================== 自定义日期头部 ====================
interface DateHeaderInfo {
  date: string // YYYY-MM-DD
  dayName: string // Mon, Tue, etc.
  dateNumber: string // 20日
  isToday: boolean
  width?: number // 列宽度（像素）
}

const displayDates = ref<DateHeaderInfo[]>([])
const timeAxisWidth = ref(0) // 时间轴宽度
const headerDropzones = new Map<string, HTMLElement>()

// 同步列宽度：从日历网格获取实际列宽
function syncColumnWidths() {
  if (!calendarRef.value) return

  // 获取时间轴宽度
  const timeAxisEl = document.querySelector('.fc-timegrid-axis') as HTMLElement
  if (timeAxisEl) {
    timeAxisWidth.value = timeAxisEl.offsetWidth
  }

  // 获取日历列元素（使用 data-date 属性精确匹配）
  const dayColumns = document.querySelectorAll('.fc-day[data-date]') as NodeListOf<HTMLElement>
  if (dayColumns.length === 0) return

  // 更新每个日期的宽度
  displayDates.value = displayDates.value.map((dateInfo, index) => {
    const columnEl = dayColumns[index]
    if (columnEl) {
      return {
        ...dateInfo,
        width: columnEl.offsetWidth,
      }
    }
    return dateInfo
  })

  logger.debug(LogTags.COMPONENT_CALENDAR, 'Column widths synced', {
    timeAxisWidth: timeAxisWidth.value,
    columnCount: displayDates.value.length,
    widths: displayDates.value.map((d) => d.width),
  })
}

// 更新显示的日期列表
function updateDisplayDates() {
  if (!calendarRef.value) {
    displayDates.value = []
    return
  }

  const calendarApi = calendarRef.value.getApi()
  if (!calendarApi) {
    displayDates.value = []
    return
  }

  const view = calendarApi.view
  const start = view.activeStart
  const end = view.activeEnd

  // 使用本地时间获取今天的日期
  const now = new Date()
  const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`

  const dates: DateHeaderInfo[] = []
  const current = new Date(start)

  // 根据视图类型决定显示哪些日期
  while (current < end) {
    // 使用本地时间获取日期字符串
    const year = current.getFullYear()
    const month = String(current.getMonth() + 1).padStart(2, '0')
    const day = String(current.getDate()).padStart(2, '0')
    const dateStr = `${year}-${month}-${day}`

    // 对于周视图和所有天数视图（包括1天），显示所有日期
    if (props.viewType === 'week' || props.viewType === 'day') {
      const dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
      const dayName = dayNames[current.getDay()] ?? 'Sun'
      const month = current.getMonth() + 1
      const day = current.getDate()

      dates.push({
        date: dateStr,
        dayName,
        dateNumber: `${month}/${day}`,
        isToday: dateStr === today,
      })
    }

    current.setDate(current.getDate() + 1)
  }

  displayDates.value = dates

  // 在下一帧同步列宽度并注册头部拖放区域
  nextTick(() => {
    syncColumnWidths()
    registerHeaderDropzones()
  })

  logger.debug(LogTags.COMPONENT_CALENDAR, 'Display dates updated', { count: dates.length })
}

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

// ==================== 视图类型切换功能 ====================
// 获取视图名称的辅助函数
function getViewName(viewType: 'day' | 'week' | 'month', days: 1 | 3 | 5 | 7): string {
  if (viewType === 'day') {
    if (days === 3) return 'timeGrid3Days'
    if (days === 5) return 'timeGrid5Days'
    if (days === 7) return 'timeGrid7Days'
    return 'timeGridDay'
  } else if (viewType === 'week') {
    return 'timeGridWeek'
  } else {
    return 'dayGridMonth'
  }
}

// 监听 viewType 和 days prop 变化，动态切换视图
watch(
  [() => props.viewType, () => props.days],
  async ([newViewType, newDays]) => {
    if (!calendarRef.value) return

    const calendarApi = calendarRef.value.getApi()
    if (!calendarApi) return

    const viewName = getViewName(newViewType, newDays ?? 1)

    logger.info(LogTags.COMPONENT_CALENDAR, 'Changing calendar view', {
      from: calendarApi.view.type,
      to: viewName,
      viewType: newViewType,
      days: newDays,
    })

    // 保存当前日期
    const currentDate = calendarApi.getDate()

    // 切换视图
    calendarApi.changeView(viewName)

    // 等待 DOM 更新
    await nextTick()

    // 强制更新尺寸
    calendarApi.updateSize()

    // 恢复到之前的日期
    calendarApi.gotoDate(currentDate)

    // 清除缓存，强制重新计算位置
    clearCache()

    // 更新自定义日期头部
    updateDisplayDates()

    logger.debug(LogTags.COMPONENT_CALENDAR, 'Calendar view changed successfully', {
      viewName,
      viewType: newViewType,
      days: newDays,
    })
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
    // decorativeLine.updatePosition() // 已禁用
  }
)

onMounted(async () => {
  // 使用 nextTick 确保DOM完全渲染后再获取数据
  await nextTick()

  // 🔥 注册日历为 dropzone（新系统）
  drag.registerCalendarDropzone()

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

    // 计算装饰竖线位置（已禁用）
    await nextTick()
    // decorativeLine.updatePosition()

    // 🔥 初始化后强制更新尺寸，确保显示正确
    if (calendarRef.value) {
      const calendarApi = calendarRef.value.getApi()
      if (calendarApi) {
        // 多次更新确保尺寸正确
        calendarApi.updateSize()
        await nextTick()
        calendarApi.updateSize()

        logger.debug(LogTags.COMPONENT_CALENDAR, 'Initial calendar size updated', {
          viewType: props.viewType,
          days: props.days,
        })
      }
    }
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'Failed to fetch initial time blocks',
      error instanceof Error ? error : new Error(String(error))
    )
  }
})

onBeforeUnmount(() => {
  headerDropzones.forEach((el) => interactManager.unregisterDropzone(el))
  headerDropzones.clear()
})

// ==================== 日期头部拖放处理 ====================
function registerHeaderDropzones() {
  // 清理旧的dropzones
  headerDropzones.forEach((el) => {
    interactManager.unregisterDropzone(el)
  })
  headerDropzones.clear()

  const headerEls = document.querySelectorAll(
    '.custom-day-headers .custom-day-header'
  ) as NodeListOf<HTMLElement>

  headerEls.forEach((el) => {
    const date = el.dataset.date
    if (!date) return

    const zoneId = `calendar-header-${date}`
    el.setAttribute('data-zone-id', zoneId)

    interactManager.registerDropzone(el, {
      zoneId,
      type: 'calendar',
      onDrop: async (session) => {
        try {
          logger.info(LogTags.COMPONENT_CALENDAR, 'Drop task on calendar header', {
            taskId: session.object.data.id,
            targetDate: date,
          })

          // 构造日期视图的viewKey
          const viewKey = `daily::${date}`

          // 执行拖放策略，排序放在最前面
          const result = await dragStrategy.executeDrop(session, viewKey, {
            sourceContext: session.metadata?.sourceContext || {},
            targetContext: {
              taskIds: [], // 空列表表示放在最前面
              displayTasks: [],
            },
          })

          if (!result.success) {
            logger.error(
              LogTags.COMPONENT_CALENDAR,
              'Failed to drop task on calendar header',
              new Error(result.error || 'Unknown error')
            )
          }
        } catch (error) {
          logger.error(
            LogTags.COMPONENT_CALENDAR,
            'Error handling calendar header drop',
            error instanceof Error ? error : new Error(String(error))
          )
        }
      },
    })

    headerDropzones.set(zoneId, el)
  })
}

// ==================== 暴露给父组件 ====================
defineExpose({
  calendarRef, // 暴露 calendarRef，让父组件可以调用 FullCalendar API
  syncColumnWidths, // 暴露同步列宽方法，用于实时更新
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
  overflow: hidden;
  padding: 0.8rem;
  padding-left: 1.6rem; /* 增加左侧 padding，避免时间标签被截断 */
}

/* 允许时间标签溢出到左侧 */
.calendar-container :deep(.fc),
.calendar-container :deep(.fc-view-harness),
.calendar-container :deep(.fc-timegrid) {
  overflow: visible !important;
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

/* 时间标签字号和字重 */
.fc .fc-timegrid-slot-label-cushion {
  font-size: 1.3rem !important;
  font-weight: 500 !important;
  padding-right: 0.8rem !important; /* 增加右侧间距，避免被截断 */
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
 * 7. 装饰竖线样式
 * =============================================== */

.decorative-line {
  position: fixed; /* 脱离内层 padding 影响，参照 viewport */
  width: 0.8px;
  background: #d1d1d1;
  pointer-events: none;
  z-index: 5;
}

/* ===============================================
 * 8. 日历缩放样式（调整时间槽高度）
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
  transform: translate(calc(-100% - 0.4rem), -50%); /* 往左移动 0.4rem */
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
 * 9. 拖拽悬浮在已有事件上的视觉反馈（简化版：仅显示链子图标）
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

/* ===============================================
 * 10. 周视图样式优化
 * =============================================== */

/* 周视图日期头部样式 */
.fc .fc-col-header-cell {
  padding: 0.5rem;
  font-weight: 600;
  color: var(--color-text-primary);
  background-color: var(--color-background);
  border-bottom: 2px solid var(--color-border-default);
}

/* 今天的列头部高亮 */
.fc .fc-col-header-cell.fc-day-today {
  background-color: var(--color-primary-bg, #e3f2fd);
  color: var(--color-primary, #4a90e2);
}

/* 周视图列之间的分隔线 */
.fc .fc-timegrid-col {
  border-right: 1px solid var(--color-border-default);
}

/* 周视图今天的列高亮 */
.fc .fc-timegrid-col.fc-day-today {
  background-color: var(--color-background-hover, rgb(74 144 226 / 5%));
}

/* ===============================================
 * 11. 月视图样式优化
 * =============================================== */

/* stylelint-disable selector-class-pattern */

/* ✅ 月视图固定行高：防止事件多的格子撑高整行（仅月视图） */
.fc-dayGridMonth-view .fc-daygrid-body tr {
  height: 120px !important; /* 强制固定行高 */
}

.fc-dayGridMonth-view .fc-daygrid-day-frame {
  height: 120px !important; /* 强制固定格子高度 */
  overflow: hidden; /* 超出部分隐藏，配合 dayMaxEvents 使用 */
}

/* 事件容器固定高度（仅月视图） */
.fc-dayGridMonth-view .fc-daygrid-day-events {
  min-height: auto !important;
  overflow: visible; /* 允许 "+N more" 显示 */
}
/* stylelint-enable selector-class-pattern */

/* 月视图单元格样式 */
.fc .fc-daygrid-day {
  cursor: pointer;
}

.fc .fc-daygrid-day:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 2%));
}

/* 月视图今天高亮 */
.fc .fc-daygrid-day.fc-day-today {
  background-color: var(--color-primary-bg, #e3f2fd);
}

/* 月视图日期数字样式 */
.fc .fc-daygrid-day-number {
  padding: 0.4rem;
  font-size: 1.3rem;
  font-weight: 500;
}

/* 月视图今天的日期数字高亮 */
.fc .fc-day-today .fc-daygrid-day-number {
  color: var(--color-primary, #4a90e2);
  font-weight: 600;
}

/* 月视图事件样式 */
.fc .fc-daygrid-event {
  margin: 1px 2px;
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 1.2rem;
}

/* 月视图 "+N more" 链接样式 */
.fc .fc-daygrid-more-link {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-primary, #4a90e2);
  padding: 2px 4px;
  border-radius: 3px;
  transition: background-color 0.15s ease;
}

.fc .fc-daygrid-more-link:hover {
  background-color: var(--color-primary-bg, #e3f2fd);
  text-decoration: none;
}

/* ===============================================
 * 12. 任务事件样式
 * =============================================== */

/* 任务事件（全日）样式 */
.fc-event.task-event {
  opacity: 0.85;
  border-left: 3px solid currentcolor;
  font-weight: 500;
  cursor: default; /* ✅ 不可拖动，使用默认光标 */
}

.fc-event.task-event:hover {
  opacity: 1;
  transform: scale(1.02);
  transition: all 0.15s ease;
}

/* 月视图中的任务事件 */
.fc-daygrid-event.task-event {
  border-left-width: 3px;
}

/* ===============================================
 * 13. 截止日期事件样式
 * =============================================== */

/* 截止日期事件样式 */
.fc-event.due-date-event {
  opacity: 0.9;
  border: 2px dashed currentcolor;
  border-left-width: 4px;
  border-left-style: solid;
  font-weight: 600;
  cursor: default; /* ✅ 不可拖动，使用默认光标 */
  background-image: repeating-linear-gradient(
    45deg,
    transparent,
    transparent 10px,
    rgb(255 255 255 / 10%) 10px,
    rgb(255 255 255 / 10%) 20px
  );
}

.fc-event.due-date-event:hover {
  opacity: 1;
  transform: scale(1.03);
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
  transition: all 0.15s ease;
}

/* 逾期的截止日期事件（更明显的样式） */
.fc-event.due-date-event.overdue {
  animation: pulse-overdue 2s ease-in-out infinite;
  font-weight: 700;
}

@keyframes pulse-overdue {
  0%,
  100% {
    opacity: 0.9;
  }

  50% {
    opacity: 1;
  }
}

/* 月视图中的截止日期事件 */
.fc-daygrid-event.due-date-event {
  border-left-width: 4px;
}

/* ===============================================
 * 14. 自定义日期头部样式
 * =============================================== */

.custom-day-headers {
  display: flex;
  align-items: center;
  background-color: var(--color-background-content, #fff);
  border-bottom: 1px solid var(--color-border-default, #e0e0e0);
  position: sticky;
  top: 0;
  z-index: 10;
  height: 48px;
}

.time-axis-placeholder {
  flex-shrink: 0;
  border-right: 1px solid var(--color-border-default, #e0e0e0);
}

.custom-day-header {
  flex-shrink: 0; /* 使用固定宽度，不自动伸缩 */
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  padding: 0.6rem 0.4rem;
  border-right: 1px solid var(--color-border-default, #e0e0e0);
  transition: background-color 0.2s ease;
  box-sizing: border-box; /* 确保 padding 不影响宽度 */
  cursor: pointer;
}

.custom-day-header:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 3%));
}

.custom-day-header.is-today {
  background-color: var(--color-primary-bg, rgb(139 92 246 / 10%));
}

.custom-day-header .day-name {
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-secondary, #666);
  text-transform: uppercase;
}

.custom-day-header.is-today .day-name {
  color: var(--color-primary, #8b5cf6);
}

.custom-day-header .date-number {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary, #333);
}

.custom-day-header.is-today .date-number {
  color: var(--color-primary, #8b5cf6);
  font-weight: 700;
}
</style>

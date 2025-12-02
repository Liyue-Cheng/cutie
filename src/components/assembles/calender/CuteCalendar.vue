<template>
  <div class="calendar-container" :class="[`zoom-${currentZoom}x`, viewTypeClass]">
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
      :panel-position="detailPanelPosition"
      @close="handleDetailPanelClose"
    />
  </div>
</template>

<script setup lang="ts">
import FullCalendar from '@fullcalendar/vue3'
import type { DatesSetArg, EventClickArg } from '@fullcalendar/core'
import { computed, ref, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useTaskStore } from '@/stores/task'
import { useRegisterStore } from '@/stores/register'
import { useUserSettingsStore } from '@/stores/user-settings'
import { useAutoScroll } from '@/composables/calendar/useAutoScroll'
import { useTimePosition } from '@/composables/calendar/useTimePosition'
import { useDecorativeLine } from '@/composables/calendar/useDecorativeLine'
import { useCalendarEvents } from '@/composables/calendar/useCalendarEvents'
import { useCalendarHandlers } from '@/composables/calendar/useCalendarHandlers'
import { useCalendarOptions } from '@/composables/calendar/useCalendarOptions'
import { logger, LogTags } from '@/infra/logging/logger'
import { useCalendarInteractDrag } from '@/composables/calendar/useCalendarInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { interactManager, dragPreviewState, previewMousePosition } from '@/infra/drag-interact'
import TimeBlockDetailPanel from '@/components/organisms/TimeBlockDetailPanel.vue'

const timeBlockStore = useTimeBlockStore()
const taskStore = useTaskStore()
const registerStore = useRegisterStore()
const userSettingsStore = useUserSettingsStore()

// ==================== Props ====================
const props = withDefaults(
  defineProps<{
    currentDate?: string // YYYY-MM-DD 格式的日期
    zoom?: 1 | 2 | 3 // 缩放倍率
    viewType?: 'day' | 'week' | 'month' // ✅ 新增：视图类型（单天、周或月视图）
    days?: 1 | 3 | 5 | 7 // 🆕 新增：显示天数（1天、3天、5天或7天）
    monthViewFilters?: {
      showRecurringTasks: boolean
      showScheduledTasks: boolean
      showDueDates: boolean
      showAllDayEvents: boolean
    }
  }>(),
  {
    viewType: 'day', // 默认单天视图
    days: 1, // 默认显示1天
    monthViewFilters: () => ({
      showRecurringTasks: true,
      showScheduledTasks: true,
      showDueDates: true,
      showAllDayEvents: true,
    }),
  }
)

// ==================== Events ====================
const emit = defineEmits<{
  'date-change': [date: string] // 日历显示日期变化事件
  'month-date-click': [date: string] // 月视图日期点击事件
}>()

// 默认缩放倍率为 1
const currentZoom = computed(() => props.zoom ?? 1)

// 视图类型 class（用于 CSS 样式区分）
const viewTypeClass = computed(() => `view-type-${props.viewType}`)

// FullCalendar 引用
const calendarRef = ref<InstanceType<typeof FullCalendar> | null>(null)
const currentDateRef = computed(() => props.currentDate)

// 选中的时间块ID（用于显示详情面板）
const selectedTimeBlockId = ref<string | null>(null)

type DetailPanelPosition = {
  top: number
  left: number
}

const detailPanelPosition = ref<DetailPanelPosition | null>(null)
let detailPanelAnchorEl: HTMLElement | null = null
const DETAIL_PANEL_GAP = 12
const DETAIL_PANEL_VIEWPORT_PADDING = 48
let viewportListenersRegistered = false

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max)
}

function resetDetailPanelAnchor() {
  detailPanelAnchorEl = null
  detailPanelPosition.value = null
}

function updateDetailPanelPosition(anchorEl: HTMLElement | null) {
  if (!anchorEl) {
    resetDetailPanelAnchor()
    return
  }

  detailPanelAnchorEl = anchorEl
  const rect = anchorEl.getBoundingClientRect()
  const viewportHeight = typeof window !== 'undefined' ? window.innerHeight : 0
  const top =
    viewportHeight > 0
      ? clamp(
          rect.top,
          DETAIL_PANEL_VIEWPORT_PADDING,
          viewportHeight - DETAIL_PANEL_VIEWPORT_PADDING
        )
      : rect.top
  const left = rect.left - DETAIL_PANEL_GAP

  detailPanelPosition.value = {
    top,
    left,
  }
}

function handleViewportChange() {
  if (!detailPanelAnchorEl || !selectedTimeBlockId.value) {
    return
  }

  if (typeof document !== 'undefined' && !document.body.contains(detailPanelAnchorEl)) {
    resetDetailPanelAnchor()
    return
  }

  updateDetailPanelPosition(detailPanelAnchorEl)
}

function handleDetailPanelClose() {
  selectedTimeBlockId.value = null
  resetDetailPanelAnchor()
}

watch(selectedTimeBlockId, (newValue) => {
  if (!newValue) {
    resetDetailPanelAnchor()
  }
})

onMounted(() => {
  if (typeof window === 'undefined') {
    return
  }
  window.addEventListener('resize', handleViewportChange)
  window.addEventListener('scroll', handleViewportChange, true)
  viewportListenersRegistered = true
})

onBeforeUnmount(() => {
  if (typeof window === 'undefined' || !viewportListenersRegistered) {
    return
  }
  window.removeEventListener('resize', handleViewportChange)
  window.removeEventListener('scroll', handleViewportChange, true)
  viewportListenersRegistered = false
})

watch(
  () => userSettingsStore.theme,
  () => {
    nextTick(() => {
      const api = calendarRef.value?.getApi()
      if (!api) {
        return
      }
      api.render()
      clearCache()
      updateDisplayDates()
      syncColumnWidths()
    })
  }
)

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

// 日历事件数据（传入视图类型和筛选器）
const viewTypeRef = computed(() => props.viewType)
const monthViewFiltersRef = computed(() => props.monthViewFilters)
const { calendarEvents } = useCalendarEvents(drag.previewEvent, viewTypeRef, monthViewFiltersRef)

// 月视图日期点击回调
function handleMonthDateClick(date: string) {
  emit('month-date-click', date)
}

// 日期头部点击（周视图/多日视图）
function onDayHeaderClick(date: string) {
  emit('month-date-click', date)
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Day header clicked', { date })
}

// 事件处理器
const handlers = useCalendarHandlers(
  drag.previewEvent,
  currentDateRef,
  selectedTimeBlockId,
  handleMonthDateClick
)

function handleCalendarEventClick(clickInfo: EventClickArg) {
  handlers.handleEventClick(clickInfo)

  const extended = clickInfo.event.extendedProps as {
    type?: string
  }

  if (extended?.type === 'timeblock') {
    updateDetailPanelPosition(clickInfo.el as HTMLElement | null)
  } else {
    resetDetailPanelAnchor()
  }
}

const calendarHandlers = {
  ...handlers,
  handleEventClick: handleCalendarEventClick,
}

function formatDateShort(d: Date) {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

// 🔥 拉取月视图数据的辅助函数
const fetchMonthViewData = async () => {
  if (props.viewType !== 'month' || !calendarRef.value) {
    return
  }

  const calendarApi = calendarRef.value.getApi()
  const view = calendarApi.view
  const startDate = view.activeStart
  const endDate = view.activeEnd

  const startDateStr = formatDateShort(startDate)
  const endDateStr = formatDateShort(new Date(endDate.getTime() - 1)) // 结束日期为独，占用前一天

  logger.info(LogTags.COMPONENT_CALENDAR, 'Fetching data for month view', {
    startDate: startDateStr,
    endDate: endDateStr,
  })

  const fetchStartTs = typeof performance !== 'undefined' ? performance.now() : Date.now()
  let totalFetchDays = 0

  try {
    // 拉取该月份的时间块数据（后端会自动生成循环任务）
    await timeBlockStore.fetchTimeBlocksForRange(startDateStr, endDateStr)
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'Failed to fetch time blocks for month view',
      error instanceof Error ? error : new Error(String(error)),
      { startDate: startDateStr, endDate: endDateStr }
    )
  }

  // 🔄 同步加载日期范围任务，确保循环任务实例生成
  try {
    const datesToFetch: string[] = []
    const cursor = new Date(startDate)
    const exclusiveEnd = new Date(endDate)

    while (cursor < exclusiveEnd) {
      datesToFetch.push(formatDateShort(cursor))
      cursor.setDate(cursor.getDate() + 1)
    }

    totalFetchDays = datesToFetch.length

    if (datesToFetch.length > 0) {
      const rangeStart = datesToFetch[0]!
      const rangeEnd = datesToFetch[datesToFetch.length - 1]!
      await taskStore.fetchDailyTasksRange_DMA(rangeStart, rangeEnd)
    }
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'Failed to load calendar tasks for month view',
      error instanceof Error ? error : new Error(String(error)),
      { startDate: startDateStr, endDate: endDateStr }
    )
  } finally {
    const fetchEndTs = typeof performance !== 'undefined' ? performance.now() : Date.now()
    const durationMs = Math.round(fetchEndTs - fetchStartTs)
    logger.info(LogTags.COMPONENT_CALENDAR, 'Month view data fetch completed', {
      startDate: startDateStr,
      endDate: endDateStr,
      durationMs,
      totalDays: totalFetchDays,
    })
  }
}

// 日历日期变化回调
const handleDatesSet = (dateInfo: DatesSetArg) => {
  const calendarApi = calendarRef.value?.getApi()
  const activeDate =
    calendarApi?.getDate() ??
    (dateInfo.view?.currentStart ? new Date(dateInfo.view.currentStart.valueOf()) : dateInfo.start)

  // 🔧 FIX: 使用本地时间而不是 UTC 时间，避免时区偏移
  const date = activeDate
  const dateStr = formatDateShort(date)

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
  calendarHandlers,
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

  // 获取时间轴宽度（使用浮点宽度，避免整数舍入误差）
  const timeAxisEl = document.querySelector('.fc-timegrid-axis') as HTMLElement
  if (timeAxisEl) {
    const rect = timeAxisEl.getBoundingClientRect()
    timeAxisWidth.value = rect.width
  }

  // 获取日历列元素（使用 data-date 属性精确匹配）
  const dayColumns = document.querySelectorAll('.fc-day[data-date]') as NodeListOf<HTMLElement>
  if (dayColumns.length === 0) return

  // 更新每个日期的宽度
  displayDates.value = displayDates.value.map((dateInfo, index) => {
    const columnEl = dayColumns[index]
    if (columnEl) {
      const rect = columnEl.getBoundingClientRect()
      return {
        ...dateInfo,
        // 使用浮点宽度而不是 offsetWidth，避免 0.x / 1.x 像素误差
        width: rect.width,
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
  async (newDate, oldDate) => {
    // 🔍 检查点3：日历日期同步
    logger.debug(LogTags.COMPONENT_CALENDAR, 'Date changed', { oldDate, newDate })

    if (newDate && calendarRef.value) {
      const calendarApi = calendarRef.value.getApi()
      if (calendarApi) {
        logger.info(LogTags.COMPONENT_CALENDAR, 'Switching to date', { newDate })
        calendarApi.gotoDate(newDate)

        // 🔧 FIX: 清除缓存，强制重新计算位置
        clearCache()

        // 🔥 月视图：日期变化时拉取新月份的数据
        if (props.viewType === 'month') {
          await nextTick() // 确保日期已切换
          await fetchMonthViewData()
        }

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

    // 🔥 如果切换到月视图，拉取该月份的数据
    if (newViewType === 'month') {
      await nextTick() // 确保视图已切换
      await fetchMonthViewData()
    }

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

// 窗口resize处理函数
let resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  // 使用 nextTick 确保DOM完全渲染后再获取数据
  await nextTick()

  // 🔥 注册日历为 dropzone（新系统）
  drag.registerCalendarDropzone()

  // 🔥 监听窗口大小变化，同步列宽和更新日历尺寸
  resizeObserver = new ResizeObserver(() => {
    if (calendarRef.value) {
      const calendarApi = calendarRef.value.getApi()
      if (calendarApi) {
        // 更新日历尺寸
        calendarApi.updateSize()
        // 延迟同步列宽，等待DOM更新
        nextTick(() => {
          syncColumnWidths()
        })
      }
    }
  })

  // 观察日历容器的大小变化
  const calendarContainer = document.querySelector('.calendar-container')
  if (calendarContainer) {
    resizeObserver.observe(calendarContainer)
  }

  try {
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

    // 🔥 月视图：拉取当前月份的数据
    if (props.viewType === 'month') {
      await nextTick() // 确保日历已渲染
      await fetchMonthViewData()
    } else {
      // 其他视图：加载更大的时间范围（前后各 3 个月）
      const today = new Date()
      const startDate = new Date(today.getFullYear(), today.getMonth() - 3, 1) // 3个月前
      const endDate = new Date(today.getFullYear(), today.getMonth() + 4, 0) // 3个月后

      // 🔥 使用本地日期格式（YYYY-MM-DD），符合 TIME_CONVENTION.md 规范
      const startDateStr = formatDateShort(startDate)
      const endDateStr = formatDateShort(endDate)

      logger.debug(LogTags.COMPONENT_CALENDAR, 'Loading time blocks for range', {
        startDate: startDateStr,
        endDate: endDateStr,
      })
      await timeBlockStore.fetchTimeBlocksForRange(startDateStr, endDateStr)
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
  // 清理resize observer
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }

  // 清理header dropzones
  headerDropzones.forEach((el) => interactManager.unregisterDropzone(el))
  headerDropzones.clear()
})

// ==================== 日期头部拖放处理 ====================
// 检测是否拖动到指定日期
function isDragTargetDate(date: string): boolean {
  const preview = dragPreviewState.value
  if (!preview) return false

  const targetZoneId = preview.raw.targetZoneId
  if (!targetZoneId || targetZoneId !== `daily::${date}`) {
    return false
  }

  const mousePosition = previewMousePosition.value
  const headerEl = headerDropzones.get(targetZoneId)
  if (!mousePosition || !headerEl) {
    return false
  }

  const rect = headerEl.getBoundingClientRect()
  const { x, y } = mousePosition

  return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom
}

function registerHeaderDropzones() {
  // 清理旧的dropzones
  headerDropzones.forEach((el) => {
    interactManager.unregisterDropzone(el)
  })
  headerDropzones.clear()

  // 使用 FullCalendar 原生列头作为拖放目标，保证与网格列像素级对齐
  const headerEls = document.querySelectorAll(
    '.calendar-container .fc-col-header-cell[data-date]'
  ) as NodeListOf<HTMLElement>

  headerEls.forEach((el) => {
    const date = el.dataset.date
    if (!date) return

    // 使用daily::date作为zoneId，这样预览系统可以统一识别
    const zoneId = `daily::${date}`
    el.setAttribute('data-zone-id', zoneId)

    // 绑定点击事件：点击头部日期时触发与之前相同的逻辑
    el.onclick = () => {
      onDayHeaderClick(date)
    }

    interactManager.registerDropzone(el, {
      zoneId,
      type: 'kanban', // 看板型目标：拖到头部表示放到该日期最上方
      computePreview: () => ({
        dropIndex: 0, // 总是放在最上面
      }),
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
 * FullCalendar 自定义样式 - Cutie日历组件
 * ===============================================
 *
 * 🎯 功能概述：
 * 本文件为FullCalendar组件提供完整的样式重写，实现：
 * - 与Cutie设计系统的完全集成
 * - 支持1x/2x/3x三种缩放级别
 * - 任务、时间块、截止日期三种事件类型的自定义渲染
 * - 响应式布局和主题切换支持
 *
 * 🏗️ 架构说明：
 * - 使用FullCalendar CSS变量统一主题控制
 * - 按功能模块分组，每个模块有明确的职责边界
 * - 利用CSS自定义属性实现动态配置
 * - 遵循BEM命名约定和语义化类名
 *
 * 📋 样式模块索引：
 * 0. 容器配置与FullCalendar变量设置
 * 1. 核心布局与溢出控制
 * 2. 边框统一管理
 * 3. 时间轴与标签系统
 * 4. 缩放系统(1x/2x/3x)
 * 5. 事件样式统一(task/timeblock/due-date)
 * 6. 视图特定样式(week/month/day)
 * 7. 交互反馈与状态管理
 * 8. 自定义组件集成
 */

/* ===============================================
 * 0. 日历容器配置与FullCalendar变量设置
 * =============================================== */

.calendar-container {
  /* 🎛️ 容器布局配置 */
  height: 100%;
  position: relative;
  overflow: hidden;
  padding: 0.8rem;
  padding-left: 1.6rem; /* 🔧 为时间标签预留溢出空间 */

  /* 🎨 FullCalendar主题变量映射 - 统一使用Cutie设计token */
  --fc-border-color: var(--color-border-default); /* 📐 统一边框颜色 */
  --fc-today-bg-color: transparent; /* 📅 今日背景透明，无染色 */
  --fc-now-indicator-color: var(--color-danger); /* ⏰ 当前时间指示器 */
  --fc-neutral-text-color: var(--color-text-secondary); /* 📝 次要文本颜色 */
  --fc-small-font-size: 1.1rem; /* 📏 小字体尺寸 */
  --fc-event-selected-overlay-color: transparent; /* ❌ 禁用事件选中覆盖 */
  --fc-highlight-color: transparent; /* ❌ 禁用原生选区高亮，使用 selectMirror 自定义渲染 */

  /* 🔧 自定义缩放变量 - 支持动态时间槽高度调节 */
  --zoom-slot-height-1x: 0.75rem; /* 紧凑视图：10分钟=0.75rem, 1小时=4.5rem */
  --zoom-slot-height-2x: 1.5rem; /* 标准视图：10分钟=1.5rem, 1小时=9rem */
  --zoom-slot-height-3x: 3rem; /* 详细视图：10分钟=3rem, 1小时=18rem */
}

/* ===============================================
 * 1. 核心布局与溢出控制
 * =============================================== */

/* 🌊 允许时间标签向左溢出 - 避免标签被容器边界裁切 */
.calendar-container :deep(.fc),
.calendar-container :deep(.fc-view-harness),
.calendar-container :deep(.fc-timegrid) {
  overflow: visible !important; /* 🔓 解除FullCalendar默认的overflow:hidden限制 */
}

/* ⏰ 当前时间指示器配置 - 使用FullCalendar内置功能 */
.fc-timegrid-now-indicator-line {
  border-color: var(--fc-now-indicator-color) !important; /* 🎨 使用统一的危险色 */
  border-width: 2px !important; /* 📏 增加线条粗细提升可见性 */
  z-index: 10 !important; /* 🔝 确保在所有事件之上 */
}

.fc-timegrid-now-indicator-arrow {
  display: none !important; /* ❌ 隐藏默认箭头，保持简洁 */
}

/* 📅 今日背景控制 - 保持透明，无染色 */
.fc .fc-day-today {
  background-color: transparent !important; /* ❌ 移除今日默认背景染色 */
}

/* ===============================================
 * 2. 边框精细管理 - 选择性移除FullCalendar默认边框
 * =============================================== */

/* 🗂️ 移除主网格外边框 */
.fc-theme-standard .fc-scrollgrid {
  border: none !important; /* ❌ 移除最外层网格边框 */
}

/* 📊 移除表格单元格右边框 - 保留其他边框 */
.fc-theme-standard td,
.fc-theme-standard th {
  border-right: none !important; /* ❌ 仅移除右边框，保留上下边框 */
}

/* 🌊 移除液体布局容器边框 */
.fc .fc-scrollgrid-section-liquid > td {
  border: none !important; /* ❌ 移除液体布局单元格边框 */
}

/* ⏰ 移除时间标签和次要时间槽边框 */
.fc .fc-timegrid-slot-label,
.fc .fc-timegrid-slot-minor {
  border: none !important; /* ❌ 移除时间相关元素边框 */
}

/* 🛤️ 时间网格车道边框控制 - 默认移除，特定情况保留 */
.calendar-container .fc .fc-timegrid-slot-lane {
  border: none !important; /* ❌ 默认移除所有时间槽边框 */
}

/* 🎯 保留整点时间横线 - 提供时间分隔视觉提示 */
.calendar-container .fc .fc-timegrid-slot-lane[data-time$=':00:00'] {
  border-top: 1px solid var(--fc-border-color) !important; /* ✅ 整点横线使用统一边框色 */
}

/* 🔧 时间网格分隔线配置 */
.fc .fc-timegrid-divider {
  padding: 0 !important; /* ❌ 移除内边距 */
  border-bottom: none !important; /* ❌ 移除底边框 */
  background-color: transparent !important; /* 🎭 透明背景 */
}

/* ===============================================
 * 3. 时间轴与标签系统
 * =============================================== */

/* ⏰ 时间标签垂直对齐 */
.fc .fc-timegrid-slot-label {
  transform: translateY(-50%); /* 🎯 精确居中对齐 */
}

/* 📝 时间标签文字样式 */
.fc .fc-timegrid-slot-label-cushion {
  font-size: 1.3rem !important; /* 📏 适中的字体大小 */
  font-weight: 500 !important; /* 📝 中等字重，保持清晰 */
  color: var(--fc-neutral-text-color) !important; /* 🎨 使用FullCalendar变量 */
  padding-right: 0.8rem !important; /* 📐 右侧留白避免截断 */
}

/* 🏗️ 时间槽容器配置 */
.fc .fc-timegrid-slots {
  padding-top: 1rem !important; /* 🔝 顶部留白防止translateY裁切 */
}

/* ❌ 隐藏时间轴装饰元素 */
.fc-timegrid-axis-cushion {
  display: none !important; /* 🎭 移除不需要的时间轴装饰 */
}

/* ===============================================
 * 4. 滚动条美化 - WebKit浏览器样式定制
 * =============================================== */

/* 📏 滚动条尺寸控制 */
.fc .fc-scroller::-webkit-scrollbar {
  width: 8px; /* 📐 细滚动条，不占用过多空间 */
  background-color: transparent; /* 🎭 透明背景 */
}

/* 🛤️ 滚动条轨道 */
.fc .fc-scroller::-webkit-scrollbar-track {
  background-color: transparent; /* 🎭 透明轨道 */
}

/* 🎛️ 滚动条滑块 */
.fc .fc-scroller::-webkit-scrollbar-thumb {
  background-color: var(--color-border-default); /* 🎨 使用默认边框色 */
  border-radius: 4px; /* ⭕ 圆角设计 */
  transition: background-color 0.2s; /* 🎬 平滑颜色过渡 */
}

/* 🖱️ 滑块悬停效果 */
.fc .fc-scroller::-webkit-scrollbar-thumb:hover {
  background-color: var(--color-border-strong); /* 🎨 悬停时加深颜色 */
}

/* ===============================================
 * 5. 事件基础样式统一
 * =============================================== */

/* 🎭 统一移除事件阴影效果 */
.fc-event,
.fc-timegrid-event {
  box-shadow: none !important; /* ❌ 移除FullCalendar默认阴影 */
}

/* 📝 事件文本统一样式 */
.fc-event .fc-event-title,
.fc-event .fc-event-time,
.fc-timegrid-event .fc-event-title,
.fc-timegrid-event .fc-event-time {
  color: var(--color-text-primary, #575279) !important; /* 🎨 统一主要文本色 */
  font-weight: 600 !important; /* 📝 加粗提升可读性 */
}

/* 📦 全天事件布局控制 */
.fc-daygrid-event {
  padding: 2px 6px !important; /* 📐 上下2px，左右6px内边距 */
  margin: 1px 4px !important; /* 📏 事件间距分离 */
}

/* 📄 全天事件文本精细调节 */
.fc-daygrid-event .fc-event-title {
  padding: 1px 0 !important; /* 📐 文字内边距微调 */
  line-height: 1.4 !important; /* 📏 行高优化可读性 */
}

/* 📋 全天事件容器配置 */
.fc-daygrid-day-events {
  padding: 0 !important; /* ❌ 移除默认内边距 */
  min-height: 2px !important; /* 📏 最小高度保证 */
  margin-bottom: 2rem !important; /* 🔻 底部留白 */
}

/* ===============================================
 * 6. 缩放系统 - 动态时间槽高度控制(1x/2x/3x)
 * =============================================== */

/* 🔍 1x缩放(紧凑视图) - 10分钟槽高度优化 */
.calendar-container.zoom-1x .fc .fc-timegrid-slot {
  height: var(--zoom-slot-height-1x) !important; /* ✅ 使用统一变量 */
  min-height: var(--zoom-slot-height-1x) !important;
  max-height: var(--zoom-slot-height-1x) !important;
  line-height: var(--zoom-slot-height-1x) !important;
  font-size: 0 !important; /* ❌ 隐藏槽内文本 */
  padding: 0 !important; /* ❌ 移除内边距 */
}

/* ⏰ 1x缩放时间标签列高度控制 */
.calendar-container.zoom-1x .fc .fc-timegrid-slot-label {
  height: 0.6rem !important; /* 📏 比时间槽略小，防止撑高 */
  min-height: 0.6rem !important;
  max-height: 0.6rem !important;
  line-height: 0 !important;
  padding: 0 !important;
}

/* 📍 1x缩放时间标签文字绝对定位 */
.calendar-container.zoom-1x .fc .fc-timegrid-slot-label-cushion {
  position: absolute; /* 🎯 脱离文档流，不影响高度计算 */
  top: 50%;
  transform: translate(calc(-100% - 0.4rem), -50%); /* ⬅️ 向左偏移0.4rem */
  line-height: 1 !important; /* 📏 正常行高 */
  white-space: nowrap; /* 🚫 防止文字换行 */
}

/* ⏰ 1x缩放隐藏半点时间标签(:30) - 减少视觉干扰 */
.calendar-container.zoom-1x
  .fc
  .fc-timegrid-slot-label[data-time$=':30:00']
  .fc-timegrid-slot-label-cushion {
  display: none !important; /* ❌ 仅显示整点时间 */
}

/* 🔍 2x缩放(标准视图) */
.calendar-container.zoom-2x .fc .fc-timegrid-slot {
  height: var(--zoom-slot-height-2x) !important; /* ✅ 10分钟=1.5rem */
}

/* 🔍 3x缩放(详细视图) */
.calendar-container.zoom-3x .fc .fc-timegrid-slot {
  height: var(--zoom-slot-height-3x) !important; /* ✅ 10分钟=3rem */
}

/* ===============================================
 * 7. 特殊事件样式 - 预览/创建中/链接反馈
 * =============================================== */

/* 👻 预览事件 - 透明样式，不干扰用户操作 */
.fc-event.preview-event {
  background: transparent !important; /* 🎭 完全透明背景 */
  border: none !important; /* ❌ 无边框 */
  color: inherit !important; /* 🎨 继承父元素颜色 */
  pointer-events: none !important; /* 🖱️ 允许点击穿透到下方事件 */
}

/* ⚡ 创建中事件 - 脉冲动画提供视觉反馈 */
.fc-event.creating-event {
  background-color: var(--color-background-accent-light) !important; /* 🎨 浅色强调背景 */
  color: var(--color-text-primary, #575279) !important; /* 📝 主要文本色 */
  border-color: var(--color-info) !important; /* 🔷 信息色边框 */
  opacity: 0.8; /* 👻 轻微透明 */
  animation: pulse 1s infinite; /* 🎬 无限脉冲动画 */
}

/* 🎬 脉冲动画定义 - 创建中事件的呼吸效果 */
@keyframes pulse {
  0%,
  100% {
    opacity: 0.8; /* 📉 起始和结束透明度 */
  }

  50% {
    opacity: 1; /* 📈 中间点完全不透明 */
  }
}

/* 🔗 拖拽链接目标指示器 - 悬浮链子图标反馈 */
.fc-event.hover-link-target::after {
  content: '🔗'; /* 🔗 链子emoji图标 */
  position: absolute; /* 📍 绝对定位覆盖 */
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%); /* 🎯 精确居中 */
  font-size: 2rem; /* 📏 大尺寸突出显示 */
  pointer-events: none; /* 🖱️ 不阻挡鼠标事件 */
}

/* 📱 月视图任务事件样式调整 */
.calendar-container :deep(.fc-daygrid-event.task-event) {
  padding: 0.2rem 0.4rem; /* 📐 月视图专用内边距 */
}

.calendar-container :deep(.fc-daygrid-event.task-event .fc-event-main) {
  padding: 0; /* ❌ 移除主要内容内边距 */
}

/* ===============================================
 * 8. 统一事件样式 - 三种事件类型的基础样式合并
 * =============================================== */

/* 🎭 事件基础样式统一 - task/timeblock/due-date共用 */
.fc-event.task-event,
.fc-event.timeblock-allday,
.fc-event.due-date-event {
  background: transparent !important; /* 🎭 透明背景，使用Vue组件渲染 */
  border: none !important; /* ❌ 无边框 */
  font-weight: 500; /* 📝 中等字重 */
  cursor: default; /* 🖱️ 默认鼠标样式 */
  padding-left: 0 !important; /* ❌ 移除左侧内边距 */
}

/* 📝 事件主要内容文字颜色统一 */
.fc-event.task-event .fc-event-main,
.fc-event.timeblock-allday .fc-event-main,
.fc-event.due-date-event .fc-event-main {
  color: var(--color-text-primary, #575279) !important; /* 🎨 统一主要文本色 */
}

/* 🎯 特殊样式差异化处理 */
.fc-event.timeblock-allday {
  cursor: pointer; /* 👆 时间块可点击 */
}

.fc-event.due-date-event {
  font-weight: 600; /* 📝 截止日期使用更粗字重 */
}

/* 🖱️ 悬停效果统一 */
.fc-event.task-event:hover,
.fc-event.timeblock-allday:hover,
.fc-event.due-date-event:hover {
  opacity: 0.7; /* 👻 悬停时轻微透明 */
  transition: opacity 0.15s ease; /* 🎬 平滑过渡效果 */
}

/* ⚠️ 逾期截止日期特殊标记 */
.fc-event.due-date-event.overdue .fc-event-main {
  color: var(--color-danger) !important; /* 🔴 危险色突出逾期状态 */
  font-weight: 700; /* 📝 最粗字重强调 */
}

/* ===============================================
 * 9. TimeGrid事件禁用选中状态 - 完全使用自定义组件控制
 * =============================================== */

/* ❌ 禁用FullCalendar的事件选中状态 */
.fc {
  --fc-event-selected-overlay-color: transparent; /* ✅ 使用FullCalendar变量 */
}

/* 🎭 TimeGrid事件透明化处理 */
.fc-timegrid-event.fc-event:not(.fc-event-mirror, .preview-event) {
  background: transparent !important; /* 🎭 背景透明 */
  border: none !important; /* ❌ 无边框 */
  padding: 0 !important; /* ❌ 无内边距 */
}

.fc-timegrid-event.fc-event:not(.fc-event-mirror, .preview-event) .fc-event-main {
  padding: 0 !important; /* ❌ 主要内容无内边距 */
}

/* 🖱️ 禁用所有事件交互状态样式 */
.fc-event:not(.fc-event-mirror, .preview-event):hover,
.fc-event:not(.fc-event-mirror, .preview-event):active,
.fc-event:not(.fc-event-mirror, .preview-event):focus {
  outline: none !important; /* ❌ 移除轮廓 */
  box-shadow: none !important; /* ❌ 移除阴影 */
}

/* 🎭 TimeGrid事件额外状态重置 */
.fc-timegrid-event.fc-event:not(.fc-event-mirror, .preview-event):hover,
.fc-timegrid-event.fc-event:not(.fc-event-mirror, .preview-event):active,
.fc-timegrid-event.fc-event:not(.fc-event-mirror, .preview-event):focus {
  background: transparent !important; /* 🎭 保持透明背景 */
}

/* ❌ 禁用事件选中状态的所有视觉反馈 */
.fc-event.fc-event-selected,
.fc-timegrid-event.fc-event.fc-event-selected {
  outline: none !important;
  box-shadow: none !important;
  background: transparent !important;
}

/* ❌ 移除选中状态伪元素 */
.fc-event.fc-event-selected::before,
.fc-event.fc-event-selected::after,
.fc-event:focus::before,
.fc-event:focus::after {
  display: none !important;
}

/* 🎭 确保拖拽mirror事件也保持透明 */
.fc-timegrid-event.fc-event-mirror {
  background: transparent !important;
  border: none !important;
}

/* ===============================================
 * 10. 视图特定样式 - 周视图/月视图定制
 * =============================================== */

/* 📅 周视图 / 多日视图 / 月视图日期头部
 * 使用与自定义头部相同的背景色，并让内容在单元格内完全居中
 */
.fc .fc-col-header-cell {
  padding: 0; /* 由内部自定义头部控制内边距，避免垂直偏移 */
  font-weight: 600; /* 📝 加粗字重 */
  color: var(--color-text-primary); /* 🎨 主要文本色 */
  background-color: var(--color-background-content); /* 🎭 与内容区域一致的浅色背景 */
  border-bottom: 1px solid var(--color-border-default); /* 🔲 底部分隔线，与网格对齐 */
  height: 48px; /* 📏 固定高度，与之前自定义头部保持一致 */
}

/* 让同步容器和 cushion 链接撑满单元格高度，方便内部 flex 居中 */
.fc .fc-col-header-cell .fc-scrollgrid-sync-inner {
  height: 100%;
}

.fc .fc-col-header-cell .fc-col-header-cell-cushion {
  display: block;
  height: 100%;
  padding: 0; /* 由 .custom-day-header 控制内部留白 */
  text-decoration: none; /* 🔧 取消默认下划线 */
}

/* 选中 / 悬停列头时也不显示下划线 */
.fc .fc-col-header-cell .fc-col-header-cell-cushion:hover,
.fc .fc-col-header-cell .fc-col-header-cell-cushion:focus,
.fc .fc-col-header-cell .fc-col-header-cell-cushion:active {
  text-decoration: none;
}

/* 🌟 今日列头部高亮 - 仅保留文字颜色，无背景 */
.fc .fc-col-header-cell.fc-day-today {
  background-color: transparent !important; /* ❌ 移除列头背景染色 */
  color: var(--color-calendar-today); /* 🎨 仅保留今日文字色 */
}

/* 📅 周视图今日列背景 - 保持透明 */
.fc .fc-timegrid-col.fc-day-today {
  background-color: transparent !important; /* ❌ 移除时间网格列背景染色 */
}

/* 📱 月视图网格样式 */
.fc .fc-daygrid-day {
  cursor: pointer; /* 👆 可点击单元格 */
}

.fc .fc-daygrid-day:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 2%)); /* 🖱️ 悬停反馈 */
}

/* 📅 月视图今日高亮 - 仅数字徽章，无格子背景 */
.fc .fc-daygrid-day.fc-day-today {
  background-color: transparent !important; /* ❌ 移除月视图格子背景染色 */
}

/* 🎯 月视图今日数字徽章 */
.fc .fc-day-today .fc-daygrid-day-number {
  color: var(--color-text-on-accent); /* 🎨 高对比度文字 */
  background-color: var(--color-calendar-today); /* 🎨 今日强调色 */
  font-weight: 700; /* 📝 最粗字重 */
  padding: 0.2rem 0.6rem; /* 📐 徽章内边距 */
  border-radius: 999px; /* ⭕ 胶囊形状 */
  display: inline-flex; /* 🎪 弹性布局 */
  align-items: center; /* ⬆️ 垂直居中 */
  justify-content: center; /* ↔️ 水平居中 */
}

/* 📦 月视图事件样式 */
.fc .fc-daygrid-event {
  margin: 1px 2px; /* 📏 事件间距 */
  padding: 2px 4px; /* 📐 事件内边距 */
  border-radius: 3px; /* ⭕ 圆角 */
  font-size: 1.2rem; /* 📏 字体大小 */
}

/* 📝 "+N more"链接样式 */
.fc .fc-daygrid-more-link {
  font-size: 1.1rem; /* 📏 字体大小 */
  font-weight: 600; /* 📝 字重 */
  color: var(--color-text-accent); /* 🎨 强调色 */
  padding: 2px 4px; /* 📐 内边距 */
  border-radius: 3px; /* ⭕ 圆角 */
  transition: background-color 0.15s ease; /* 🎬 过渡动画 */
  cursor: pointer; /* 👆 可点击 */
}

.fc .fc-daygrid-more-link:hover {
  background-color: var(--color-background-hover); /* 🖱️ 悬停背景 */
  text-decoration: none; /* ❌ 移除下划线 */
}

/* ===============================================
 * 11. 月视图高度固定 - 防止内容撑高布局
 * =============================================== */

/* stylelint-disable selector-class-pattern */

/* 📏 月视图固定行高 - 防止事件过多撑高 */
.fc-dayGridMonth-view .fc-daygrid-body tr {
  height: 120px !important; /* 🔒 强制固定行高 */
}

.fc-dayGridMonth-view .fc-daygrid-day-frame {
  height: 120px !important; /* 🔒 固定单元格高度 */
  overflow: hidden; /* ❌ 隐藏超出内容，配合dayMaxEvents */
}

/* 📦 月视图事件容器 */
.fc-dayGridMonth-view .fc-daygrid-day-events {
  min-height: auto !important; /* 📏 自动最小高度 */
  overflow: visible; /* ✅ 允许"+N more"显示 */
}

/* stylelint-enable selector-class-pattern */

/* ===============================================
 * 12. Popover弹窗样式 - 月视图"+more"事件展示
 * =============================================== */

/* 🎪 Popover主容器 */
.fc .fc-popover {
  background: var(--color-background-primary); /* 🎭 主背景色 */
  border-radius: 8px; /* ⭕ 大圆角 */
  box-shadow: 0 4px 12px rgb(0 0 0 / 15%); /* 🌫️ 深度阴影 */
  z-index: 9999; /* 🔝 最高层级 */

  /* 🔲 border由--fc-border-color变量控制 */
}

/* 📋 Popover头部 */
.fc .fc-popover-header {
  background: var(--color-background-primary); /* 🎭 背景色 */
  padding: 0.8rem 1rem; /* 📐 内边距 */
  border-radius: 8px 8px 0 0; /* ⭕ 顶部圆角 */

  /* 🔲 border-bottom由--fc-border-color变量控制 */
}

/* 📝 Popover标题 */
.fc .fc-popover-title {
  font-size: 1.3rem; /* 📏 标题字体 */
  font-weight: 600; /* 📝 加粗 */
  color: var(--color-text-primary); /* 🎨 主要文字色 */
}

/* ❌ Popover关闭按钮 */
.fc .fc-popover-close {
  font-size: 1.6rem; /* 📏 关闭按钮大小 */
  color: var(--color-text-secondary); /* 🎨 次要文字色 */
  cursor: pointer; /* 👆 可点击 */
  opacity: 0.6; /* 👻 半透明 */
  transition: opacity 0.15s ease; /* 🎬 透明度过渡 */
}

.fc .fc-popover-close:hover {
  opacity: 1; /* 🔆 悬停时完全不透明 */
}

/* 📄 Popover内容区域 */
.fc .fc-popover-body {
  background: var(--color-background-primary); /* 🎭 背景色 */
  padding: 0.4rem; /* 📐 内边距 */
  max-height: 400px; /* 📏 最大高度限制 */
  overflow-y: auto; /* 📜 垂直滚动 */
  border-radius: 0 0 8px 8px; /* ⭕ 底部圆角 */
}

/* 📦 Popover内事件样式 */
.fc .fc-popover-body .fc-daygrid-event {
  margin: 2px 0; /* 📏 事件间距 */
  cursor: pointer; /* 👆 可点击 */
}

.fc .fc-popover-body .fc-daygrid-event:hover {
  opacity: 0.8; /* 👻 悬停透明效果 */
}

/* ===============================================
 * 13. 装饰线系统 - 时间分隔视觉辅助
 * =============================================== */

.decorative-line {
  position: fixed; /* 📍 固定定位，参照视口 */
  width: 0.8px; /* 📏 细线宽度 */
  background: var(--color-border-default); /* 🎨 默认边框色 */
  pointer-events: none; /* 🖱️ 鼠标事件穿透 */
  z-index: 5; /* 🔝 适中的层级 */
}

/* ===============================================
 * 14. 自定义日期头部 - 多日视图顶部导航
 * =============================================== */

/* 📅 自定义日期头部容器 */
.custom-day-headers {
  display: flex; /* 🎪 弹性布局 */
  align-items: center; /* ⬆️ 垂直居中 */
  background-color: var(--color-background-content); /* 🎭 内容背景色 */
  border-bottom: 1px solid var(--color-border-default); /* 🔲 底部边框 */
  position: sticky; /* 📍 粘性定位 */
  top: 0; /* 🔝 顶部对齐 */
  z-index: 10; /* 🔝 高层级 */
  height: 48px; /* 📏 固定高度 */
}

/* ⏰ 时间轴占位符 */
.time-axis-placeholder {
  flex-shrink: 0; /* 🚫 不收缩 */
  height: 100%; /* 📏 继承容器高度 */
}

/* 📅 单个日期头部 */
.custom-day-header {
  flex-shrink: 0; /* 🚫 固定宽度，不收缩 */
  display: flex; /* 🎪 弹性布局 */
  flex-direction: row; /* ➡️ 水平排列 */
  align-items: center; /* ⬆️ 垂直居中 */
  justify-content: center; /* ↔️ 水平居中 */
  gap: 0.6rem; /* 📏 子元素间距 */
  padding: 0 0.4rem; /* 📐 水平内边距 */
  height: 100%; /* 📏 继承容器高度 */
  transition: background-color 0.2s ease; /* 🎬 背景色过渡 */
  box-sizing: border-box; /* 📦 边框盒模型 */
  cursor: pointer; /* 👆 可点击 */
}

/* 🖱️ 日期头部悬停效果 */
.custom-day-header:hover {
  background-color: var(--color-background-hover); /* 🎨 悬停背景 */
}

/* 🎯 拖拽目标状态 */
.custom-day-header.is-drag-target {
  background-color: var(--color-background-accent-light); /* 🎨 强调背景 */
  border-color: var(--color-text-accent); /* 🎨 强调边框 */
}

/* 📍 拖拽预览指示器 */
.drag-preview-indicator {
  font-size: 1.6rem; /* 📏 指示器大小 */
  font-weight: 600; /* 📝 加粗 */
  color: var(--color-text-accent); /* 🎨 强调色 */
  line-height: 1; /* 📏 紧凑行高 */
}

/* 📝 日期头部文字元素 */
.custom-day-header .day-name {
  font-size: 1.4rem; /* 📏 日期名字体 */
  font-weight: 600; /* 📝 加粗 */
  color: var(--color-text-secondary); /* 🎨 次要文字色 */
  text-transform: uppercase; /* 🔤 大写转换 */
  line-height: 1.4; /* 📏 固定行高，避免中英文高度差异 */
}

.custom-day-header .date-number {
  font-size: 1.6rem; /* 📏 日期数字字体 */
  font-weight: 500; /* 📝 中等字重 */
  color: var(--color-text-primary); /* 🎨 主要文字色 */
  line-height: 1.4; /* 📏 固定行高，避免中英文高度差异 */
}

/* 🌟 周视图今天日期数字 - 圆角矩形背景 */
.calendar-container.view-type-week .custom-day-header .date-number.is-today {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.2rem 0.6rem;
  font-weight: 600;
  color: var(--color-text-on-accent);
  background-color: var(--color-calendar-today);
  border-radius: 0.4rem;
}

/* 🌟 今日徽章（仅单日/多日视图显示） */
.custom-day-header .today-badge {
  display: inline-flex; /* 🎪 内联弹性布局 */
  align-items: center; /* ⬆️ 垂直居中 */
  padding: 0.2rem 0.6rem; /* 📐 徽章内边距 */
  margin-left: 0.4rem; /* 📏 左边距 */
  font-size: 1.3rem; /* 📏 徽章字体 */
  font-weight: 600; /* 📝 加粗 */
  color: var(--color-text-accent); /* 🎨 强调文字色 */
  background-color: var(--color-background-accent-light); /* 🎨 强调背景 */
  border-radius: 1rem; /* ⭕ 胶囊形状 */
  line-height: 1.4; /* 📏 舒适行高 */
}
</style>

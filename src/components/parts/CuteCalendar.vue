<template>
  <div
    class="calendar-container"
    @dragenter="handleDragEnter"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <FullCalendar :options="calendarOptions" />
  </div>
</template>

<script setup lang="ts">
import FullCalendar from '@fullcalendar/vue3'
import interactionPlugin from '@fullcalendar/interaction'
import timeGridPlugin from '@fullcalendar/timegrid'
import { reactive, onMounted, onUnmounted, computed, ref } from 'vue'
import { useMessage } from 'naive-ui'
import { useTimeBlockStore } from '@/stores/timeblock'
import type { EventInput, EventChangeArg, DateSelectArg, EventMountArg } from '@fullcalendar/core'
import { useContextMenu } from '@/composables/useContextMenu'
import CalendarEventMenu from '@/components/parts/CalendarEventMenu.vue'
import type { Task } from '@/types/models'

const timeBlockStore = useTimeBlockStore()
const contextMenu = useContextMenu()
const message = useMessage()

// 预览时间块状态
const previewEvent = ref<EventInput | null>(null)
const isDragging = ref(false)
const currentDraggedTask = ref<Task | null>(null)

onMounted(() => {
  // 获取当前日期范围的时间块
  const today = new Date()
  const startOfWeek = new Date(today.setDate(today.getDate() - today.getDay()))
  const endOfWeek = new Date(today.setDate(today.getDate() - today.getDay() + 6))
  
  timeBlockStore.fetchTimeBlocksForRange(
    startOfWeek.toISOString(),
    endOfWeek.toISOString()
  )

  // 监听全局拖拽开始事件
  document.addEventListener('dragstart', handleGlobalDragStart)
  document.addEventListener('dragend', handleGlobalDragEnd)
})

function handleGlobalDragStart(event: DragEvent) {
  try {
    if (event.dataTransfer) {
      const dragData = JSON.parse(event.dataTransfer.getData('application/json'))
      if (dragData.type === 'task' && dragData.task) {
        currentDraggedTask.value = dragData.task
      }
    }
  } catch (error) {
    // 忽略解析错误
  }
}

function handleGlobalDragEnd() {
  currentDraggedTask.value = null
  clearPreviewEvent()
  stopAutoScroll()
}

onUnmounted(() => {
  // 清理事件监听器
  document.removeEventListener('dragstart', handleGlobalDragStart)
  document.removeEventListener('dragend', handleGlobalDragEnd)
})

const calendarEvents = computed((): EventInput[] => {
  const events = timeBlockStore.allTimeBlocks.map((timeBlock) => ({
    id: timeBlock.id,
    title: timeBlock.title ?? 'Time Block',
    start: timeBlock.start_time,
    end: timeBlock.end_time,
    allDay: false, // 时间块总是有具体时间
    color: '#4a90e2', // 默认颜色，可以根据area_id来设置不同颜色
  }))

  // 添加预览事件
  if (previewEvent.value) {
    events.push({
      id: previewEvent.value.id || 'preview-event',
      title: previewEvent.value.title || '预览',
      start: typeof previewEvent.value.start === 'string' ? previewEvent.value.start : '',
      end: typeof previewEvent.value.end === 'string' ? previewEvent.value.end : '',
      allDay: previewEvent.value.allDay || false,
      color: previewEvent.value.color || '#4a90e2',
    })
  }

  return events
})

async function handleDateSelect(selectInfo: DateSelectArg) {
  const calendarApi = selectInfo.view.calendar
  calendarApi.unselect() // clear date selection

  const title = prompt('Please enter a new title for your time block')
  if (title) {
    try {
      await timeBlockStore.createTimeBlock({
        title,
        start_time: selectInfo.start.toISOString(),
        end_time: selectInfo.end.toISOString(),
        task_ids: [], // 空的任务ID列表
      })
    } catch (error) {
      console.error('Failed to create event:', error)

      // 显示错误信息给用户
      let errorMessage = 'Could not create the event. It might be overlapping with another event.'
      if (error instanceof Error) {
        errorMessage = error.message
      } else if (typeof error === 'string') {
        errorMessage = error
      }

      message.error(`创建事件失败: ${errorMessage}`, {
        duration: 5000,
        closable: true,
      })
      // No need to manually revert, as it was never added to the store successfully
    }
  }
}

async function handleEventChange(changeInfo: EventChangeArg) {
  const { event, oldEvent } = changeInfo

  // Check if this is a drag from all-day to timed event
  const wasAllDay = oldEvent.allDay
  const isNowTimed = !event.allDay

  let startTime = event.start?.toISOString()
  let endTime = event.end?.toISOString()

  // If dragging from all-day to timed, set duration to 1 hour
  if (wasAllDay && isNowTimed && event.start) {
    const start = new Date(event.start)
    const end = new Date(start.getTime() + 60 * 60 * 1000) // Add 1 hour
    startTime = start.toISOString()
    endTime = end.toISOString()

    console.log(
      `[Calendar] Converting all-day event to 1-hour timed event: ${startTime} - ${endTime}`
    )
  }

  try {
    await timeBlockStore.updateTimeBlock(event.id, {
      title: event.title,
      start_time: startTime,
      end_time: endTime,
    })
  } catch (error) {
    console.error('Failed to update event:', error)

    // 显示错误信息给用户
    let errorMessage = 'Could not update the event. It might be overlapping with another event.'
    if (error instanceof Error) {
      errorMessage = error.message
    } else if (typeof error === 'string') {
      errorMessage = error
    }

    message.error(`更新事件失败: ${errorMessage}`, {
      duration: 5000,
      closable: true,
    })

    changeInfo.revert() // Revert the change on the calendar
  }
}

function handleEventContextMenu(info: EventMountArg) {
  info.el.addEventListener('contextmenu', (e: MouseEvent) => {
    contextMenu.show(CalendarEventMenu, { event: info.event }, e)
  })
}

let lastUpdateTime = 0
const UPDATE_THROTTLE = 16 // 约60fps
const SCROLL_ZONE_SIZE = 100 // 触发滚动的边缘区域大小（像素）
const SCROLL_SPEED = 5 // 滚动速度（像素/次）
let scrollTimer: number | null = null

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy'
  }

  // 节流更新预览，避免过于频繁的计算
  const now = Date.now()
  if (isDragging.value && now - lastUpdateTime > UPDATE_THROTTLE) {
    updatePreviewEvent(event)
    handleAutoScroll(event)
    lastUpdateTime = now
  }
}

function handleDragEnter(event: DragEvent) {
  event.preventDefault()

  // 检查是否包含任务数据
  if (event.dataTransfer && event.dataTransfer.types.includes('application/json')) {
    isDragging.value = true
  }
}

function handleDragLeave(event: DragEvent) {
  // 检查是否真的离开了日历区域
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  const x = event.clientX
  const y = event.clientY

  if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
    clearPreviewEvent()
    stopAutoScroll()
  }
}

function handleAutoScroll(event: DragEvent) {
  const calendarContainer = event.currentTarget as HTMLElement
  const scrollableEl = calendarContainer.querySelector('.fc-scroller') as HTMLElement

  if (!scrollableEl) return

  const rect = scrollableEl.getBoundingClientRect()
  const mouseY = event.clientY
  const relativeY = mouseY - rect.top

  let scrollDirection = 0

  // 检查是否在顶部滚动区域
  if (relativeY < SCROLL_ZONE_SIZE) {
    scrollDirection = -1 // 向上滚动
  }
  // 检查是否在底部滚动区域
  else if (relativeY > rect.height - SCROLL_ZONE_SIZE) {
    scrollDirection = 1 // 向下滚动
  }

  if (scrollDirection !== 0) {
    startAutoScroll(scrollableEl, scrollDirection)
  } else {
    stopAutoScroll()
  }
}

function startAutoScroll(scrollableEl: HTMLElement, direction: number) {
  // 如果已经在滚动，就不重复启动
  if (scrollTimer !== null) return

  scrollTimer = window.setInterval(() => {
    const scrollAmount = SCROLL_SPEED * direction
    scrollableEl.scrollTop += scrollAmount

    // 检查是否已经到达边界
    if (direction < 0 && scrollableEl.scrollTop <= 0) {
      stopAutoScroll()
    } else if (
      direction > 0 &&
      scrollableEl.scrollTop >= scrollableEl.scrollHeight - scrollableEl.clientHeight
    ) {
      stopAutoScroll()
    }
  }, 16) // 约60fps
}

function stopAutoScroll() {
  if (scrollTimer !== null) {
    clearInterval(scrollTimer)
    scrollTimer = null
  }
}

function updatePreviewEvent(event: DragEvent) {
  const dropTime = getTimeFromDropPosition(event)

  if (dropTime) {
    const endTime = new Date(dropTime.getTime() + 60 * 60 * 1000)

    // 使用全局状态中的任务信息
    const previewTitle = currentDraggedTask.value?.title || '任务'

    previewEvent.value = {
      id: 'preview-event',
      title: previewTitle,
      start: dropTime.toISOString(),
      end: endTime.toISOString(),
      allDay: false,
      color: '#4a90e2',
      classNames: ['preview-event'],
      display: 'block',
    }
  }
}

function clearPreviewEvent() {
  previewEvent.value = null
  isDragging.value = false
  // 清理缓存
  cachedCalendarEl = null
  cachedRect = null
  // 停止自动滚动
  stopAutoScroll()
}

async function handleDrop(event: DragEvent) {
  event.preventDefault()

  // 清除预览事件
  clearPreviewEvent()

  if (!event.dataTransfer) return

  try {
    const dragData = JSON.parse(event.dataTransfer.getData('application/json'))

    if (dragData.type === 'task' && dragData.task) {
      // 获取拖拽位置对应的时间
      const dropTime = getTimeFromDropPosition(event)

      if (dropTime) {
        // 创建一个默认1小时的活动
        const endTime = new Date(dropTime.getTime() + 60 * 60 * 1000)

        await timeBlockStore.createTimeBlock({
          title: dragData.task.title,
          start_time: dropTime.toISOString(),
          end_time: endTime.toISOString(),
          task_ids: [dragData.task.id], // 关联拖拽的任务
        })

        console.log(`创建时间块: ${dragData.task.title} at ${dropTime.toISOString()}`)
      }
    }
  } catch (error) {
    console.error('处理拖拽失败:', error)

    // 显示错误信息给用户
    let errorMessage = '创建时间块失败'
    if (error instanceof Error) {
      errorMessage = error.message
    } else if (typeof error === 'string') {
      errorMessage = error
    }

    // 使用 Naive UI 消息组件显示错误
    message.error(`创建时间块失败: ${errorMessage}`, {
      duration: 5000, // 显示5秒
      closable: true,
    })
  }
}

let cachedCalendarEl: HTMLElement | null = null
let cachedRect: DOMRect | null = null

function getTimeFromDropPosition(event: DragEvent): Date | null {
  // 缓存DOM元素和位置信息，避免重复查询
  if (!cachedCalendarEl) {
    cachedCalendarEl = (event.currentTarget as HTMLElement).querySelector('.fc-timegrid-body')
  }
  if (!cachedCalendarEl) return null

  // 只在必要时重新计算位置
  const now = Date.now()
  if (!cachedRect || now - lastUpdateTime > UPDATE_THROTTLE) {
    cachedRect = cachedCalendarEl.getBoundingClientRect()
  }

  const relativeY = event.clientY - cachedRect.top

  // 计算相对于日历顶部的百分比
  const percentage = relativeY / cachedRect.height

  // 获取当前日期
  const today = new Date()
  today.setHours(0, 0, 0, 0)

  // 计算时间（从0:00到24:00，共24小时）
  const totalMinutes = percentage * 24 * 60
  const hours = Math.floor(totalMinutes / 60)
  const minutes = Math.floor((totalMinutes % 60) / 10) * 10 // 10分钟间隔对齐

  const dropTime = new Date(today)
  dropTime.setHours(hours, minutes, 0, 0)

  return dropTime
}

const calendarOptions = reactive({
  plugins: [interactionPlugin, timeGridPlugin],
  headerToolbar: false as const,
  dayHeaders: false,
  initialView: 'timeGridDay',
  slotLabelFormat: {
    hour: '2-digit' as const,
    minute: '2-digit' as const,
    hour12: false,
  },
  slotMinTime: '00:00:00', // 从0:00开始显示
  slotMaxTime: '24:00:00', // 到24:00结束
  slotDuration: '00:10:00', // 10分钟时间槽
  snapDuration: '00:10:00', // 10分钟对齐精度
  nowIndicator: true, // 显示当前时间指示器
  height: '100%',
  weekends: true,
  editable: true,
  selectable: true,
  eventResizableFromStart: true, // 允许从开始时间调整大小
  events: calendarEvents,
  select: handleDateSelect,
  eventChange: handleEventChange,
  eventDidMount: handleEventContextMenu,
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
}

/* 预览事件样式 */
.fc-event.preview-event {
  background-color: #4a90e2 !important;
  color: #fff !important;
  border-color: #357abd !important;
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
  padding: 1rem !important; /* 增加分隔线区域的内边距 */
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
</style>

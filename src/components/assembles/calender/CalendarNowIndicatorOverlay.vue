<template>
  <div v-if="shouldRender" class="now-indicator-overlay" :style="{ top: `${lineTop}px` }">
    <div class="now-indicator-label" :style="labelStyle">{{ currentTimeText }}</div>

    <div class="now-indicator-lines">
      <div
        v-for="segment in segments"
        :key="segment.date"
        class="now-indicator-line"
        :class="{ 'is-today': segment.isToday }"
        :style="{ left: `${segment.left}px`, width: `${segment.width}px` }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

type DateInfo = {
  date: string
  isToday?: boolean
  width?: number
}

const props = defineProps<{
  calendarRoot: HTMLElement | null
  dates: DateInfo[]
  viewType: 'day' | 'week' | 'month'
  timeAxisWidth?: number
}>()

const debugEnabled = import.meta.env?.MODE !== 'production'
const debug = (msg: string, payload?: Record<string, unknown>) => {
  if (!debugEnabled) return
  // eslint-disable-next-line no-console
  console.debug(`[now-indicator] ${msg}`, payload ?? {})
}

const lineTop = ref<number | null>(null)
const segments = ref<Array<{ date: string; left: number; width: number; isToday: boolean }>>([])
const currentTimeText = ref<string>('')

let refreshTimer: number | null = null
let resizeObserver: ResizeObserver | null = null
let scrollEl: HTMLElement | null = null

const shouldRender = computed(
  () =>
    Boolean(props.calendarRoot) &&
    props.viewType !== 'month' &&
    segments.value.length > 0 &&
    lineTop.value !== null
)

const labelStyle = computed(() => {
  const axisWidth = props.timeAxisWidth ?? 0
  return {
    // 让时间标签更贴近时间轴，适度向左收缩
    left: `${Math.max(0, axisWidth - 27)}px`,
  }
})

function parseMinutes(value: string | null | undefined): number | null {
  if (!value) return null
  const parts = value.split(':')
  const hours = Number(parts[0])
  const minutes = Number(parts[1])
  const seconds = Number(parts[2] ?? '0')

  if ([hours, minutes, seconds].some((n) => Number.isNaN(n))) {
    return null
  }

  return hours * 60 + minutes + seconds / 60
}

function updateTimeText() {
  const now = new Date()
  currentTimeText.value = now.toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
}

function updateSegments() {
  if (!props.calendarRoot || props.viewType === 'month') {
    debug('skip updateSegments (no root or month view)')
    segments.value = []
    return
  }

  const containerRect = props.calendarRoot.getBoundingClientRect()
  const todaySet = new Set(props.dates.filter((d) => d.isToday).map((d) => d.date))
  const columns = props.calendarRoot.querySelectorAll('.fc-timegrid-col[data-date]')

  const nextSegments: Array<{ date: string; left: number; width: number; isToday: boolean }> = []

  columns.forEach((col) => {
    const date = col.getAttribute('data-date')
    if (!date) return

    const rect = col.getBoundingClientRect()
    nextSegments.push({
      date,
      left: rect.left - containerRect.left,
      width: rect.width,
      isToday: todaySet.has(date),
    })
  })

  segments.value = nextSegments
  debug('segments updated', {
    count: nextSegments.length,
    dates: nextSegments.map((s) => s.date),
  })
}

const handleScroll = () => {
  updateTimePosition()
}

function detachScrollListener() {
  if (scrollEl) {
    scrollEl.removeEventListener('scroll', handleScroll)
    scrollEl = null
  }
}

function attachScrollListener(nextEl: HTMLElement | null) {
  if (scrollEl === nextEl) return
  detachScrollListener()
  if (nextEl) {
    scrollEl = nextEl
    scrollEl.addEventListener('scroll', handleScroll, { passive: true })
  }
}

function updateTimePosition() {
  if (typeof window === 'undefined' || !props.calendarRoot || props.viewType === 'month') {
    debug('skip updateTimePosition (no window/root or month view)')
    lineTop.value = null
    return
  }

  // 🔧 使用 .fc-timegrid-slots 容器，排除全天槽区域
  const slotsContainer = props.calendarRoot.querySelector(
    '.fc-timegrid-slots'
  ) as HTMLElement | null
  if (!slotsContainer) {
    debug('slotsContainer not found')
    lineTop.value = null
    return
  }

  const scroller = slotsContainer.closest('.fc-scroller') as HTMLElement | null
  attachScrollListener(scroller)

  // 🔧 只查询带有 data-time 属性的时间槽（排除全天槽）
  const slotNodes = slotsContainer.querySelectorAll('.fc-timegrid-slot[data-time]')
  if (!slotNodes.length) {
    debug('slotNodes empty')
    lineTop.value = null
    return
  }

  // 解析所有 slot 的时间和位置信息
  const slots: Array<{ el: HTMLElement; minutes: number }> = []
  slotNodes.forEach((node) => {
    const el = node as HTMLElement
    const minutes = parseMinutes(el.dataset.time) ?? parseMinutes(el.getAttribute('data-time'))
    if (minutes !== null) {
      slots.push({ el, minutes })
    }
  })

  const firstSlot = slots[0]
  if (!firstSlot) {
    debug('no valid slots')
    lineTop.value = null
    return
  }

  // 计算步长（相邻 slot 的时间差）
  const secondSlot = slots[1]
  const stepMinutes = secondSlot ? secondSlot.minutes - firstSlot.minutes : 5

  const now = new Date()
  const nowMinutes = now.getHours() * 60 + now.getMinutes() + now.getSeconds() / 60

  // 🔧 新算法：找到当前时间所在的 slot，用该 slot 的实际位置作为基准
  // 这样可以避免不同缩放下的累积舍入误差
  let targetSlot = firstSlot
  for (const slot of slots) {
    if (slot.minutes <= nowMinutes) {
      targetSlot = slot
    } else {
      break
    }
  }

  // 计算在目标 slot 内的偏移比例
  const offsetInSlot = nowMinutes - targetSlot.minutes
  const ratio = Math.min(Math.max(offsetInSlot / stepMinutes, 0), 1)

  // 使用目标 slot 的实际 rect 位置，而不是从头累加
  const containerRect = props.calendarRoot.getBoundingClientRect()
  const slotRect = targetSlot.el.getBoundingClientRect()
  const slotHeight = slotRect.height

  // 最终位置 = slot 顶部相对容器的位置 + slot 内偏移
  lineTop.value = slotRect.top - containerRect.top + slotHeight * ratio

  debug('updateTimePosition', {
    targetSlotTime: targetSlot.minutes,
    slotHeight,
    stepMinutes,
    nowMinutes,
    offsetInSlot,
    ratio,
    lineTop: lineTop.value,
  })
}

async function refreshAll() {
  await nextTick()
  updateSegments()
  updateTimePosition()
  updateTimeText()
}

function cleanupResizeObserver() {
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
}

function registerResizeObserver() {
  cleanupResizeObserver()
  if (!props.calendarRoot) return

  resizeObserver = new ResizeObserver(() => {
    refreshAll()
  })
  resizeObserver.observe(props.calendarRoot)
}

watch(
  () => props.calendarRoot,
  () => {
    registerResizeObserver()
    refreshAll()
  }
)

watch(
  () => props.viewType,
  () => {
    refreshAll()
  }
)

watch(
  () => props.dates,
  () => {
    refreshAll()
  },
  { deep: true }
)

onMounted(() => {
  if (typeof window === 'undefined') return

  refreshAll()
  registerResizeObserver()
  // 每秒更新位置和时间，实现与系统时间实时同步
  refreshTimer = window.setInterval(() => {
    updateTimePosition()
    updateTimeText()
  }, 1000)
})

onBeforeUnmount(() => {
  cleanupResizeObserver()
  detachScrollListener()
  if (refreshTimer) {
    window.clearInterval(refreshTimer)
    refreshTimer = null
  }
})
</script>

<style scoped>
.now-indicator-overlay {
  position: absolute;
  left: 0;
  right: 0;
  z-index: 12;
  pointer-events: none;
}

.now-indicator-label {
  position: absolute;
  top: -1px; /* 与线的顶部位置对齐 */
  transform: translateY(-50%);
  padding: 0;
  background: transparent;
  border: none;
  border-radius: 0;
  font-size: 1.3rem; /* 与时间轴标签 .fc-timegrid-slot-label-cushion 一致 */
  font-weight: 500; /* 与时间轴标签一致 */
  line-height: 1; /* 确保文字在元素内垂直居中 */
  color: var(--color-danger);
  box-shadow: none;
}

.now-indicator-lines {
  position: relative;
  height: 0;
}

.now-indicator-line {
  position: absolute;
  top: -1px;
  height: 2px;
  background: var(--color-danger);
  opacity: 0.35; /* 仍用危险色，降低不透明度呈现偏灰效果 */
}

.now-indicator-line.is-today {
  background: var(--color-danger);
  opacity: 1;
}

/* 今日实线两端的小竖线 */
.now-indicator-line.is-today::before,
.now-indicator-line.is-today::after {
  content: '';
  position: absolute;
  top: 50%;
  width: 2px;
  height: 8px;
  background: var(--color-danger);
  transform: translateY(-50%);
}

.now-indicator-line.is-today::before {
  left: 0;
}

.now-indicator-line.is-today::after {
  right: 0;
}
</style>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed, nextTick } from 'vue'
import type { ViewMetadata, DateViewConfig } from '@/types/drag'
import SimpleKanbanColumn from '@/components/parts/kanban/SimpleKanbanColumn.vue'
// import { useTaskStore } from '@/stores/task' // 🗑️ 不再需要
import { useViewStore } from '@/stores/view'
import { useRegisterStore } from '@/stores/register'
import { controllerDebugState } from '@/infra/drag-interact'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString, toDateString, isSameDate } from '@/infra/utils/dateUtils'

// ==================== Stores ====================
// const taskStore = useTaskStore() // 🗑️ 不再需要：SimpleKanbanColumn 内部处理任务数据
const viewStore = useViewStore()
const registerStore = useRegisterStore()

// ==================== 配置常量 ====================
const KANBAN_WIDTH = 23 // 每个看板宽度（rem）
const REM_TO_PX = 10 // 1rem = 10px (定义在 style.css 中)
const KANBAN_WIDTH_PX = KANBAN_WIDTH * REM_TO_PX // 230px，用于滚动计算
const KANBAN_GAP_PX = 0 // ✅ gap 设为 0（看板自身 padding 填补缝隙）
const TRACK_PADDING_PX = 1 * REM_TO_PX // track 的左右 padding 1rem = 10px
const KANBAN_TOTAL_WIDTH_PX = KANBAN_WIDTH_PX + KANBAN_GAP_PX // 每个看板总宽度 = 230px
const VISIBLE_COUNT = 6 // 可见看板数量（用户屏幕显示的）
const BUFFER_SIZE = 7 // 左右缓冲区大小（增大缓冲区，提前加载）
const TOTAL_KANBANS = VISIBLE_COUNT + BUFFER_SIZE * 2 // 总共 20 个看板 (7+6+7)
const TRIGGER_DISTANCE = 3 // 触发加载的距离（距离缓冲区边界几个看板时触发）

// ==================== 状态 ====================
const scrollContainer = ref<HTMLElement | null>(null)
const isScrolling = ref(false) // 防止滚动补偿时触发额外逻辑

// 拖动滚动状态（看板横向拖动）
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartScrollLeft = ref(0)

// ✅ 使用 interact.js 的全局拖动状态来检测任务卡片是否正在拖动
const isTaskDragging = computed(() => {
  // 当拖动状态不是 IDLE 时，说明有任务正在被拖动
  return controllerDebugState.value.phase !== 'IDLE'
})

// ==================== Props ====================
// 🗑️ 移除 props drilling - 现在直接从 register store 读取

// ==================== 日期看板系统 ====================
interface DailyKanban {
  id: string // 日期字符串 YYYY-MM-DD
  date: Date
  viewKey: string // daily::YYYY-MM-DD
  offset: number // 相对于今天的偏移量
}

const kanbans = ref<DailyKanban[]>([])

// 格式化日期为 YYYY-MM-DD
function formatDate(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

// 添加天数到日期
function addDays(date: Date, days: number): Date {
  const result = new Date(date)
  result.setDate(result.getDate() + days)
  return result
}

// 判断是否是今天
function isToday(date: Date): boolean {
  return isSameDate(toDateString(date), getTodayDateString())
}

// 🆕 判断看板是否过期（日期在今天之前）
function isExpired(date: Date): boolean {
  const today = new Date()
  today.setHours(0, 0, 0, 0) // 重置到当天的开始时间
  const compareDate = new Date(date)
  compareDate.setHours(0, 0, 0, 0)
  return compareDate < today
}

// 🆕 判断日期是否与当前日历日期相同
// ✅ 直接从寄存器读取，消除 props drilling
function isCalendarDate(date: Date): boolean {
  const currentCalendarDate = registerStore.readRegister<string>(
    registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME
  )

  if (!currentCalendarDate) return false

  const dateStr = formatDate(date)
  const isMatch = dateStr === currentCalendarDate
  // logger.debug(LogTags.COMPONENT_KANBAN, 'Checking calendar date match', {
  //   kanbanDate: dateStr,
  //   calendarDate: currentCalendarDate,
  //   isMatch,
  // })
  return isMatch
}

// 获取星期几（中文）
function getWeekdayName(date: Date): string {
  const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  return weekdays[date.getDay()] || '周日'
}

// 初始化看板
function initKanbans() {
  const today = new Date()

  const initialKanbans: DailyKanban[] = []
  // 创建20个看板：左缓冲7个 + 可见6个 + 右缓冲7个
  // 今天在可见区的第一个位置（索引7）
  for (let i = 0; i < TOTAL_KANBANS; i++) {
    const offsetFromToday = i - BUFFER_SIZE // 索引7对应offset=0（今天）
    const date = addDays(today, offsetFromToday)
    const dateStr = formatDate(date)
    initialKanbans.push({
      id: dateStr,
      date: date,
      viewKey: `daily::${dateStr}`,
      offset: offsetFromToday,
    })
  }

  kanbans.value = initialKanbans
  // console.log('[InfiniteDailyKanban] 📅 Initialized kanbans:', {
  //   total: kanbans.value.length,
  //   first: kanbans.value[0]?.id,
  //   today: kanbans.value[BUFFER_SIZE]?.id,
  //   last: kanbans.value[kanbans.value.length - 1]?.id,
  // })

  // 设置初始滚动位置：让今天（索引7）显示在可见区左侧
  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.scrollLeft = BUFFER_SIZE * KANBAN_TOTAL_WIDTH_PX
      // console.log('[InfiniteDailyKanban] 📍 Initial scroll position:', {
      //   scrollLeft: scrollContainer.value.scrollLeft,
      //   calculation: `${BUFFER_SIZE} * ${KANBAN_TOTAL_WIDTH_PX} = ${BUFFER_SIZE * KANBAN_TOTAL_WIDTH_PX}`,
      // })
    }
  })
}

// 批量移动看板：一次性添加/移除多个并做一次滚动补偿
function shiftKanbansBatch(direction: 'left' | 'right', steps: number) {
  if (isScrolling.value || kanbans.value.length === 0) return
  if (steps <= 0) return

  isScrolling.value = true
  const currentScrollLeft = scrollContainer.value?.scrollLeft || 0

  if (direction === 'right') {
    for (let i = 0; i < steps; i++) {
      // 移除最左侧
      kanbans.value.shift()

      // 在右侧添加新看板（未来日期）
      const lastKanban = kanbans.value[kanbans.value.length - 1]
      if (!lastKanban) break

      const newDate = addDays(lastKanban.date, 1)
      const dateStr = formatDate(newDate)
      kanbans.value.push({
        id: dateStr,
        date: newDate,
        viewKey: `daily::${dateStr}`,
        offset: lastKanban.offset + 1,
      })
    }
  } else {
    for (let i = 0; i < steps; i++) {
      // 移除最右侧
      kanbans.value.pop()

      // 在左侧添加新看板（过去日期）
      const firstKanban = kanbans.value[0]
      if (!firstKanban) break

      const newDate = addDays(firstKanban.date, -1)
      const dateStr = formatDate(newDate)
      kanbans.value.unshift({
        id: dateStr,
        date: newDate,
        viewKey: `daily::${dateStr}`,
        offset: firstKanban.offset - 1,
      })
    }
  }

  if (!scrollContainer.value) {
    isScrolling.value = false
    return
  }

  const originalBehavior = scrollContainer.value.style.scrollBehavior
  scrollContainer.value.style.scrollBehavior = 'auto'

  nextTick(() => {
    if (!scrollContainer.value) {
      isScrolling.value = false
      return
    }

    const compensation = steps * KANBAN_TOTAL_WIDTH_PX
    scrollContainer.value.scrollLeft =
      direction === 'right' ? currentScrollLeft - compensation : currentScrollLeft + compensation

    // 恢复原始滚动行为
    scrollContainer.value.style.scrollBehavior = originalBehavior

    // 短暂锁定，避免重复触发
    setTimeout(() => {
      isScrolling.value = false
    }, 1)
  })
}

// 🗑️ 已删除：滚动导致日历变化的功能
// - calculateVisibleLeftmostDate()
// - handleScroll()

// 为每个看板获取任务（响应式）
// 🗑️ 移除：任务获取和排序现在由 SimpleKanbanColumn 内部处理
// const kanbanTasksMap = computed(() => { ... })
// function getKanbanTasks(kanban: DailyKanban): TaskCard[] { ... }

// 🆕 为每个看板生成 ViewMetadata
function getKanbanMetadata(kanban: DailyKanban): ViewMetadata {
  const config: DateViewConfig = {
    date: kanban.id, // YYYY-MM-DD
  }

  return {
    type: 'date',
    id: kanban.viewKey, // daily::YYYY-MM-DD
    config,
    label: `${kanban.date.getMonth() + 1}月${kanban.date.getDate()}日`,
  }
}

// ==================== Props & Events ====================
// 🗑️ 已删除不必要的 emit 定义

// 跳转到指定日期
function goToDate(dateStr: string) {
  logger.info(LogTags.COMPONENT_KANBAN, 'Jumping to date', { dateStr })

  if (!scrollContainer.value) {
    logger.warn(LogTags.COMPONENT_KANBAN, 'Scroll container not ready')
    return
  }

  try {
    const targetDate = new Date(dateStr)
    const today = new Date()
    today.setHours(0, 0, 0, 0)
    targetDate.setHours(0, 0, 0, 0)

    // 计算目标日期与今天的天数差
    const daysDiff = Math.floor((targetDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24))

    logger.debug(LogTags.COMPONENT_KANBAN, 'Calculated date offset', {
      daysDiff,
      targetDate: dateStr,
    })

    // 重新生成看板列表，让目标日期在可见区的第一个位置（索引 BUFFER_SIZE）
    const newKanbans: DailyKanban[] = []
    for (let i = 0; i < TOTAL_KANBANS; i++) {
      const offsetFromTarget = i - BUFFER_SIZE // 索引 BUFFER_SIZE 对应目标日期
      const date = addDays(targetDate, offsetFromTarget)
      const dateStrFormatted = formatDate(date)
      newKanbans.push({
        id: dateStrFormatted,
        date: date,
        viewKey: `daily::${dateStrFormatted}`,
        offset: daysDiff + offsetFromTarget, // 相对于今天的偏移
      })
    }

    kanbans.value = newKanbans

    // 滚动到目标位置（让目标日期显示在可见区左侧）
    nextTick(() => {
      if (scrollContainer.value) {
        scrollContainer.value.scrollLeft = BUFFER_SIZE * KANBAN_TOTAL_WIDTH_PX
        logger.info(LogTags.COMPONENT_KANBAN, 'Jumped to date successfully', {
          dateStr,
          scrollLeft: scrollContainer.value.scrollLeft,
        })
      }
    })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN,
      'Failed to jump to date',
      error instanceof Error ? error : new Error(String(error)),
      { dateStr }
    )
  }
}

// ==================== Props & Events ====================
const emit = defineEmits<{
  'date-click': [date: string] // 日期点击事件
  'calendar-date-visibility-change': [isVisible: boolean] // 🆕 日历当前显示的日期是否在可见区域
}>()

// ==================== 事件处理 ====================
// 处理看板标题点击
function handleKanbanTitleClick(date: string) {
  logger.debug(LogTags.COMPONENT_KANBAN, 'Kanban title clicked', { date })
  emit('date-click', date)
}

// ==================== 暴露属性和方法给父组件 ====================
defineExpose({
  kanbanCount: computed(() => kanbans.value.length),
  goToDate, // 暴露跳转方法
})

// 🗑️ 移除 handleOpenEditor - SimpleKanbanColumn 和 KanbanTaskCard 直接调用 UI Store
// 🗑️ 移除不再需要的事件处理器（SimpleKanbanColumn 内部处理）：
// function handleAddTask() { ... }
// async function handleReorder() { ... }

// ==================== 拖动滚动 ====================
function handleMouseDown(event: MouseEvent) {
  // 只处理左键
  if (event.button !== 0) return

  // ✅ 核心修复：检测鼠标是否在任务卡片上
  const target = event.target as HTMLElement

  // 如果点击的是任务卡片或其内部元素，不启动看板拖动
  if (target.closest('.task-card-wrapper')) {
    return
  }

  // 如果点击的是其他可交互元素（输入框、按钮等），也不启动看板拖动
  if (
    target.closest('input') ||
    target.closest('button') ||
    target.closest('textarea') ||
    target.closest('select')
  ) {
    return
  }

  // ✅ 额外检查：如果任务卡片已经在拖动中（防抖阈值期间），也不启动看板拖动
  if (isTaskDragging.value) {
    return
  }

  isDragging.value = true
  dragStartX.value = event.pageX
  dragStartScrollLeft.value = scrollContainer.value?.scrollLeft || 0

  // 改变光标样式：按下时显示grab
  if (scrollContainer.value) {
    scrollContainer.value.style.cursor = 'grab'
    scrollContainer.value.style.userSelect = 'none'
  }
}

function handleMouseMove(event: MouseEvent) {
  // ✅ 如果任务正在拖动（通过 interact.js），立即停止看板拖动
  if (isTaskDragging.value && isDragging.value) {
    handleMouseUp()
    return
  }

  if (!isDragging.value || !scrollContainer.value) return

  event.preventDefault()

  // 开始拖动时，改变光标为grabbing
  if (scrollContainer.value.style.cursor !== 'grabbing') {
    scrollContainer.value.style.cursor = 'grabbing'
  }

  const deltaX = event.pageX - dragStartX.value
  scrollContainer.value.scrollLeft = dragStartScrollLeft.value - deltaX
}

function handleMouseUp() {
  if (!isDragging.value) return

  isDragging.value = false

  // 恢复光标样式为pointer
  if (scrollContainer.value) {
    scrollContainer.value.style.cursor = 'pointer'
    scrollContainer.value.style.userSelect = ''
  }
}

function handleMouseLeave() {
  if (isDragging.value) {
    handleMouseUp()
  }
}

// ==================== 滚动监控与自动加载 ====================
let monitorInterval: number | null = null
let lastCalendarDateVisibility: boolean | null = null // 🆕 记录上次日历日期的可见状态

function startScrollMonitor() {
  if (monitorInterval) return

  monitorInterval = window.setInterval(() => {
    if (!scrollContainer.value || isScrolling.value) return

    const scrollLeft = scrollContainer.value.scrollLeft
    const containerWidth = scrollContainer.value.offsetWidth
    // ✅ 总宽度 = 左padding + (看板数量 * 看板总宽度) + 右padding（gap=0无需减）
    const totalWidth = TRACK_PADDING_PX + TOTAL_KANBANS * KANBAN_TOTAL_WIDTH_PX + TRACK_PADDING_PX
    const maxScrollLeft = totalWidth - containerWidth

    // 🆕 检测日历当前显示的日期对应的看板是否在可见区域
    const currentCalendarDate = registerStore.readRegister<string>(
      registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME
    )

    if (currentCalendarDate) {
      // 查找日历当前显示日期对应的看板
      const calendarDateKanban = kanbans.value.find(
        (k) => formatDate(k.date) === currentCalendarDate
      )

      if (calendarDateKanban) {
        const kanbanIndex = kanbans.value.indexOf(calendarDateKanban)
        const kanbanLeftPosition = TRACK_PADDING_PX + kanbanIndex * KANBAN_TOTAL_WIDTH_PX
        const kanbanRightPosition = kanbanLeftPosition + KANBAN_WIDTH_PX

        // 判断该看板是否在可见区域内
        const isCalendarDateVisible =
          kanbanLeftPosition < scrollLeft + containerWidth && kanbanRightPosition > scrollLeft

        // 只在可见性发生变化时发出事件
        if (lastCalendarDateVisibility !== isCalendarDateVisible) {
          lastCalendarDateVisibility = isCalendarDateVisible
          emit('calendar-date-visibility-change', isCalendarDateVisible)
          logger.debug(LogTags.COMPONENT_KANBAN, 'Calendar date visibility changed', {
            date: currentCalendarDate,
            isVisible: isCalendarDateVisible,
          })
        }
      }
    }

    // 触发阈值计算：
    // 左触发点：当滚动位置 < (BUFFER_SIZE - TRIGGER_DISTANCE) * KANBAN_TOTAL_WIDTH_PX
    //   例如：当 scrollLeft < 960px 时触发（还剩4个左缓冲看板）
    //
    // 右触发点：当滚动位置 > maxScrollLeft - (BUFFER_SIZE - TRIGGER_DISTANCE) * KANBAN_TOTAL_WIDTH_PX
    //   例如：当 scrollLeft > (maxScrollLeft - 960px) 时触发（还剩4个右缓冲看板）
    const leftTrigger = (BUFFER_SIZE - TRIGGER_DISTANCE) * KANBAN_TOTAL_WIDTH_PX
    const rightTrigger = maxScrollLeft - (BUFFER_SIZE - TRIGGER_DISTANCE) * KANBAN_TOTAL_WIDTH_PX

    // 调试日志（每次检查都输出）
    // console.log('[InfiniteDailyKanban] 🔍 Monitor:', {
    //   scrollLeft: scrollLeft.toFixed(0),
    //   maxScrollLeft: maxScrollLeft.toFixed(0),
    //   leftTrigger: leftTrigger.toFixed(0),
    //   rightTrigger: rightTrigger.toFixed(0),
    //   distanceToLeft: (scrollLeft - leftTrigger).toFixed(0),
    //   distanceToRight: (rightTrigger - scrollLeft).toFixed(0),
    //   overflowLeftPx: (leftTrigger - scrollLeft).toFixed(0),
    //   overflowRightPx: (scrollLeft - rightTrigger).toFixed(0),
    //   isScrolling: isScrolling.value,
    //   kanbanRange: `${kanbans.value[0]?.id} ~ ${kanbans.value[kanbans.value.length - 1]?.id}`,
    // })

    // 触发批量 shift 操作（一次性计算步数并执行）
    const overflowLeftPx = leftTrigger - scrollLeft
    const overflowRightPx = scrollLeft - rightTrigger

    if (overflowLeftPx > 0) {
      const steps = Math.ceil(overflowLeftPx / KANBAN_TOTAL_WIDTH_PX)
      // console.log('[InfiniteDailyKanban] 🎯 BATCH LEFT shift steps:', steps)
      shiftKanbansBatch('left', steps)
    } else if (overflowRightPx > 0) {
      const steps = Math.ceil(overflowRightPx / KANBAN_TOTAL_WIDTH_PX)
      // console.log('[InfiniteDailyKanban] 🎯 BATCH RIGHT shift steps:', steps)
      shiftKanbansBatch('right', steps)
    }
  }, 500) // 每100ms检查一次，快速响应
}

function stopScrollMonitor() {
  if (monitorInterval) {
    clearInterval(monitorInterval)
    monitorInterval = null
  }
}

// ==================== 任务卡片拖动监听 ====================
// ✅ 不再需要手动监听拖动事件，使用 interact.js 的全局状态 (controllerDebugState)

// ==================== 生命周期 ====================
onMounted(async () => {
  logger.info(LogTags.COMPONENT_KANBAN, 'Initializing daily kanbans')
  // 初始化日期看板
  initKanbans()

  // ✅ 批量加载所有看板的view preferences（防抖优化）
  const viewKeys = kanbans.value.map((k) => k.viewKey)
  await viewStore.batchFetchViewPreferences(viewKeys)

  // ✅ 无需手动加载任务，getKanbanTasks 会自动从 TaskStore 获取（响应式）

  // 启动滚动监控
  startScrollMonitor()

  // ✅ 不再需要手动监听拖动事件，interact.js 通过 controllerDebugState 自动同步状态
})

onBeforeUnmount(() => {
  stopScrollMonitor()
})
</script>

<template>
  <div
    ref="scrollContainer"
    class="kanban-scroll-container"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseLeave"
  >
    <div class="kanban-track" :style="{ width: `${TOTAL_KANBANS * KANBAN_WIDTH}rem` }">
      <SimpleKanbanColumn
        v-for="kanban in kanbans"
        :key="kanban.id"
        :title="kanban.id"
        :subtitle="`${getWeekdayName(kanban.date)}${isToday(kanban.date) ? ' · 今天' : ''}`"
        :view-key="kanban.viewKey"
        :view-metadata="getKanbanMetadata(kanban)"
        :show-add-input="true"
        :is-expired="isExpired(kanban.date)"
        :is-calendar-date="isCalendarDate(kanban.date)"
        :style="{ width: `${KANBAN_WIDTH}rem`, flexShrink: 0 }"
        @title-click="handleKanbanTitleClick"
      />
    </div>
  </div>
</template>

<style scoped>
.kanban-scroll-container {
  overflow: auto hidden;
  height: 100%;
  width: 100%;

  /* 关键：防止内容撑破容器 */
  min-width: 0;
  min-height: 0;

  /* 拖动滚动样式 */
  cursor: pointer;
  user-select: none;
}

.kanban-track {
  display: flex;
  gap: 0; /* ✅ gap 设为 0，由看板自身 padding 填补 */
  height: 100%;
  padding: 0 1rem;

  /* width 动态计算，始终恒定 */
}
</style>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed, nextTick } from 'vue'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata, DateViewConfig } from '@/types/drag'
import SimpleKanbanColumn from '@/components/parts/kanban/SimpleKanbanColumn.vue'
import { useTaskStore } from '@/stores/task'
import { useViewStore } from '@/stores/view'
import { useDragTransfer } from '@/composables/drag'

// ==================== Stores ====================
const taskStore = useTaskStore()
const viewStore = useViewStore()
const dragTransfer = useDragTransfer()

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

// 拖动滚动状态
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartScrollLeft = ref(0)

// 任务卡片拖动状态（用于禁用看板拖动）
const isTaskDragging = ref(false)

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
  const today = new Date()
  return (
    date.getFullYear() === today.getFullYear() &&
    date.getMonth() === today.getMonth() &&
    date.getDate() === today.getDate()
  )
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

      // 发送初始可见日期
      const initialVisibleDate = calculateVisibleLeftmostDate()
      if (initialVisibleDate) {
        emit('visible-date-change', initialVisibleDate)
        // console.log('[InfiniteDailyKanban] 📅 Initial visible date:', initialVisibleDate)
      }
    }
  })
}

// 向右滚动：在右侧添加未来日期，在左侧移除过去日期
function shiftKanbansRight() {
  if (isScrolling.value || kanbans.value.length === 0) return

  // console.log('[InfiniteDailyKanban] ➡️ Shifting kanbans right (adding future, removing past)')
  isScrolling.value = true

  const currentScrollLeft = scrollContainer.value?.scrollLeft || 0

  // 移除最左侧的看板（用户看不到的区域）
  kanbans.value.shift()

  // 在右侧添加新看板（未来日期）
  const lastKanban = kanbans.value[kanbans.value.length - 1]
  if (!lastKanban) return

  const newDate = addDays(lastKanban.date, 1)
  const dateStr = formatDate(newDate)
  kanbans.value.push({
    id: dateStr,
    date: newDate,
    viewKey: `daily::${dateStr}`,
    offset: lastKanban.offset + 1,
  })

  // ✅ 无需手动加载任务，getKanbanTasks 会自动从 TaskStore 获取

  // console.log('[InfiniteDailyKanban] ✅ New kanban added:', dateStr)

  // 调整滚动位置：因为左侧移除了一个看板，需要减少scrollLeft以保持视窗不变
  // 使用同步方式立即调整，避免视觉闪烁
  if (scrollContainer.value) {
    // 临时禁用滚动动画，确保瞬间完成
    const originalBehavior = scrollContainer.value.style.scrollBehavior
    scrollContainer.value.style.scrollBehavior = 'auto'

    // 在 nextTick 中调整位置（等待 DOM 更新）
    nextTick(() => {
      if (scrollContainer.value) {
        scrollContainer.value.scrollLeft = currentScrollLeft - KANBAN_TOTAL_WIDTH_PX
        // console.log('[InfiniteDailyKanban] 📍 Adjusted scroll (removed left):', {
        //   before: currentScrollLeft,
        //   after: scrollContainer.value.scrollLeft,
        // })

        // 恢复原始滚动行为
        scrollContainer.value.style.scrollBehavior = originalBehavior

        // 锁定时间：防止在补偿期间重复触发shift
        setTimeout(() => {
          isScrolling.value = false
        }, 150)
      }
    })
  }
}

// 向左滚动：在左侧添加过去日期，在右侧移除未来日期
function shiftKanbansLeft() {
  if (isScrolling.value || kanbans.value.length === 0) return

  // console.log('[InfiniteDailyKanban] ⬅️ Shifting kanbans left (adding past, removing future)')
  isScrolling.value = true

  const currentScrollLeft = scrollContainer.value?.scrollLeft || 0

  // 移除最右侧的看板（用户看不到的区域）
  kanbans.value.pop()

  // 在左侧添加新看板（过去日期）
  const firstKanban = kanbans.value[0]
  if (!firstKanban) return

  const newDate = addDays(firstKanban.date, -1)
  const dateStr = formatDate(newDate)
  kanbans.value.unshift({
    id: dateStr,
    date: newDate,
    viewKey: `daily::${dateStr}`,
    offset: firstKanban.offset - 1,
  })

  // ✅ 无需手动加载任务，getKanbanTasks 会自动从 TaskStore 获取

  // console.log('[InfiniteDailyKanban] ✅ New kanban added:', dateStr)

  // 调整滚动位置：因为左侧添加了一个看板，需要增加scrollLeft以保持视窗不变
  // 使用同步方式立即调整，避免视觉闪烁
  if (scrollContainer.value) {
    // 临时禁用滚动动画，确保瞬间完成
    const originalBehavior = scrollContainer.value.style.scrollBehavior
    scrollContainer.value.style.scrollBehavior = 'auto'

    // 在 nextTick 中调整位置（等待 DOM 更新）
    nextTick(() => {
      if (scrollContainer.value) {
        scrollContainer.value.scrollLeft = currentScrollLeft + KANBAN_TOTAL_WIDTH_PX
        // console.log('[InfiniteDailyKanban] 📍 Adjusted scroll (added left):', {
        //   before: currentScrollLeft,
        //   after: scrollContainer.value.scrollLeft,
        // })

        // 恢复原始滚动行为
        scrollContainer.value.style.scrollBehavior = originalBehavior

        // 锁定时间：防止在补偿期间重复触发shift
        setTimeout(() => {
          isScrolling.value = false
        }, 150)
      }
    })
  }
}

// 计算可见区域最左边的看板日期（露出一半才算可见）
function calculateVisibleLeftmostDate(): string | null {
  if (!scrollContainer.value || kanbans.value.length === 0) return null

  const scrollLeft = scrollContainer.value.scrollLeft
  const containerWidth = scrollContainer.value.offsetWidth

  // 遍历所有看板，找到第一个露出至少一半的看板
  for (let i = 0; i < kanbans.value.length; i++) {
    const kanban = kanbans.value[i]
    if (!kanban) continue

    // 计算看板在 track 中的绝对位置（考虑 padding 和 gap）
    // 第 i 个看板的左边距 = track的左padding + i * (看板宽度 + gap)
    const kanbanAbsoluteLeft = TRACK_PADDING_PX + i * KANBAN_TOTAL_WIDTH_PX

    // 计算看板在可见区域的相对位置
    const kanbanRelativeLeft = kanbanAbsoluteLeft - scrollLeft

    // 计算看板中心点的相对位置
    const kanbanCenter = kanbanRelativeLeft + KANBAN_WIDTH_PX / 2

    // 如果看板中心点在可见区域内（0 到 containerWidth 之间），说明露出了至少一半
    if (kanbanCenter >= 0 && kanbanCenter < containerWidth) {
      // console.log(
      //   `[InfiniteDailyKanban] 📍 Visible leftmost: ${kanban.id} (center at ${kanbanCenter.toFixed(0)}px)`
      // )
      return kanban.id
    }
  }

  return null
}

// 滚动事件处理
function handleScroll(_event: Event) {
  // 计算并发送可见日期变化事件
  const visibleDate = calculateVisibleLeftmostDate()
  if (visibleDate) {
    emit('visible-date-change', visibleDate)
  }
}

// 为每个看板获取任务（响应式）
// ✅ 使用computed缓存，避免重复计算
const kanbanTasksMap = computed(() => {
  console.log('[InfiniteDailyKanban] 🔄 Recomputing all kanban tasks')
  const map = new Map<string, TaskCard[]>()

  kanbans.value.forEach((kanban) => {
    const tasks = taskStore.getTasksByDate(kanban.id)
    const sorted = viewStore.applySorting(tasks, kanban.viewKey)
    map.set(kanban.viewKey, sorted)
    console.log(`[InfiniteDailyKanban] Cached ${sorted.length} tasks for ${kanban.id}`)
  })

  return map
})

// 获取缓存的任务列表
function getKanbanTasks(kanban: DailyKanban): TaskCard[] {
  return kanbanTasksMap.value.get(kanban.viewKey) ?? []
}

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
const emit = defineEmits<{
  'open-editor': [task: TaskCard]
  'add-task': [title: string, date: string]
  'visible-date-change': [date: string] // 可见日期变化事件
}>()

// 暴露属性给父组件
defineExpose({
  kanbanCount: computed(() => kanbans.value.length),
})

function handleOpenEditor(task: TaskCard) {
  emit('open-editor', task)
}

function handleAddTask(title: string, kanban: DailyKanban) {
  emit('add-task', title, kanban.id)
}

async function handleReorder(viewKey: string, newOrder: string[]) {
  console.log('[InfiniteDailyKanban] 🔄 Reorder:', viewKey, newOrder)
  await viewStore.updateSorting(viewKey, newOrder)
}

// ==================== 拖动滚动 ====================
function handleMouseDown(event: MouseEvent) {
  // 只处理左键
  if (event.button !== 0) return

  // ✅ 关键修复：如果任务卡片正在拖动，完全禁用看板拖动
  if (isTaskDragging.value) {
    return
  }

  // 如果点击的是看板内部元素（比如任务卡片），不启用拖动
  const target = event.target as HTMLElement
  if (target.closest('.simple-kanban-column')) {
    // 如果点击的是看板列本身内部的可交互元素，跳过
    if (
      target.closest('.kanban-card') ||
      target.closest('input') ||
      target.closest('button') ||
      target.closest('[draggable="true"]') // ✅ 检测所有可拖动元素
    ) {
      return
    }
  }

  isDragging.value = true
  dragStartX.value = event.pageX
  dragStartScrollLeft.value = scrollContainer.value?.scrollLeft || 0

  // 改变光标样式
  if (scrollContainer.value) {
    scrollContainer.value.style.cursor = 'grabbing'
    scrollContainer.value.style.userSelect = 'none'
  }
}

function handleMouseMove(event: MouseEvent) {
  if (!isDragging.value || !scrollContainer.value) return

  event.preventDefault()

  const deltaX = event.pageX - dragStartX.value
  scrollContainer.value.scrollLeft = dragStartScrollLeft.value - deltaX
}

function handleMouseUp() {
  if (!isDragging.value) return

  isDragging.value = false

  // 恢复光标样式
  if (scrollContainer.value) {
    scrollContainer.value.style.cursor = 'grab'
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

function startScrollMonitor() {
  if (monitorInterval) return

  monitorInterval = window.setInterval(() => {
    if (!scrollContainer.value || isScrolling.value) return

    const scrollLeft = scrollContainer.value.scrollLeft
    const containerWidth = scrollContainer.value.offsetWidth
    // ✅ 总宽度 = 左padding + (看板数量 * 看板总宽度) + 右padding（gap=0无需减）
    const totalWidth = TRACK_PADDING_PX + TOTAL_KANBANS * KANBAN_TOTAL_WIDTH_PX + TRACK_PADDING_PX
    const maxScrollLeft = totalWidth - containerWidth

    // 触发阈值计算：
    // 左触发点：当滚动位置 < (BUFFER_SIZE - TRIGGER_DISTANCE) * KANBAN_TOTAL_WIDTH_PX
    //   例如：当 scrollLeft < 960px 时触发（还剩4个左缓冲看板）
    //
    // 右触发点：当滚动位置 > maxScrollLeft - (BUFFER_SIZE - TRIGGER_DISTANCE) * KANBAN_TOTAL_WIDTH_PX
    //   例如：当 scrollLeft > (maxScrollLeft - 960px) 时触发（还剩4个右缓冲看板）
    const leftTrigger = (BUFFER_SIZE - TRIGGER_DISTANCE) * KANBAN_TOTAL_WIDTH_PX
    const rightTrigger = maxScrollLeft - (BUFFER_SIZE - TRIGGER_DISTANCE) * KANBAN_TOTAL_WIDTH_PX

    const shouldShiftLeft = scrollLeft < leftTrigger
    const shouldShiftRight = scrollLeft > rightTrigger

    // 调试日志（每次检查都输出）
    // console.log('[InfiniteDailyKanban] 🔍 Monitor:', {
    //   scrollLeft: scrollLeft.toFixed(0),
    //   maxScrollLeft: maxScrollLeft.toFixed(0),
    //   leftTrigger: leftTrigger.toFixed(0),
    //   rightTrigger: rightTrigger.toFixed(0),
    //   distanceToLeft: (scrollLeft - leftTrigger).toFixed(0),
    //   distanceToRight: (rightTrigger - scrollLeft).toFixed(0),
    //   shouldShiftLeft,
    //   shouldShiftRight,
    //   isScrolling: isScrolling.value,
    //   kanbanRange: `${kanbans.value[0]?.id} ~ ${kanbans.value[kanbans.value.length - 1]?.id}`,
    // })

    // 触发shift操作
    if (shouldShiftLeft) {
      // console.log('[InfiniteDailyKanban] 🎯 Triggering LEFT shift (entering left buffer zone)')
      shiftKanbansLeft()
    } else if (shouldShiftRight) {
      // console.log('[InfiniteDailyKanban] 🎯 Triggering RIGHT shift (entering right buffer zone)')
      shiftKanbansRight()
    }
  }, 100) // 每100ms检查一次，快速响应
}

function stopScrollMonitor() {
  if (monitorInterval) {
    clearInterval(monitorInterval)
    monitorInterval = null
  }
}

// ==================== 任务卡片拖动监听 ====================
// 监听任务卡片的拖动开始和结束，以禁用/启用看板拖动
function handleTaskDragStart(event: DragEvent) {
  // 检查是否是任务卡片拖动（使用统一的 dragTransfer 检测）
  if (dragTransfer.hasDragData(event)) {
    isTaskDragging.value = true
    console.log('[InfiniteDailyKanban] 🎯 Task drag started, disabling kanban drag')
  }
}

function handleTaskDragEnd() {
  isTaskDragging.value = false
  console.log('[InfiniteDailyKanban] 🎯 Task drag ended, enabling kanban drag')
}

// ==================== 生命周期 ====================
onMounted(async () => {
  console.log('[InfiniteDailyKanban] 🚀 Initializing daily kanbans...')
  // 初始化日期看板
  initKanbans()

  // ✅ 批量加载所有看板的view preferences（防抖优化）
  const viewKeys = kanbans.value.map((k) => k.viewKey)
  await viewStore.batchFetchViewPreferences(viewKeys)

  // ✅ 无需手动加载任务，getKanbanTasks 会自动从 TaskStore 获取（响应式）

  // 启动滚动监控
  startScrollMonitor()

  // 监听任务卡片拖动事件
  document.addEventListener('dragstart', handleTaskDragStart)
  document.addEventListener('dragend', handleTaskDragEnd)

  // 🆕 兜底：当全局 drop 发生时，确保恢复看板拖动能力
  document.addEventListener(
    'drop',
    () => {
      if (isTaskDragging.value) {
        isTaskDragging.value = false
        console.log('[InfiniteDailyKanban] ♻️ Global drop detected, re-enable kanban drag')
      }
    },
    true
  )
})

onBeforeUnmount(() => {
  stopScrollMonitor()

  // 清理事件监听器
  document.removeEventListener('dragstart', handleTaskDragStart)
  document.removeEventListener('dragend', handleTaskDragEnd)
})
</script>

<template>
  <div
    ref="scrollContainer"
    class="kanban-scroll-container"
    @scroll="handleScroll"
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
        :tasks="getKanbanTasks(kanban)"
        :show-add-input="true"
        :style="{ width: `${KANBAN_WIDTH}rem`, flexShrink: 0 }"
        @open-editor="handleOpenEditor"
        @add-task="(title) => handleAddTask(title, kanban)"
        @reorder-tasks="(order) => handleReorder(kanban.viewKey, order)"
        @cross-view-drop="
          (taskId, targetViewId) => console.log('📦 Cross-view drop:', taskId, 'to', targetViewId)
        "
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
  cursor: grab;
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

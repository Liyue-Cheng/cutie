<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue'
import TimelineCard from '@/components/parts/timeline/TimelineCard.vue'
import TimelineDivider from '@/components/parts/timeline/TimelineDivider.vue'
import { useTaskStore } from '@/stores/task'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import { logger, LogTags } from '@/infra/logging/logger'

// ==================== Stores ====================
const taskStore = useTaskStore()

// ==================== 配置常量 ====================
const INITIAL_PAST_DAYS = 10 // 初始加载过去10天
const INITIAL_FUTURE_DAYS = 30 // 初始加载未来30天
const LOAD_MORE_DAYS = 7 // 每次加载更多时增加的天数
const TRIGGER_THRESHOLD = 800 // 触发加载的距离阈值（px）

// ==================== 状态 ====================
const scrollContainer = ref<HTMLElement | null>(null)
const timelineTrack = ref<HTMLElement | null>(null)
const isLoadingMore = ref(false)

// ==================== 时间卡片数据 ====================
interface TimelineCardData {
  id: string // 日期字符串 YYYY-MM-DD
  date: Date
  dateString: string // YYYY-MM-DD
  isToday: boolean
  isPast: boolean
}

const cards = ref<TimelineCardData[]>([])
const earliestDate = ref<Date>(new Date())
const latestDate = ref<Date>(new Date())

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

// 创建卡片数据
function createCardData(date: Date, todayString: string): TimelineCardData {
  const dateStr = formatDate(date)
  return {
    id: dateStr,
    date: date,
    dateString: dateStr,
    isToday: dateStr === todayString,
    isPast: dateStr < todayString,
  }
}

// 初始化卡片
function initCards() {
  const today = new Date()
  const todayString = getTodayDateString()

  const initialCards: TimelineCardData[] = []

  // 加载过去的日期
  for (let i = INITIAL_PAST_DAYS; i >= 1; i--) {
    const date = addDays(today, -i)
    initialCards.push(createCardData(date, todayString))
  }

  // 加载今天
  initialCards.push(createCardData(today, todayString))

  // 加载未来的日期
  for (let i = 1; i <= INITIAL_FUTURE_DAYS; i++) {
    const date = addDays(today, i)
    initialCards.push(createCardData(date, todayString))
  }

  cards.value = initialCards
  earliestDate.value = addDays(today, -INITIAL_PAST_DAYS)
  latestDate.value = addDays(today, INITIAL_FUTURE_DAYS)

  logger.debug(LogTags.COMPONENT_TIMELINE, 'Timeline cards initialized', {
    total: cards.value.length,
    earliest: formatDate(earliestDate.value),
    latest: formatDate(latestDate.value),
  })
}

// 向过去方向加载更多
function loadMorePast() {
  if (isLoadingMore.value) return

  isLoadingMore.value = true
  const todayString = getTodayDateString()
  const newCards: TimelineCardData[] = []

  // 保存当前第一个卡片的位置，用于滚动补偿
  const firstCard = timelineTrack.value?.firstElementChild as HTMLElement
  const firstCardOffsetTop = firstCard?.offsetTop || 0

  for (let i = LOAD_MORE_DAYS; i >= 1; i--) {
    const date = addDays(earliestDate.value, -i)
    newCards.push(createCardData(date, todayString))
  }

  cards.value = [...newCards, ...cards.value]
  earliestDate.value = addDays(earliestDate.value, -LOAD_MORE_DAYS)

  logger.debug(LogTags.COMPONENT_TIMELINE, 'Loaded more past days', {
    count: LOAD_MORE_DAYS,
    newEarliest: formatDate(earliestDate.value),
  })

  // 滚动补偿：保持用户看到的内容位置不变
  requestAnimationFrame(() => {
    if (firstCard && scrollContainer.value) {
      const newOffsetTop = firstCard.offsetTop
      const offset = newOffsetTop - firstCardOffsetTop
      scrollContainer.value.scrollTop += offset
    }
    isLoadingMore.value = false
  })
}

// 向未来方向加载更多
function loadMoreFuture() {
  if (isLoadingMore.value) return

  isLoadingMore.value = true
  const todayString = getTodayDateString()

  for (let i = 1; i <= LOAD_MORE_DAYS; i++) {
    const date = addDays(latestDate.value, i)
    cards.value.push(createCardData(date, todayString))
  }

  latestDate.value = addDays(latestDate.value, LOAD_MORE_DAYS)

  logger.debug(LogTags.COMPONENT_TIMELINE, 'Loaded more future days', {
    count: LOAD_MORE_DAYS,
    newLatest: formatDate(latestDate.value),
  })

  isLoadingMore.value = false
}

// 获取指定日期的任务
function getTasksForDate(dateString: string) {
  return taskStore.getTasksByDate_Mux(dateString)
}

// 判断是否应该在当前卡片后显示分割线
function shouldShowDivider(currentCard: TimelineCardData, index: number): boolean {
  if (!currentCard.isPast) {
    return false
  }

  const nextCard = cards.value[index + 1]
  if (!nextCard) {
    return false
  }

  return currentCard.isPast && !nextCard.isPast
}

// ==================== 滚动监控 ====================
function handleScroll() {
  if (!scrollContainer.value || isLoadingMore.value) return

  const scrollTop = scrollContainer.value.scrollTop
  const scrollHeight = scrollContainer.value.scrollHeight
  const clientHeight = scrollContainer.value.clientHeight

  // 接近顶部，加载更早的日期
  if (scrollTop < TRIGGER_THRESHOLD) {
    loadMorePast()
  }

  // 接近底部，加载更晚的日期
  if (scrollTop + clientHeight > scrollHeight - TRIGGER_THRESHOLD) {
    loadMoreFuture()
  }
}

// 滚动到今天
function scrollToToday() {
  if (!scrollContainer.value || !timelineTrack.value) return

  const todayCardIndex = cards.value.findIndex((card) => card.isToday)
  if (todayCardIndex === -1) return

  const todayCard = timelineTrack.value.children[todayCardIndex * 2] as HTMLElement // *2 因为有divider
  if (todayCard) {
    todayCard.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

// ==================== 生命周期 ====================
onMounted(async () => {
  logger.info(LogTags.COMPONENT_TIMELINE, 'Initializing timeline')

  // 初始化时间卡片
  initCards()

  // 确保任务已加载
  await taskStore.fetchAllIncompleteTasks_DMA()

  // 绑定滚动事件
  if (scrollContainer.value) {
    scrollContainer.value.addEventListener('scroll', handleScroll, { passive: true })
  }

  // 初始滚动到今天
  setTimeout(() => {
    scrollToToday()
  }, 100)

  logger.info(LogTags.COMPONENT_TIMELINE, 'Timeline initialized', {
    cardCount: cards.value.length,
    taskCount: taskStore.allTasks.length,
  })
})

onBeforeUnmount(() => {
  if (scrollContainer.value) {
    scrollContainer.value.removeEventListener('scroll', handleScroll)
  }
})
</script>

<template>
  <div ref="scrollContainer" class="infinite-timeline-container">
    <div ref="timelineTrack" class="timeline-track">
      <template v-for="(card, index) in cards" :key="`card-${card.id}`">
        <!-- 时间卡片 - 移除固定高度，让内容自然撑开 -->
        <TimelineCard
          :date="card.dateString"
          :tasks="getTasksForDate(card.dateString)"
          :is-today="card.isToday"
          :is-past="card.isPast"
        />

        <!-- 在过去日期和今天/未来日期之间插入分割线 -->
        <TimelineDivider v-if="shouldShowDivider(card, index)" :key="`divider-${card.id}`" />
      </template>

      <!-- 加载提示 -->
      <div v-if="isLoadingMore" class="loading-indicator">
        <span>加载中...</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.infinite-timeline-container {
  overflow: hidden auto;
  height: 100%;
  width: 100%;
  padding: 0 0.8rem;
  min-width: 0;
  min-height: 0;

  /* 隐藏滚动条但保留滚动功能 */
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE/Edge */
}

/* Webkit 浏览器（Chrome, Safari, Edge）隐藏滚动条 */
.infinite-timeline-container::-webkit-scrollbar {
  display: none;
}

.timeline-track {
  display: flex;
  flex-direction: column;
  gap: 0;
  width: 100%;
  padding: 0.8rem 0;
}

.loading-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.6rem;
  color: var(--color-text-tertiary);
  font-size: 1.2rem;
}
</style>

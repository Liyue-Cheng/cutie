<template>
  <div class="daily-planning-wizard">
    <!-- 标题区域 -->
    <div class="wizard-header">
      <h1 class="wizard-title">{{ currentStepConfig.title }}</h1>
      <p class="wizard-subtitle">{{ currentStepConfig.subtitle }}</p>
    </div>

    <!-- 内容区域 -->
    <div class="wizard-content">
      <!-- 饼状图：按 Area 分组的任务统计 -->
      <div class="chart-section">
        <!-- 图表容器：带发光效果 -->
        <div class="chart-wrapper">
          <div class="chart-glow"></div>
          <div class="chart-container">
            <VChart class="donut-chart" :option="chartOption" autoresize />
          </div>
        </div>

        <!-- 精致图例 -->
        <div class="chart-legend">
          <TransitionGroup name="legend-fade">
            <div
              v-for="(item, index) in areaStats"
              :key="item.id"
              class="legend-item"
              :style="{ '--delay': `${index * 50}ms` }"
            >
              <CuteIcon name="Hash" size="1.65rem" :color="item.color" class="legend-icon" />
              <span class="legend-name">{{ item.name }}</span>
              <span class="legend-count">{{
                isStage2 ? formatMinutesCompact(item.value) : item.value
              }}</span>
            </div>
          </TransitionGroup>
          <div v-if="areaStats.length === 0" class="legend-empty">
            <span class="empty-icon">○</span>
            <span>{{ $t('task.label.noTasks') }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部导航 -->
    <div class="wizard-navigation">
      <button v-if="step === 2" class="nav-btn back-btn" @click="handleBack">
        <span class="back-icon">←</span>
      </button>
      <button class="nav-btn next-btn" @click="handleNext">
        {{ step === 1 ? $t('view.dailyPlanning.next') : $t('view.dailyPlanning.done') }}
      </button>
    </div>

    <!-- 底部提示 -->
    <div class="wizard-hint">
      <p class="hint-text">{{ currentStepConfig.hint }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { PieChart } from 'echarts/charts'
import { TooltipComponent, GraphicComponent } from 'echarts/components'
import type { ECBasicOption } from 'echarts/types/dist/shared'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import type { TaskCard } from '@/types/dtos'
import { getTodayDateString } from '@/infra/utils/dateUtils'

use([CanvasRenderer, PieChart, TooltipComponent, GraphicComponent])

const { t } = useI18n()
const taskStore = useTaskStore()
const areaStore = useAreaStore()

// ECharts 是 Canvas 渲染：不能直接使用 `var(--css-variable)` 作为颜色值
// 这里在运行时读取真实背景色，避免扇区 borderColor 退化成黑色导致“黑边”
const chartBgColor = ref('#f0f')
const chartTextColor = ref('#f0f')
const chartFontFamily = ref(
  'system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif'
)
onMounted(() => {
  const root = document.documentElement
  const styles = getComputedStyle(root)
  const contentBg = styles.getPropertyValue('--color-background-content').trim()
  const primaryBg = styles.getPropertyValue('--color-background-primary').trim()
  chartBgColor.value = contentBg || primaryBg || '#f0f'

  const textPrimary = styles.getPropertyValue('--color-text-primary').trim()
  chartTextColor.value = textPrimary || '#f0f'

  // Canvas 不支持 fontFamily: 'inherit'，需要注入真实字体栈
  const bodyFont = getComputedStyle(document.body).fontFamily?.trim()
  if (bodyFont) {
    chartFontFamily.value = bodyFont
  }
})

// Props
interface Props {
  step: 1 | 2
}

const props = withDefaults(defineProps<Props>(), {
  step: 1,
})

// Emits
const emit = defineEmits<{
  next: []
  back: []
  done: []
}>()

// 步骤配置
const currentStepConfig = computed(() => {
  if (props.step === 1) {
    return {
      title: t('view.dailyPlanning.step1.title'),
      subtitle: t('view.dailyPlanning.step1.subtitle'),
      hint: t('view.dailyPlanning.step1.hint'),
    }
  } else {
    return {
      title: t('view.dailyPlanning.step2.title'),
      subtitle: t('view.dailyPlanning.step2.subtitle'),
      hint: t('view.dailyPlanning.step2.hint'),
    }
  }
})

// ==================== 饼状图数据 ====================

// 今日任务
const todayTasks = computed(() => {
  const today = getTodayDateString()
  return taskStore.getTasksByDate_Mux(today)
})

const today = computed(() => getTodayDateString())

const isStage2 = computed(() => props.step === 2)

function parseTimeMs(day: string, input: string): number | null {
  // ISO / RFC3339
  if (input.includes('T')) {
    const ms = Date.parse(input)
    return Number.isNaN(ms) ? null : ms
  }

  // HH:MM:SS（按本地时间解释，足够用于计算 duration）
  const ms = Date.parse(`${day}T${input}`)
  return Number.isNaN(ms) ? null : ms
}

function getTimeBlockMinutes(day: string, start: string, end: string): number {
  const startMs = parseTimeMs(day, start)
  const endMs = parseTimeMs(day, end)
  if (startMs === null || endMs === null) return 0
  const diff = endMs - startMs
  if (diff <= 0) return 0
  return Math.round(diff / 60000)
}

function getTaskScheduledMinutesForDay(task: TaskCard, day: string): number {
  const schedule = task.schedules?.find((s) => s.scheduled_day === day)
  if (!schedule) return 0
  if (!Array.isArray(schedule.time_blocks) || schedule.time_blocks.length === 0) return 0

  let minutes = 0
  for (const block of schedule.time_blocks) {
    minutes += getTimeBlockMinutes(day, block.start_time, block.end_time)
  }
  return minutes
}

function formatMinutesCompact(minutes: number): string {
  const mins = Math.max(0, Math.round(minutes))
  const h = Math.floor(mins / 60)
  const m = mins % 60

  // ✅ 24小时制显示：HH:MM（用于“总时长/已安排时长”展示）
  const hh = String(h).padStart(2, '0')
  const mm = String(m).padStart(2, '0')
  return `${hh}:${mm}`
}

const totalScheduledMinutes = computed(() => {
  if (!isStage2.value) return 0
  return areaStats.value.reduce((sum, s) => sum + s.value, 0)
})

// 按 Area 分组统计
interface AreaStats {
  id: string
  name: string
  color: string
  value: number
}

const areaStats = computed<AreaStats[]>(() => {
  const statsMap = new Map<string, number>()
  const noAreaKey = '__no_area__'

  // 统计每个 area 的任务数
  for (const task of todayTasks.value) {
    const areaId = task.area_id || noAreaKey

    if (isStage2.value) {
      const minutes = getTaskScheduledMinutesForDay(task, today.value)
      if (minutes <= 0) continue
      statsMap.set(areaId, (statsMap.get(areaId) || 0) + minutes)
    } else {
      statsMap.set(areaId, (statsMap.get(areaId) || 0) + 1)
    }
  }

  // 转换为数组
  const result: AreaStats[] = []
  for (const [areaId, count] of statsMap) {
    if (areaId === noAreaKey) {
      result.push({
        id: noAreaKey,
        name: t('task.label.noArea'),
        color: '#6b7280',
        value: count,
      })
    } else {
      const area = areaStore.getAreaById(areaId)
      result.push({
        id: areaId,
        name: area?.name || t('task.label.noArea'),
        color: area?.color || '#6b7280',
        value: count,
      })
    }
  }

  // ✅ 稳定顺序：不要按 value 排序，否则数据变化会导致扇区顺序重排，引发“跳边/旋转”的怪动画
  // 使用 name + id 的稳定排序，动画只表现为角度变化而不是位置互换
  return result.sort((a, b) => {
    const byName = a.name.localeCompare(b.name)
    if (byName !== 0) return byName
    return a.id.localeCompare(b.id)
  })
})

// 🔥 数据签名：用于比对数据是否真的变化
let lastDataSignature = ''

/**
 * 生成数据签名（用于比对）
 * 格式：id1:count1,id2:count2,...
 */
function getDataSignature(stats: AreaStats[]): string {
  return stats.map((s) => `${s.id}:${s.value}`).join(',')
}

const centerText = computed(() => {
  return isStage2.value
    ? formatMinutesCompact(totalScheduledMinutes.value)
    : String(todayTasks.value.length)
})

const chartOption = computed<ECBasicOption>(() => {
  const bg = chartBgColor.value
  const empty = areaStats.value.length === 0

  const data = empty
    ? [
        {
          name: t('task.label.noTasks'),
          value: 1,
          itemStyle: {
            color: '#374151',
            borderColor: bg,
            borderWidth: 6,
            borderRadius: 6,
          },
        },
      ]
    : areaStats.value.map((s) => ({
        name: s.name,
        value: s.value,
        itemStyle: {
          color: s.color,
          borderColor: bg,
          borderWidth: 6,
          borderRadius: 6,
        },
      }))

  return {
    animation: true,
    animationDuration: 600,
    animationDurationUpdate: 400,
    animationEasing: 'cubicOut' as const,
    animationEasingUpdate: 'cubicOut' as const,
    tooltip: empty
      ? { show: false }
      : {
          trigger: 'item',
          formatter: (p: any) => {
            const v = typeof p?.value === 'number' ? p.value : 0
            if (isStage2.value) {
              return `${p.name}: ${formatMinutesCompact(v)}`
            }
            return `${p.name}: ${Math.round(v)} ${t('task.count.tasks')}`
          },
        },
    series: [
      {
        type: 'pie',
        radius: ['62%', '86%'],
        center: ['50%', '50%'],
        startAngle: 90,
        clockwise: true,
        avoidLabelOverlap: true,
        label: { show: false },
        labelLine: { show: false },
        emphasis: { scale: true, scaleSize: 6 },
        data,
      },
    ],
    graphic: [
      {
        type: 'text',
        left: 'center',
        top: 'middle',
        style: {
          text: centerText.value,
          fill: chartTextColor.value,
          fontSize: 34,
          fontWeight: 700,
          fontFamily: chartFontFamily.value,
          textAlign: 'center',
          textVerticalAlign: 'middle',
        },
      },
    ],
  }
})

watch(
  areaStats,
  async () => {
    // 🔥 数据比对：如果数据签名相同，跳过更新（避免乐观更新 + HTTP 响应的双重触发）
    const newSignature = getDataSignature(areaStats.value)
    if (newSignature === lastDataSignature) {
      return // 数据没变，跳过
    }
    lastDataSignature = newSignature
  },
  { immediate: true, deep: true }
)

// 处理返回按钮
function handleBack() {
  emit('back')
}

// 处理下一步/完成按钮
function handleNext() {
  if (props.step === 1) {
    emit('next')
  } else {
    emit('done')
  }
}
</script>

<style scoped>
.daily-planning-wizard {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1rem 3rem 3rem;
  overflow-y: auto;
}

/* ==================== 标题区域 ==================== */
.wizard-header {
  margin-bottom: 2.5rem;
  text-align: center;
  padding-top: 0;
}

.wizard-title {
  font-size: 2.4rem;
  font-weight: 600;
  color: var(--color-text-accent, #f0f);
  margin: 0 0 0.8rem;
  line-height: 1.3;
  letter-spacing: -0.02em;
}

.wizard-subtitle {
  font-size: 1.6rem;
  color: var(--color-text-primary, #f0f);
  margin: 0;
  line-height: 1.4;
  opacity: 0.85;
}

/* ==================== 内容区域 ==================== */
.wizard-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1.6rem;
}

/* ==================== 饼状图区域 ==================== */
.chart-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2rem;
}

/* 图表外层容器 */
.chart-wrapper {
  position: relative;
  width: 100%;
  max-width: 22rem;
  display: flex;
  justify-content: center;
}

/* 发光效果 */
.chart-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 70%;
  height: 70%;
  background: radial-gradient(
    circle,
    var(--color-background-accent-light, #f0f) 0%,
    transparent 70%
  );
  filter: blur(30px);
  pointer-events: none;
  opacity: 0.6;
  animation: glow-pulse 4s ease-in-out infinite;
}

@keyframes glow-pulse {
  0%,
  100% {
    opacity: 0.4;
    transform: translate(-50%, -50%) scale(1);
  }

  50% {
    opacity: 0.7;
    transform: translate(-50%, -50%) scale(1.05);
  }
}

.chart-container {
  width: 100%;
  position: relative;
  z-index: 1;
}

.donut-chart {
  width: 100%;
  height: 220px;
}

/* ==================== 精致图例 ==================== */
.chart-legend {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  width: 100%;
  padding: 0 0.5rem;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.75rem 1rem;
  border-radius: 0.75rem;
  background: transparent;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: default;
  animation: legend-slide-in 0.3s ease-out backwards;
  animation-delay: var(--delay, 0ms);
}

@keyframes legend-slide-in {
  from {
    opacity: 0;
    transform: translateX(-8px);
  }

  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.legend-item:hover {
  background: var(--color-background-hover, #f0f);
}

/* 图例图标 */
.legend-icon {
  flex-shrink: 0;
}

.legend-name {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-tertiary, #f0f);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: color 0.2s ease;
}

.legend-item:hover .legend-name {
  color: var(--color-text-secondary, #f0f);
}

.legend-count {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  min-width: 2rem;
  text-align: right;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.02em;
}

.legend-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  font-size: 1.3rem;
  color: var(--color-text-tertiary, #f0f);
  padding: 2rem 1rem;
  opacity: 0.7;
}

.empty-icon {
  font-size: 1.6rem;
  opacity: 0.5;
}

/* TransitionGroup 动画 */
.legend-fade-enter-active,
.legend-fade-leave-active {
  transition: all 0.3s ease;
}

.legend-fade-enter-from {
  opacity: 0;
  transform: translateX(-8px);
}

.legend-fade-leave-to {
  opacity: 0;
  transform: translateX(8px);
}

.legend-fade-move {
  transition: transform 0.3s ease;
}

/* ==================== 底部导航 ==================== */
.wizard-navigation {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-top: 2rem;
}

.nav-btn {
  padding: 1rem 2rem;
  border-radius: 0.75rem;
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  line-height: 1.4;
}

.back-btn {
  width: 4.8rem;
  height: 4.8rem;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  border: 1px solid var(--color-border-default, #f0f);
  color: var(--color-text-secondary, #f0f);
}

.back-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover, #f0f);
  transform: translateX(-2px);
}

.back-btn:focus-visible {
  outline: none;
  box-shadow: var(--shadow-focus, #f0f);
  border-color: var(--color-border-focus, #f0f);
}

.back-icon {
  font-size: 1.8rem;
}

.next-btn {
  flex: 1;
  height: 4.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-button-primary-bg, #f0f);
  border: 1px solid transparent;
  color: var(--color-button-primary-text, #f0f);
  box-shadow: var(--shadow-sm, #f0f);
}

.next-btn:hover {
  background-color: var(--color-button-primary-hover, #f0f);
  box-shadow: var(--shadow-md, #f0f);
  transform: translateY(-1px);
}

.next-btn:active {
  transform: translateY(0);
  box-shadow: var(--shadow-sm, #f0f);
}

.next-btn:focus-visible {
  outline: none;
  box-shadow: var(--shadow-focus, #f0f);
}

/* ==================== 底部提示 ==================== */
.wizard-hint {
  margin-top: 2rem;
  padding-top: 1.6rem;
  border-top: 1px solid var(--color-border-subtle, #f0f);
}

.hint-text {
  font-size: 1.35rem;
  color: var(--color-text-accent, #f0f);
  margin: 0;
  line-height: 1.6;
  font-style: italic;
  opacity: 0.85;
}
</style>

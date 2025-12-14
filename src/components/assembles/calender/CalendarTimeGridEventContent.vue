<script setup lang="ts">
import { computed } from 'vue'
import CuteDualModeCheckbox from '@/components/parts/CuteDualModeCheckbox.vue'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'
import { useUserSettingsStore } from '@/stores/user-settings'

type CheckboxState = null | 'completed' | 'present'

interface Props {
  title: string
  areaColor: string
  startTime: string // ISO 时间字符串
  endTime: string // ISO 时间字符串
  taskId?: string // 关联的任务ID（如果这是时间块而非任务，则为空）
  isCompleted?: boolean // 任务是否已完成
  scheduleOutcome?: string | null // 日程的 outcome 状态
  scheduleDay?: string // 日程日期
  isPreviewOnly?: boolean // 🆕 是否为纯预览模式（仅显示时间，隐藏标题和复选框）
}

const props = defineProps<Props>()

const userSettingsStore = useUserSettingsStore()

// 判断是否为深色主题
const isDarkTheme = computed(() => {
  const theme = userSettingsStore.theme
  // rose-pine 和 rose-pine-moon 是深色主题，rose-pine-dawn 和 able 是浅色主题
  return theme === 'rose-pine' || theme === 'rose-pine-moon'
})

// 格式化时间为 "09:30 AM" 格式
function formatTime(isoString: string): string {
  const date = new Date(isoString)
  if (Number.isNaN(date.getTime())) {
    return '--:--'
  }
  let hours = date.getHours()
  const minutes = date.getMinutes()
  const period = hours >= 12 ? 'PM' : 'AM'
  hours = hours % 12 || 12 // 转换为 12 小时制，0 点显示为 12
  const paddedMinutes = minutes.toString().padStart(2, '0')
  return `${hours}:${paddedMinutes} ${period}`
}

// 解析颜色为 RGB 分量
function parseColorToRGB(color: string): { r: number; g: number; b: number } {
  let r = 128,
    g = 128,
    b = 128

  // 如果是 hex 格式
  if (color.startsWith('#')) {
    const hex = color.replace('#', '')
    r = parseInt(hex.substring(0, 2), 16)
    g = parseInt(hex.substring(2, 4), 16)
    b = parseInt(hex.substring(4, 6), 16)
  }
  // 如果是 rgb/rgba 格式
  else if (color.startsWith('rgb')) {
    const match = color.match(/\d+/g)
    if (match && match.length >= 3 && match[0] && match[1] && match[2]) {
      r = parseInt(match[0])
      g = parseInt(match[1])
      b = parseInt(match[2])
    }
  }

  return { r, g, b }
}

// 根据主题生成不透明的背景色
function getAdaptiveBackgroundColor(color: string, isDark: boolean): string {
  const { r, g, b } = parseColorToRGB(color)

  if (isDark) {
    // 深色主题：将颜色与深色背景混合（保留 20% 原色，80% 深色背景）
    // 深色背景基准色：约 #1f1d2e (Rose Pine) -> rgb(31, 29, 46)
    const bgR = 31,
      bgG = 29,
      bgB = 46
    const ratio = 0.2 // 原色占比

    const mixR = Math.round(r * ratio + bgR * (1 - ratio))
    const mixG = Math.round(g * ratio + bgG * (1 - ratio))
    const mixB = Math.round(b * ratio + bgB * (1 - ratio))

    return `rgb(${mixR}, ${mixG}, ${mixB})`
  } else {
    // 浅色主题：将颜色与白色混合（保留 15% 原色，85% 白色）
    const ratio = 0.15

    const mixR = Math.round(r * ratio + 255 * (1 - ratio))
    const mixG = Math.round(g * ratio + 255 * (1 - ratio))
    const mixB = Math.round(b * ratio + 255 * (1 - ratio))

    return `rgb(${mixR}, ${mixG}, ${mixB})`
  }
}

const timeRange = `${formatTime(props.startTime)} > ${formatTime(props.endTime)}`

// 响应式背景色，随主题变化
const backgroundColor = computed(() => getAdaptiveBackgroundColor(props.areaColor, isDarkTheme.value))

// 计算复选框状态
const effectiveScheduleDay = computed(() => props.scheduleDay ?? props.startTime.slice(0, 10))

const checkboxState = computed<CheckboxState>(() => {
  // 只有有任务ID的时间块才显示复选框
  if (!props.taskId) return null

  if (props.isCompleted) {
    return 'completed'
  }

  const normalizedOutcome = props.scheduleOutcome
    ? String(props.scheduleOutcome).toUpperCase()
    : null
  if (normalizedOutcome === 'PRESENCE_LOGGED') {
    return 'present'
  }
  return null
})

const checkboxInteractionKey = computed(() => {
  if (!props.taskId) return undefined
  const scheduleDay = effectiveScheduleDay.value
  return scheduleDay ? `timegrid::${props.taskId}::${scheduleDay}` : `timegrid::${props.taskId}`
})

// 处理复选框状态变化
async function handleCheckboxStateChange(newState: CheckboxState) {
  const scheduleDay = effectiveScheduleDay.value
  const previousState = checkboxState.value

  if (!props.taskId || !scheduleDay) {
    logger.warn(
      LogTags.COMPONENT_CALENDAR,
      'Cannot change checkbox state: missing taskId or scheduleDay',
      {
        taskId: props.taskId,
        scheduleDay,
      }
    )
    return
  }

  logger.debug(LogTags.COMPONENT_CALENDAR, 'TimeGrid checkbox state changed', {
    taskId: props.taskId,
    oldState: checkboxState.value,
    newState,
    scheduleDay: props.scheduleDay,
  })

  // 完成状态变化
  if (newState === 'completed') {
    // 标记为完成（依赖视图上下文）
    await pipeline.dispatch('task.complete', {
      id: props.taskId,
      view_context: `daily::${scheduleDay}`,
    })
  } else if (newState === 'present') {
    // 标记在场（更新日程 outcome）
    await pipeline.dispatch('schedule.update', {
      task_id: props.taskId,
      scheduled_day: scheduleDay,
      updates: { outcome: 'PRESENCE_LOGGED' },
    })
  } else {
    // 取消状态（重开任务或取消在场）
    if (previousState === 'completed') {
      await pipeline.dispatch('task.reopen', {
        id: props.taskId,
      })
    } else if (previousState === 'present') {
      await pipeline.dispatch('schedule.update', {
        task_id: props.taskId,
        scheduled_day: scheduleDay,
        updates: { outcome: 'PLANNED' },
      })
    }
  }
}
</script>

<template>
  <div class="timegrid-event-content" :style="{ backgroundColor }">
    <!-- 左侧强调条 -->
    <div class="accent-bar" :style="{ backgroundColor: areaColor }"></div>

    <!-- 内容区域 -->
    <div class="event-body">
      <!-- 时间范围（顶格） -->
      <div class="time-range">{{ timeRange }}</div>

      <!-- 标题行：复选框 + 标题（预览模式下隐藏） -->
      <div v-if="!isPreviewOnly" class="title-row">
        <CuteDualModeCheckbox
          v-if="taskId"
          class="event-checkbox"
          :state="checkboxState"
          size="1.6rem"
          :interaction-key="checkboxInteractionKey"
          @update:state="handleCheckboxStateChange"
          @click.stop
        />
        <div class="event-title">{{ title }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.timegrid-event-content {
  display: flex;
  width: 100%;
  height: 100%;
  border-radius: 0.4rem;
  overflow: hidden;
  position: relative;
  padding-left: 0.5rem;
}

/* 左侧强调条 */
.accent-bar {
  width: 0.4rem;
  flex-shrink: 0;
  border-radius: 0.2rem;
  align-self: stretch;
  margin: 0.5rem 0;
}

/* 内容区域 */
.event-body {
  flex: 1;
  padding: 0.4rem 0.6rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  min-width: 0;
}

/* 时间范围（顶格显示） */
.time-range {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text-secondary, #f0f);
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 标题行：复选框 + 标题 */
.title-row {
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
  min-width: 0;
}

/* 复选框 */
.event-checkbox {
  flex-shrink: 0;
  margin-top: 0.1rem; /* 微调对齐 */
}

/* 事件标题 */
.event-title {
  flex: 1;
  font-size: 1.3rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow-wrap: break-word;
  min-width: 0;
}
</style>

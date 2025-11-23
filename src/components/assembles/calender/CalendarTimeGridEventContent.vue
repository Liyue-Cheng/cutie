<script setup lang="ts">
import { computed } from 'vue'
import CuteDualModeCheckbox from '@/components/parts/CuteDualModeCheckbox.vue'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'

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
  isPreview?: boolean // 是否为预览事件（创建中），预览时不显示标题
}

const props = defineProps<Props>()

/**
 * 格式化时间为 "09:30 AM" 格式
 *
 * 🕐 输入：ISO 8601 时间字符串（如 "2024-11-24T14:30:00Z"）
 * 📤 输出：12 小时制时间（如 "2:30 PM"）
 *
 * @param isoString ISO 时间字符串
 * @returns 格式化后的时间字符串
 */
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

/**
 * 将区域颜色调浅作为卡片背景色
 *
 * 🎨 算法：
 * - 提取 RGB 三个通道的值
 * - 每个通道向 255（白色）混合，保留 15% 原色，混入 85% 白色
 * - 例如：#4a90e2（蓝色）→ rgb(230, 240, 250)（浅蓝背景）
 *
 * 📌 支持格式：
 * - Hex：#4a90e2
 * - RGB/RGBA：rgb(74, 144, 226) / rgba(74, 144, 226, 0.5)
 *
 * @param color 区域颜色（hex 或 rgb 格式）
 * @returns 调浅后的颜色（rgb 格式）
 */
function getLightenedColor(color: string): string {
  // 如果是 hex 格式
  if (color.startsWith('#')) {
    const hex = color.replace('#', '')
    const r = parseInt(hex.substring(0, 2), 16)
    const g = parseInt(hex.substring(2, 4), 16)
    const b = parseInt(hex.substring(4, 6), 16)

    // 调浅颜色：向白色(255)混合，保持85%的白色
    const lighten = (value: number) => Math.round(value + (255 - value) * 0.85)

    const lightR = lighten(r)
    const lightG = lighten(g)
    const lightB = lighten(b)

    return `rgb(${lightR}, ${lightG}, ${lightB})`
  }

  // 如果是 rgb/rgba 格式
  if (color.startsWith('rgb')) {
    const match = color.match(/\d+/g)
    if (match && match.length >= 3 && match[0] && match[1] && match[2]) {
      const r = parseInt(match[0])
      const g = parseInt(match[1])
      const b = parseInt(match[2])

      const lighten = (value: number) => Math.round(value + (255 - value) * 0.85)

      const lightR = lighten(r)
      const lightG = lighten(g)
      const lightB = lighten(b)

      return `rgb(${lightR}, ${lightG}, ${lightB})`
    }
  }

  // 默认返回浅灰色
  return '#f5f5f5'
}

// 🕐 时间范围字符串："9:30 AM > 10:45 AM"
const timeRange = `${formatTime(props.startTime)} > ${formatTime(props.endTime)}`

// 🎨 背景颜色：将区域颜色调浅 85%
const backgroundColor = getLightenedColor(props.areaColor)

// 📅 计算有效的日程日期（用于复选框状态判断）
// 如果没有显式传入 scheduleDay，则从 startTime 提取日期部分
const effectiveScheduleDay = computed(() => props.scheduleDay ?? props.startTime.slice(0, 10))

/**
 * 计算复选框状态
 *
 * 🎯 状态定义：
 * - null：未完成且未在场
 * - 'completed'：任务已完成
 * - 'present'：已记录在场（schedule.outcome = 'PRESENCE_LOGGED'）
 *
 * 📌 注意：
 * - 只有关联了任务的时间块才显示复选框（taskId 不为空）
 * - 空时间块（独立事件）不显示复选框
 */
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

/**
 * 处理复选框状态变化
 *
 * 🔄 状态流转：
 * - null → 'completed'：完成任务（task.complete）
 * - null → 'present'：记录在场（schedule.update outcome=PRESENCE_LOGGED）
 * - 'completed' → null：重新打开任务（task.reopen）
 * - 'present' → null：取消在场（schedule.update outcome=PLANNED）
 *
 * 🎯 关键点：
 * - 使用 view_context = `daily::{scheduleDay}` 确保乐观更新正确
 * - 失败时会自动回滚（由 CPU pipeline 的指令系统处理）
 *
 * @param newState 新的复选框状态
 */
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
      <div v-if="!isPreview" class="title-row">
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
  /* 🔒 尺寸锁定：使用绝对定位填满 FullCalendar 分配的空间 */
  position: absolute;
  inset: 0; /* top: 0; right: 0; bottom: 0; left: 0; */
  display: flex;
  border-radius: 0.4rem;
  overflow: hidden; /* 🔑 关键：裁剪超出内容，防止撑大容器 */
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
  overflow: hidden; /* 🔑 裁剪超出内容，防止内部元素撑大 */
}

/* 时间范围（顶格显示） */
.time-range {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text-secondary, #6e6a86);
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
  color: var(--color-text-primary, #575279);
  line-height: 1.4;
  overflow: hidden; /* 🔑 裁剪超出文本 */
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2; /* 最多显示 2 行 */
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow-wrap: break-word;
  min-width: 0;

  /* 🔒 即使标题很长或有复选框，也不会撑大容器高度 */
}
</style>

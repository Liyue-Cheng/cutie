<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useTimeBlockRecurrenceOperations } from '@/composables/useTimeBlockRecurrenceOperations'
import { getTimeBlockRecurrenceById } from '@/cpu/isa/timeblock-recurrence-isa'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import TimeBlockRecurrenceConfigDialog from '@/components/parts/recurrence/TimeBlockRecurrenceConfigDialog.vue'

const props = defineProps<{
  timeBlockId: string | null
  panelPosition?: {
    top: number
    left: number
  } | null
}>()

const emit = defineEmits<{
  close: []
}>()

// 面板引用，用于点击外部关闭和位置调整
const panelRef = ref<HTMLElement | null>(null)

// 调整后的位置（考虑视口边界）
const adjustedPosition = ref<{ top: number; left: number } | null>(null)

// 视口边距
const VIEWPORT_PADDING = 16

// 调整面板位置，确保不超出视口
function adjustPanelPosition() {
  if (!panelRef.value || !props.panelPosition) return

  const panel = panelRef.value
  const panelRect = panel.getBoundingClientRect()
  const viewportHeight = window.innerHeight
  const viewportWidth = window.innerWidth

  let { top, left } = props.panelPosition

  // 面板实际渲染后的高度
  const panelHeight = panelRect.height
  const panelWidth = panelRect.width

  // 检查底部是否超出视口
  // 注意：面板使用 transform: translate(-100% - 1.2rem, 0)，所以实际位置在 left 左边
  // top 是面板顶部位置
  if (top + panelHeight > viewportHeight - VIEWPORT_PADDING) {
    // 往上移动，使面板底部距离视口底部有 VIEWPORT_PADDING 的距离
    top = viewportHeight - panelHeight - VIEWPORT_PADDING
  }

  // 确保顶部不超出视口
  if (top < VIEWPORT_PADDING) {
    top = VIEWPORT_PADDING
  }

  // 检查左侧是否超出视口（面板在锚点左边）
  // 面板右边缘在 left - 1.2rem 处，左边缘在 left - 1.2rem - panelWidth 处
  const panelLeftEdge = left - 12 - panelWidth // 1.2rem ≈ 12px
  if (panelLeftEdge < VIEWPORT_PADDING) {
    // 面板太靠左，调整 left 使面板左边缘有足够边距
    left = panelWidth + 12 + VIEWPORT_PADDING
  }

  adjustedPosition.value = { top, left }
}

// 监听 panelPosition 变化和 timeBlockId 变化，重新调整位置
watch(
  [() => props.panelPosition, () => props.timeBlockId],
  () => {
    // 重置调整位置，等待下一帧重新计算
    adjustedPosition.value = null
    nextTick(() => {
      // 等待 DOM 更新后再调整
      nextTick(() => {
        adjustPanelPosition()
      })
    })
  },
  { immediate: true }
)

// 点击外部关闭
function handleClickOutside(event: MouseEvent) {
  if (!panelRef.value) return

  // 如果循环配置对话框打开，不处理点击外部关闭
  // 对话框有自己的 backdrop 处理关闭逻辑
  if (showRecurrenceDialog.value) {
    return
  }

  const target = event.target as HTMLElement

  // 检查是否点击在面板内部
  if (panelRef.value.contains(target)) {
    return
  }

  // 阻止事件传播，防止其他组件响应
  event.stopPropagation()
  event.preventDefault()

  // 点击外部，关闭面板
  emit('close')
}

onMounted(() => {
  // 使用 capture 阶段捕获点击，防止其他组件先响应
  document.addEventListener('mousedown', handleClickOutside, true)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside, true)
})

const timeBlockStore = useTimeBlockStore()
const recurrenceOps = useTimeBlockRecurrenceOperations()

// 循环配置对话框状态
const showRecurrenceDialog = ref(false)

// 动态计算面板位置（优先使用调整后的位置）
const panelStyle = computed(() => {
  // 优先使用调整后的位置
  if (adjustedPosition.value) {
    return {
      top: `${adjustedPosition.value.top}px`,
      left: `${adjustedPosition.value.left}px`,
    }
  }

  // 首次渲染时使用原始位置
  const top =
    props.panelPosition?.top ?? (typeof window !== 'undefined' ? window.innerHeight / 2 : 0)
  const left =
    props.panelPosition?.left ?? (typeof window !== 'undefined' ? window.innerWidth / 2 : 0)

  return {
    top: `${top}px`,
    left: `${left}px`,
  }
})

// 获取当前时间块
const timeBlock = computed(() => {
  if (!props.timeBlockId) return null
  return timeBlockStore.getTimeBlockById(props.timeBlockId)
})

// 获取链接的任务
const linkedTasks = computed(() => {
  if (!timeBlock.value) return []
  return timeBlock.value.linked_tasks || []
})

// 格式化时间范围
const { t } = useI18n()

function formatTimeRange(timeBlock: any) {
  if (timeBlock.is_all_day) {
    return t('timeBlock.label.allDay')
  }

  let startTime: string
  let endTime: string

  // 如果是浮动时间且有本地时间，使用本地时间
  if (
    timeBlock.time_type === 'FLOATING' &&
    timeBlock.start_time_local &&
    timeBlock.end_time_local
  ) {
    startTime = timeBlock.start_time_local.substring(0, 5) // HH:MM
    endTime = timeBlock.end_time_local.substring(0, 5) // HH:MM
  } else {
    // 否则使用UTC时间转换为本地时间显示
    const startDate = new Date(timeBlock.start_time)
    const endDate = new Date(timeBlock.end_time)
    startTime = `${startDate.getHours().toString().padStart(2, '0')}:${startDate.getMinutes().toString().padStart(2, '0')}`
    endTime = `${endDate.getHours().toString().padStart(2, '0')}:${endDate.getMinutes().toString().padStart(2, '0')}`
  }

  return `${startTime} - ${endTime}`
}

// 处理循环配置成功
function handleRecurrenceSuccess() {
  showRecurrenceDialog.value = false
  emit('close')
}

// 检查是否为循环时间块
const isRecurringTimeBlock = computed(() => {
  return !!(timeBlock.value?.recurrence_id && timeBlock.value?.recurrence_original_date)
})

// 检查循环是否已停止（有 end_date 表示已停止）
const isRecurrenceStopped = computed(() => {
  if (!timeBlock.value?.recurrence_id) return false
  const recurrence = getTimeBlockRecurrenceById(timeBlock.value.recurrence_id)
  return !!recurrence?.end_date
})

// 循环操作处理函数
async function handleStopRepeating() {
  if (!timeBlock.value?.recurrence_id || !timeBlock.value?.recurrence_original_date) return

  try {
    await recurrenceOps.stopRepeating(
      timeBlock.value.recurrence_id,
      timeBlock.value.recurrence_original_date
    )
    emit('close')
  } catch (error) {
    console.error('Failed to stop repeating:', error)
  }
}

async function handleChangeFrequency() {
  if (!timeBlock.value?.recurrence_id) return
  recurrenceOps.openEditDialog(timeBlock.value.recurrence_id)
  emit('close')
}

async function handleResumeRepeating() {
  if (!timeBlock.value?.recurrence_id) return

  try {
    await recurrenceOps.resumeRecurrence(timeBlock.value.recurrence_id)
  } catch (error) {
    console.error('Failed to resume repeating:', error)
  }
}
</script>

<template>
  <div v-if="timeBlock" ref="panelRef" class="time-block-detail-panel" :style="panelStyle">
    <div class="panel-header">
      <h3>{{ $t('timeBlock.title.detail') }}</h3>
      <button class="close-btn" @click="emit('close')">
        <CuteIcon name="X" :size="16" />
      </button>
    </div>

    <div class="panel-content">
      <!-- 时间块基本信息 -->
      <div class="time-block-info">
        <div class="info-row">
          <CuteIcon name="Clock" :size="16" />
          <span class="time-range">
            {{ formatTimeRange(timeBlock) }}
          </span>
          <span v-if="timeBlock.time_type === 'FLOATING'" class="time-type-badge floating">
            {{ $t('timeBlock.label.timeType.floating') }}
          </span>
          <span v-else class="time-type-badge fixed">
            {{ $t('timeBlock.label.timeType.fixed') }}
          </span>
        </div>
        <div v-if="timeBlock.title" class="info-row">
          <CuteIcon name="FileText" :size="16" />
          <span class="block-title">{{ timeBlock.title }}</span>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="actions-section">
        <!-- 设置循环按钮（仅非循环时间块显示） -->
        <button
          v-if="!timeBlock.is_recurring"
          class="action-btn"
          @click="showRecurrenceDialog = true"
        >
          <CuteIcon name="Repeat" :size="16" />
          <span>{{ $t('recurrence.action.setRecurrence') }}</span>
        </button>
        <!-- 已是循环时间块的提示 -->
        <div v-else class="recurring-badge">
          <CuteIcon name="Repeat" :size="14" />
          <span>{{ $t('recurrence.label.isRecurring') }}</span>
        </div>
      </div>

      <!-- 循环时间块相关操作 -->
      <div v-if="isRecurringTimeBlock" class="recurrence-actions-section">
        <div class="section-header">
          <CuteIcon name="Repeat" :size="16" />
          <span>{{ $t('recurrence.timeBlockMenuSection') }}</span>
        </div>
        <div class="recurrence-actions">
          <!-- 停止和继续是互斥的：有 end_date 时显示继续，否则显示停止 -->
          <button
            v-if="!isRecurrenceStopped"
            class="recurrence-action-btn"
            @click="handleStopRepeating"
          >
            <CuteIcon name="Square" :size="14" />
            <span>{{ $t('recurrence.action.stop') }}</span>
          </button>
          <button v-else class="recurrence-action-btn" @click="handleResumeRepeating">
            <CuteIcon name="Play" :size="14" />
            <span>{{ $t('recurrence.action.continue') }}</span>
          </button>
          <button class="recurrence-action-btn" @click="handleChangeFrequency">
            <CuteIcon name="RefreshCw" :size="14" />
            <span>{{ $t('recurrence.action.changeFrequency') }}</span>
          </button>
        </div>
      </div>

      <!-- 链接的任务列表 -->
      <div class="linked-tasks-section">
        <div class="section-header">
          <CuteIcon name="Link" :size="16" />
          <span>{{ $t('task.label.linkedTasks') }}</span>
          <span class="task-count">{{ linkedTasks.length }}</span>
        </div>

        <div v-if="linkedTasks.length === 0" class="empty-state">
          <p>{{ $t('task.label.noLinkedTasks') }}</p>
        </div>

        <div v-else class="tasks-list">
          <div v-for="task in linkedTasks" :key="task.id" class="task-item">
            <div class="task-title">{{ task.title }}</div>
            <div class="task-status" :class="{ completed: task.is_completed }">
              {{ task.is_completed ? $t('task.status.completed') : $t('task.status.inProgress') }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 循环配置对话框 -->
  <TimeBlockRecurrenceConfigDialog
    v-if="timeBlock"
    :time-block="timeBlock"
    :open="showRecurrenceDialog"
    @close="showRecurrenceDialog = false"
    @success="handleRecurrenceSuccess"
  />
</template>

<style scoped>
.time-block-detail-panel {
  position: fixed;
  transform: translate(
    calc(-100% - 1.2rem),
    0
  ); /* 面板右边缘距离锚点左边缘1.2rem，上边缘与时间块上边缘对齐 */

  width: 28rem;
  max-height: 80vh;
  background-color: var(--color-background-elevated, #f0f);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  box-shadow: var(--shadow-lg, #f0f);
  display: flex;
  flex-direction: column;
  z-index: 1000;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.5rem;
  border-bottom: 1px solid var(--color-border-default);
}

.panel-header h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.close-btn {
  padding: 0.4rem;
  background: none;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  color: var(--color-text-tertiary);
  transition: all 0.2s ease;
}

.close-btn:hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.time-block-info {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  color: var(--color-text-secondary);
  font-size: 1.4rem;
}

.time-range,
.block-title {
  color: var(--color-text-primary);
  font-weight: 500;
}

.linked-tasks-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
  height: 2rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-tertiary);
  background-color: var(--color-background-hover);
  border-radius: 1rem;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.empty-state p {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
}

.tasks-list {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.task-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border-light);
  transition: all 0.2s ease;
  cursor: pointer;
}

.task-item:hover {
  background-color: var(--color-background-hover);
}

.task-title {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.task-item:last-child {
  border-bottom: none;
}

.task-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  background-color: var(--color-background-accent-light);
  color: var(--color-info);
}

.task-status.completed {
  background-color: var(--color-background-secondary);
  color: var(--color-success);
}

.time-type-badge {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  margin-left: 8px;
  font-weight: 500;
}

.time-type-badge.floating {
  background-color: var(--color-background-accent-light);
  color: var(--color-info);
}

.time-type-badge.fixed {
  background-color: var(--color-background-secondary);
  color: var(--color-warning);
}

/* 操作按钮区域 */
.actions-section {
  display: flex;
  gap: 0.8rem;
  padding-top: 1rem;
  border-top: 1px solid var(--color-border-light);
  margin-top: 0.5rem;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.8rem 1.2rem;
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  color: var(--color-text-secondary);
  font-size: 1.3rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: var(--color-background-hover);
  border-color: var(--color-border-hover);
  color: var(--color-text-primary);
}

.recurring-badge {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.6rem 1rem;
  background: var(--color-background-accent-light);
  border-radius: 0.4rem;
  color: var(--color-info);
  font-size: 1.2rem;
  font-weight: 500;
}

/* 循环操作区域 */
.recurrence-actions-section {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  padding-top: 1rem;
  border-top: 1px solid var(--color-border-light);
}

.recurrence-actions {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.recurrence-action-btn {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem 1rem;
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border-light);
  border-radius: 0.6rem;
  color: var(--color-text-secondary);
  font-size: 1.3rem;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.recurrence-action-btn:hover {
  background: var(--color-background-hover);
  border-color: var(--color-border-hover);
  color: var(--color-text-primary);
}

.recurrence-action-btn.danger {
  color: var(--color-danger);
}

.recurrence-action-btn.danger:hover {
  background: var(--color-danger-bg);
  border-color: var(--color-danger);
}
</style>

<template>
  <div class="home-view">
    <!-- 左栏 -->
    <div class="left-column" :style="{ width: leftPaneWidth + '%' }">
      <RecentView v-if="currentView === 'recent'" v-model="calendarDays" />
      <StagingView v-else-if="currentView === 'staging'" />
    </div>

    <!-- 可拖动的分割线 -->
    <div class="divider" @mousedown="startDragging" @dblclick="resetPaneWidth"></div>

    <!-- 右栏 -->
    <div class="right-column">
      <TwoRowLayout>
        <template #top>
          <div class="column-header">
            <h2>日历</h2>
          </div>
        </template>
        <template #bottom>
          <div class="calendar-wrapper">
            <CuteCalendar
              ref="calendarRef"
              :current-date="currentCalendarDate"
              :view-type="effectiveCalendarViewType"
              :zoom="1"
              :days="calendarDays"
            />
          </div>
        </template>
      </TwoRowLayout>
    </div>

    <!-- 任务编辑器模态框挂载点 -->
    <KanbanTaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import RecentView from '@/components/templates/RecentView.vue'
import StagingView from '@/components/templates/StagingView.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import { useRegisterStore } from '@/stores/register'
import { useUIStore } from '@/stores/ui'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

const route = useRoute()
const registerStore = useRegisterStore()
const uiStore = useUIStore()

// ==================== 视图切换状态 ====================
const currentView = ref<'recent' | 'staging'>('recent') // 当前视图

// ==================== 日历天数联动状态 ====================
const calendarDays = ref<1 | 3 | 5 | 7>(3) // 默认显示3天，与 RecentView 联动
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)

// 根据天数计算视图类型：7天显示本周视图，其他显示多天视图
const calendarViewType = computed(() => {
  return calendarDays.value === 7 ? 'week' : 'day'
})

// 最终的日历视图类型：Staging 视图强制使用月视图
const effectiveCalendarViewType = computed(() => {
  if (currentView.value === 'staging') {
    return 'month'
  }
  return calendarViewType.value
})

// 监听路由变化，切换视图
watch(
  () => route.query.view,
  (newView) => {
    if (newView === 'staging') {
      currentView.value = 'staging'
      logger.info(LogTags.VIEW_HOME, 'Switched to Staging view')
    } else {
      currentView.value = 'recent'
      logger.info(LogTags.VIEW_HOME, 'Switched to Recent view')
    }
  },
  { immediate: true }
)

// 初始化
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing Home view with Recent + Calendar...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'home')
})

// ==================== 日历状态 ====================
const currentCalendarDate = computed(() => {
  return (
    registerStore.readRegister<string>(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME) ||
    getTodayDateString()
  )
})

// ==================== 可拖动分割线逻辑 ====================
const leftPaneWidth = ref(40) // 默认比例 2:3，左栏占 40%
const isDragging = ref(false)
let rafId: number | null = null

function startDragging(e: MouseEvent) {
  isDragging.value = true
  document.addEventListener('mousemove', onDragging)
  document.addEventListener('mouseup', stopDragging)
  e.preventDefault()
}

function onDragging(e: MouseEvent) {
  if (!isDragging.value) return

  const container = document.querySelector('.home-view') as HTMLElement
  if (!container) return

  const containerRect = container.getBoundingClientRect()
  const containerWidth = containerRect.width
  const mouseX = e.clientX - containerRect.left

  // 计算新的左栏宽度百分比
  let newWidth = (mouseX / containerWidth) * 100

  // 限制最小和最大宽度（20% - 80%）
  newWidth = Math.max(20, Math.min(80, newWidth))

  leftPaneWidth.value = newWidth

  // 使用 requestAnimationFrame 实现流畅的实时更新
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
  rafId = requestAnimationFrame(() => {
    updateCalendarSize()
    rafId = null
  })
}

// 更新日历尺寸的辅助函数
function updateCalendarSize() {
  if (calendarRef.value?.calendarRef) {
    const calendarApi = calendarRef.value.calendarRef.getApi()
    if (calendarApi) {
      calendarApi.updateSize()
    }
  }
}

async function stopDragging() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)

  // 清除 requestAnimationFrame
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }

  // 最后确保更新一次日历尺寸
  await nextTick()
  updateCalendarSize()
  logger.debug(LogTags.VIEW_HOME, 'Calendar size updated after pane resize')
}

// 双击重置为默认比例
async function resetPaneWidth() {
  leftPaneWidth.value = 40

  // 通知日历更新尺寸
  await nextTick()
  updateCalendarSize()
  logger.debug(LogTags.VIEW_HOME, 'Calendar size updated after pane reset')
}

// 清理事件监听器和动画帧
onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
})
</script>

<style scoped>
.home-view {
  width: 100%;
  height: 100%;
  display: flex;
  overflow: hidden;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

/* 左右栏 */
.left-column,
.right-column {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: transparent;
}

.left-column {
  flex-shrink: 0;
  position: relative;
}

.right-column {
  flex: 1;
  min-width: 0;
  position: relative;
}

/* 分割线 */
.divider {
  width: 1px;
  height: 100%;
  background-color: var(--color-border-default);
  cursor: col-resize;
  flex-shrink: 0;
  transition: background-color 0.2s;
  position: relative;
  z-index: 10;
}

/* 扩大可点击区域 */
.divider::before {
  content: '';
  position: absolute;
  inset: 0 -4px;
  cursor: col-resize;
}

.divider:hover {
  background-color: var(--color-border-hover, var(--color-border-default));
}

/* 列头部 */
.column-header {
  width: 100%;
  display: flex;
  align-items: center;
}

.column-header h2 {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

/* 日历包装器 */
.calendar-wrapper {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

/* 列内容 */
.column-content {
  padding: 2rem;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.placeholder-text {
  font-size: 1.6rem;
  color: var(--color-text-secondary);
}
</style>

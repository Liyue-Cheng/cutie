<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'
import VerticalToolbar from '@/components/functional/VerticalToolbar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import DailyShutdownWizard from '@/components/organisms/DailyShutdownWizard.vue'
import ShutdownRitualStage from '@/components/organisms/ShutdownRitualStage.vue'
import { useTaskStore } from '@/stores/task'
import { useRegisterStore } from '@/stores/register'
import { logger, LogTags } from '@/infra/logging/logger'
import {
  getTodayDateString,
  getTomorrowDateString,
  parseDateString,
  toDateString,
} from '@/infra/utils/dateUtils'

const { t, locale } = useI18n()
const taskStore = useTaskStore()
const registerStore = useRegisterStore()

type Stage = 1 | 2
type RightPaneView = 'completed' | 'daily'

const stage = ref<Stage>(1)
const currentRightView = ref<RightPaneView>('completed')

const today = ref(getTodayDateString())
const todayDailyViewKey = computed(() => `daily::${today.value}`)

const todayTasks = computed(() => taskStore.getTasksByDate_Mux(today.value))
const completedCount = computed(() => todayTasks.value.filter((t) => t.is_completed).length)
const incompleteCount = computed(() => Math.max(0, todayTasks.value.length - completedCount.value))

// Right daily date (start from tomorrow)
const dailyRightDate = ref<string>(getTomorrowDateString())

function shiftDate(baseDate: string, offsetDays: number): string {
  const date = parseDateString(baseDate)
  date.setDate(date.getDate() + offsetDays)
  return toDateString(date)
}

function ensureNotBeforeTomorrow(dateStr: string): string {
  const tomorrow = getTomorrowDateString()
  return dateStr < tomorrow ? tomorrow : dateStr
}

function goToDailyTomorrow() {
  dailyRightDate.value = getTomorrowDateString()
}

function navigateDailyPrev() {
  dailyRightDate.value = ensureNotBeforeTomorrow(shiftDate(dailyRightDate.value, -1))
}

function navigateDailyNext() {
  dailyRightDate.value = shiftDate(dailyRightDate.value, 1)
}

const dailyRightLabel = computed(() => {
  const tomorrowStr = getTomorrowDateString()
  const date = parseDateString(dailyRightDate.value)
  const weekday = new Intl.DateTimeFormat(locale.value, { weekday: 'short' }).format(date)

  if (dailyRightDate.value === tomorrowStr) return `${t('time.tomorrow')} ${weekday}`

  return new Intl.DateTimeFormat(locale.value, {
    month: 'short',
    day: 'numeric',
    weekday: 'short',
  }).format(date)
})

const toolbarConfig = computed(() => ({
  completed: { icon: 'CheckCheck' as const, label: t('dailyShutdown.toolbar.completed') },
  daily: { icon: 'Calendar' as const, label: t('dailyShutdown.toolbar.daily') },
}))

function onRightViewChange(viewKey: string | null) {
  if (!viewKey) return
  currentRightView.value = viewKey as RightPaneView
  if (currentRightView.value === 'daily') {
    dailyRightDate.value = getTomorrowDateString()
  }
}

function goToStage2() {
  stage.value = 2
}

function backToStage1() {
  stage.value = 1
}

watch(
  () => currentRightView.value,
  (view) => {
    if (view === 'daily') {
      dailyRightDate.value = getTomorrowDateString()
    }
  }
)

watch(
  () => dailyRightDate.value,
  async (date) => {
    if (currentRightView.value !== 'daily') return
    await taskStore.fetchDailyTasksRange_DMA(date, date)
  }
)

onMounted(async () => {
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'daily-shutdown')
  logger.info(LogTags.VIEW_HOME, 'Daily Shutdown: Initializing...', { today: today.value })
  await taskStore.fetchDailyTasks_DMA(today.value)
})
</script>

<template>
  <div v-if="stage === 2" class="daily-shutdown-stage2">
    <ShutdownRitualStage :date="today" @back="backToStage1" />
  </div>

  <div v-else class="daily-shutdown-view">
    <!-- 左栏：Wizard -->
    <div class="pane left-pane">
      <TwoRowLayout>
        <template #top>
          <!-- 上栏留空 -->
        </template>
        <template #bottom>
          <DailyShutdownWizard
            :completed="completedCount"
            :incomplete="incompleteCount"
            :title="t('dailyShutdown.stage1.title')"
            :subtitle="t('dailyShutdown.stage1.subtitle')"
            :hint="t('dailyShutdown.stage1.hint')"
            @next="goToStage2"
          />
        </template>
      </TwoRowLayout>
    </div>

    <div class="divider"></div>

    <!-- 中栏：今日未完成 -->
    <div class="pane middle-pane">
      <TwoRowLayout>
        <template #top>
          <!-- 上栏留空 -->
        </template>
        <template #bottom>
          <TaskList
            :title="t('view.dailyShutdown.todayIncomplete')"
            :view-key="todayDailyViewKey"
            :show-add-input="false"
            :fill-remaining-space="true"
            :collapsible="false"
            :hide-completed="true"
          />
        </template>
      </TwoRowLayout>
    </div>

    <div class="divider"></div>

    <!-- 右栏：工具栏控制 -->
    <div class="pane right-pane">
      <TwoRowLayout>
        <template #top>
          <div v-if="currentRightView === 'daily'" class="daily-controls">
            <div class="daily-nav">
              <button
                class="nav-btn"
                :title="t('view.dailyPlanning.dailyTasksNav.prev')"
                @click="navigateDailyPrev"
              >
                <CuteIcon name="ChevronLeft" :size="18" />
              </button>
              <button
                class="nav-btn today-nav-btn"
                :title="t('time.tomorrow')"
                @click="goToDailyTomorrow"
              >
                <span class="today-text">{{ t('time.tomorrow') }}</span>
              </button>
              <button
                class="nav-btn"
                :title="t('view.dailyPlanning.dailyTasksNav.next')"
                @click="navigateDailyNext"
              >
                <CuteIcon name="ChevronRight" :size="18" />
              </button>
            </div>
          </div>
        </template>
        <template #bottom>
          <div v-if="currentRightView === 'completed'" class="right-wrapper">
            <TaskList
              :title="t('view.dailyShutdown.todayCompleted')"
              :view-key="`misc::completed::${today}`"
              :show-add-input="false"
              :fill-remaining-space="true"
              :collapsible="false"
              :disable-drag="true"
              :read-only="true"
            />
          </div>

          <div v-else class="right-wrapper">
            <TaskList
              :title="dailyRightLabel"
              :view-key="`daily::${dailyRightDate}`"
              :show-add-input="true"
              :fill-remaining-space="true"
              :collapsible="false"
            />
          </div>
        </template>
      </TwoRowLayout>
    </div>

    <VerticalToolbar
      :view-config="toolbarConfig"
      :current-view="currentRightView"
      @view-change="onRightViewChange"
    />
  </div>
</template>

<style scoped>
.daily-shutdown-view {
  width: 100%;
  height: 100%;
  display: flex;
  gap: 3rem;
  overflow: hidden;
  background-color: var(--color-background-content, #f0f);
}

.daily-shutdown-stage2 {
  width: 100%;
  height: 100%;
  overflow: hidden;
  background-color: var(--color-background-content, #f0f);
}

.pane {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: transparent;
}

.left-pane,
.middle-pane,
.right-pane {
  flex: 1;
  min-width: 0;
}

.divider {
  width: 1px;
  height: 100%;
  background-color: var(--color-border-adaptive-light-subtle-dark-none, #f0f);
  flex-shrink: 0;
}

.right-wrapper {
  height: 100%;
  overflow: hidden;
}

.daily-controls {
  width: 100%;
  height: 100%;
  padding: 1.2rem 0.8rem 1.2rem 1.6rem;
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.daily-nav {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.daily-controls .nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 3.6rem;
  padding: 0 1.2rem;
  color: var(--color-text-secondary, #f0f);
  background-color: transparent;
  border: 1px solid transparent;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.daily-controls .nav-btn:hover {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-default, #f0f);
}

.daily-controls .today-nav-btn {
  padding: 0 1.4rem;
}

.daily-controls .today-text {
  font-size: 1.4rem;
  font-weight: 600;
  line-height: 1.4;
}
</style>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import CutePane from '@/components/alias/CutePane.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'
import TaskStrip from '@/components/assembles/tasks/list/TaskStrip.vue'
import CuteCalendar from '@/components/assembles/calender/CuteCalendar.vue'
import { useTaskStore } from '@/stores/task'
import { useRecurrenceStore } from '@/stores/recurrence'
import { useRegisterStore } from '@/stores/register'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

const taskStore = useTaskStore()
const recurrenceStore = useRecurrenceStore()
const registerStore = useRegisterStore()

const today = ref(getTodayDateString())
const dailyViewKey = computed(() => `daily::${today.value}`)

// 今日所有排期任务（包含循环与非循环）
const todayTasks = computed(() => {
  return taskStore.getTasksByDate_Mux(today.value)
})

// 每日循环任务
const dailyRecurringTasks = computed(() => {
  return todayTasks.value.filter((task) => {
    if (!task.recurrence_id) return false

    const recurrence = recurrenceStore.getRecurrenceById(task.recurrence_id)
    if (!recurrence) return false

    return recurrence.rule.includes('FREQ=DAILY')
  })
})

onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing Daily overview view...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'daily-overview')

  // 加载未完成任务（包含循环实例）
  await taskStore.fetchAllIncompleteTasks_DMA()

  logger.info(LogTags.VIEW_HOME, 'Daily overview loaded tasks', {
    today: today.value,
    total: todayTasks.value.length,
    dailyRecurring: dailyRecurringTasks.value.length,
  })
})
</script>

<template>
  <div class="daily-overview-view">
    <TwoRowLayout>
      <!-- 顶部：标题区 -->
      <template #top>
        <div class="overview-header">
          <h2 class="overview-title">{{ $t('view.dailyOverview.greeting') }}</h2>
        </div>
      </template>

      <!-- 底部：四列内容 -->
      <template #bottom>
        <div class="overview-grid">
          <!-- 天气 -->
          <CutePane class="overview-card weather-card">
            <div class="card-header">
              <div class="card-title">
                <CuteIcon name="CloudSun" :size="18" />
                <span>{{ $t('view.dailyOverview.weather') }}</span>
              </div>
            </div>
            <div class="card-body weather-body">
              <div class="weather-main">
                <div class="weather-temp">26°</div>
                <div class="weather-summary">
                  <div class="weather-status">Partly cloudy</div>
                  <div class="weather-meta">Feels like 27° · Light breeze</div>
                </div>
              </div>
              <div class="weather-extra">
                <span>☔ 10%</span>
                <span>💨 8 km/h</span>
                <span>💧 62%</span>
              </div>
              <p class="weather-hint">{{ $t('view.dailyOverview.weatherHint') }}</p>
            </div>
          </CutePane>

          <!-- 每日循环任务 -->
          <CutePane class="overview-card recurring-card">
            <div class="card-header">
              <div class="card-title">
                <CuteIcon name="RefreshCw" :size="18" />
                <span>{{ $t('view.dailyOverview.dailyRecurring') }}</span>
              </div>
              <span class="card-count">{{ dailyRecurringTasks.length }}</span>
            </div>
            <div class="card-body task-list-body">
              <div class="task-list">
                <TaskStrip
                  v-for="task in dailyRecurringTasks"
                  :key="task.id"
                  :task="task"
                  :view-key="dailyViewKey"
                />
                <div v-if="dailyRecurringTasks.length === 0" class="empty-state">
                  <p>{{ $t('view.dailyOverview.noDailyRecurring') }}</p>
                </div>
              </div>
            </div>
          </CutePane>

          <!-- 任务看板：非每日循环任务（TaskList 实现） -->
          <CutePane class="overview-card tasks-card">
            <div class="card-header">
              <div class="card-title">
                <CuteIcon name="Check" :size="18" />
                <span>{{ $t('view.dailyOverview.todaysTasks') }}</span>
              </div>
            </div>
            <div class="card-body tasks-list-card-body">
              <TaskList
                :title="$t('view.dailyOverview.todaysTasks')"
                :view-key="dailyViewKey"
                :hide-daily-recurring-tasks="true"
                :fill-remaining-space="true"
                :collapsible="false"
              />
            </div>
          </CutePane>

          <!-- 日历：单天视图 -->
          <CutePane class="overview-card calendar-card">
            <div class="card-header">
              <div class="card-title">
                <CuteIcon name="Calendar" :size="18" />
                <span>{{ $t('time.today') }}</span>
              </div>
              <span class="card-subtitle">{{ today }}</span>
            </div>
            <div class="card-body calendar-body">
              <CuteCalendar :current-date="today" :view-type="'day'" :days="1" :zoom="1" />
            </div>
          </CutePane>
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<style scoped>
.daily-overview-view {
  width: 100%;
  height: 100%;
  background-color: var(--color-background-content);
  overflow: hidden;
}

.overview-header {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  width: 100%;
}

.overview-title {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 1rem;
  padding: 1rem;
  height: 100%;
  box-sizing: border-box;
}

.overview-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 0.8rem 0.4rem;
  border-bottom: 1px solid var(--color-border-soft, #f0f);
}

.card-title {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.card-count {
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
}

.card-subtitle {
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
}

.card-body {
  flex: 1;
  min-height: 0;
  padding: 0.8rem;
  box-sizing: border-box;
}

.tasks-list-card-body {
  padding: 0.4rem 0.4rem 0.8rem;
}

/* 天气卡片 */
.weather-body {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.weather-main {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.weather-temp {
  font-size: 3.2rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.weather-summary {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.weather-status {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.weather-meta {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
}

.weather-extra {
  display: flex;
  gap: 0.8rem;
  font-size: 1.1rem;
  color: var(--color-text-tertiary);
}

.weather-hint {
  margin: 0;
  font-size: 1.1rem;
  color: var(--color-text-tertiary);
}

/* 任务列表卡片 */
.task-list-body {
  padding: 0.4rem 0.6rem 0.8rem;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  height: 100%;
  overflow-y: auto;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-tertiary);
  font-size: 1.2rem;
}

/* 日历卡片 */
.calendar-body {
  padding: 0.4rem;
}

.calendar-body :deep(.calendar-container) {
  border-radius: 0.6rem;
  border: 1px solid var(--color-border-soft, #f0f);
}
</style>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import CutePane from '@/components/alias/CutePane.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import SimpleKanbanColumn from '@/components/assembles/tasks/kanban/SimpleKanbanColumn.vue'
import UnderConstruction from '@/components/organisms/UnderConstruction.vue'
import { useTaskStore } from '@/stores/task'
import { useRegisterStore } from '@/stores/register'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

const taskStore = useTaskStore()
const registerStore = useRegisterStore()

const today = ref(getTodayDateString())

const tomorrow = computed(() => {
  const date = new Date(today.value)
  date.setDate(date.getDate() + 1)
  return date.toISOString().split('T')[0]
})

// 视图键
const todayIncompleteViewKey = computed(() => `daily::${today.value}::incomplete`)
const todayCompletedViewKey = computed(() => `daily::${today.value}::completed`)
const tomorrowViewKey = computed(() => `daily::${tomorrow.value}`)

onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing Daily shutdown view...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'daily-shutdown')

  // 确保未完成任务已加载（完成状态由运行期更新）
  await taskStore.fetchAllIncompleteTasks_DMA()

  logger.info(LogTags.VIEW_HOME, 'Daily shutdown initial state', {
    today: today.value,
    tomorrow: tomorrow.value,
  })
})
</script>

<template>
  <div class="daily-shutdown-view">
    <TwoRowLayout>
      <!-- 顶部：标题 -->
      <template #top>
        <div class="shutdown-header">
          <h2 class="shutdown-title">{{ $t('view.dailyShutdown.title') }}</h2>
        </div>
      </template>

      <!-- 底部：四列内容 -->
      <template #bottom>
        <div class="shutdown-grid">
          <!-- 今日未完成任务 -->
          <CutePane class="shutdown-card">
            <div class="card-header">
              <div class="card-title">
                <CuteIcon name="Circle" :size="18" />
                <span>{{ $t('view.dailyShutdown.todayIncomplete') }}</span>
              </div>
            </div>
            <div class="card-body kanban-card-body">
              <SimpleKanbanColumn
                :title="$t('view.dailyShutdown.todayIncomplete')"
                :subtitle="today"
                :view-key="todayIncompleteViewKey"
                :show-add-input="false"
                :hide-calendar-icon="true"
              />
            </div>
          </CutePane>

          <!-- 今日已完成任务 -->
          <CutePane class="shutdown-card">
            <div class="card-header">
              <div class="card-title">
                <CuteIcon name="Check" :size="18" />
                <span>{{ $t('view.dailyShutdown.todayCompleted') }}</span>
              </div>
            </div>
            <div class="card-body kanban-card-body">
              <SimpleKanbanColumn
                :title="$t('view.dailyShutdown.todayCompleted')"
                :subtitle="today"
                :view-key="todayCompletedViewKey"
                :show-add-input="false"
                :hide-calendar-icon="true"
              />
            </div>
          </CutePane>

          <!-- 明日看板列 -->
          <CutePane class="shutdown-card">
            <div class="card-header">
              <div class="card-title">
                <CuteIcon name="CalendarDays" :size="18" />
                <span>{{ $t('view.dailyShutdown.tomorrow') }}</span>
              </div>
              <span class="card-subtitle">{{ tomorrow }}</span>
            </div>
            <div class="card-body kanban-card-body">
              <SimpleKanbanColumn
                :title="$t('view.dailyShutdown.tomorrow')"
                :subtitle="tomorrow"
                :view-key="tomorrowViewKey"
                :show-add-input="true"
                :hide-calendar-icon="true"
              />
            </div>
          </CutePane>

          <!-- 今日小仪式（施工中） -->
          <CutePane class="shutdown-card">
            <div class="card-header">
              <div class="card-title">
                <CuteIcon name="Sparkles" :size="18" />
                <span>{{ $t('view.dailyShutdown.ritual') }}</span>
              </div>
            </div>
            <div class="card-body ritual-card-body">
              <UnderConstruction
                :title="$t('view.dailyShutdown.ritual')"
                :description="$t('view.dailyShutdown.ritualDesc')"
              />
            </div>
          </CutePane>
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<style scoped>
.daily-shutdown-view {
  width: 100%;
  height: 100%;
  background-color: var(--color-background-content);
  overflow: hidden;
}

.shutdown-header {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  width: 100%;
}

.shutdown-title {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.shutdown-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 1rem;
  padding: 1rem;
  height: 100%;
  box-sizing: border-box;
}

.shutdown-card {
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

.kanban-card-body {
  padding: 0.4rem;
}

.ritual-card-body {
  padding: 0;
}

.ritual-card-body :deep(.under-construction) {
  border-radius: 0;
  border: none;
}
</style>

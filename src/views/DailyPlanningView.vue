<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'
import ProjectDetailPanel from '@/components/organisms/ProjectDetailPanel.vue'
import ProjectListPanel from '@/components/organisms/ProjectListPanel.vue'
import StagingTaskGroups from '@/components/assembles/tasks/StagingTaskGroups.vue'
import DailyPlanningWizard from '@/components/organisms/DailyPlanningWizard.vue'
import VerticalToolbar from '@/components/functional/VerticalToolbar.vue'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import CuteCalendar from '@/components/assembles/calender/CuteCalendar.vue'
import HomeTemplatesPanel from '@/components/organisms/HomeTemplatesPanel.vue'
import { pipeline } from '@/cpu'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { useRegisterStore } from '@/stores/register'
import { logger, LogTags } from '@/infra/logging/logger'
import {
  getTodayDateString,
  getTomorrowDateString,
  parseDateString,
  toDateString,
} from '@/infra/utils/dateUtils'

// ==================== Router & i18n ====================
const router = useRouter()
const { t, locale } = useI18n()

// ==================== Stores ====================
const taskStore = useTaskStore()
const uiStore = useUIStore()
const registerStore = useRegisterStore()

// ==================== 状态 ====================
const today = ref(getTodayDateString())
const dailyViewKey = computed(() => `daily::${today.value}`)

// ==================== 步骤与右栏视图管理 ====================
type WizardStep = 1 | 2
type RightPaneView = 'staging' | 'projects' | 'daily' | 'calendar' | 'templates'

const currentStep = ref<WizardStep>(1)
const currentRightView = ref<RightPaneView>('staging')

// ==================== Projects 右栏视图（列表 -> 详情） ====================
// 约定：
// - undefined: 列表页（未选中任何项目）
// - null: “无项目”详情页
// - string: 项目详情页
const selectedProjectId = ref<string | null | undefined>(undefined)
const projectsLoadedOnce = ref(false)

const isProjectsList = computed(
  () => currentRightView.value === 'projects' && selectedProjectId.value === undefined
)
const isProjectDetail = computed(
  () => currentRightView.value === 'projects' && selectedProjectId.value !== undefined
)

async function ensureProjectsLoaded() {
  if (projectsLoadedOnce.value) return
  projectsLoadedOnce.value = true
  try {
    await pipeline.dispatch('project.fetch_all', {})
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Daily Planning: failed to load projects',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

async function openProjectDetail(projectId: string) {
  selectedProjectId.value = projectId
  try {
    await pipeline.dispatch('project_section.fetch_all', { project_id: projectId })
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Daily Planning: failed to load project sections',
      error instanceof Error ? error : new Error(String(error)),
      { projectId }
    )
  }
}

function backToProjectList() {
  selectedProjectId.value = undefined
}

async function handleSelectProject(id: string | null) {
  selectedProjectId.value = id

  // 只有选择了具体项目，才需要拉取 sections
  if (id) {
    await openProjectDetail(id)
  }
}

function goToProjectsMainView() {
  router.push('/projects')
}

// ==================== 当天视图（默认明天，跳过今天） ====================
const dailyRightDate = ref<string>(getTomorrowDateString())

function shiftDate(baseDate: string, offsetDays: number): string {
  const date = parseDateString(baseDate)
  date.setDate(date.getDate() + offsetDays)
  return toDateString(date)
}

function ensureNotToday(dateStr: string, direction: -1 | 1): string {
  const todayStr = getTodayDateString()
  if (dateStr !== todayStr) return dateStr
  // 跳过今天：继续往同方向再走一天
  return shiftDate(dateStr, direction)
}

function goToDailyTomorrow() {
  dailyRightDate.value = getTomorrowDateString()
}

function navigateDailyPrev() {
  const next = shiftDate(dailyRightDate.value, -1)
  dailyRightDate.value = ensureNotToday(next, -1)
}

function navigateDailyNext() {
  const next = shiftDate(dailyRightDate.value, 1)
  dailyRightDate.value = ensureNotToday(next, 1)
}

const dailyRightLabel = computed(() => {
  const todayStr = getTodayDateString()
  const yesterdayStr = shiftDate(todayStr, -1)
  const tomorrowStr = shiftDate(todayStr, 1)

  const date = parseDateString(dailyRightDate.value)
  const weekday = new Intl.DateTimeFormat(locale.value, { weekday: 'short' }).format(date)

  // 永远不显示“今天”
  if (dailyRightDate.value === tomorrowStr) return `${t('time.tomorrow')} ${weekday}`
  if (dailyRightDate.value === yesterdayStr) return `${t('time.yesterday')} ${weekday}`

  // 其他日期：用 locale 输出（避免在英文界面出现“X月X日”）
  return new Intl.DateTimeFormat(locale.value, {
    month: 'short',
    day: 'numeric',
    weekday: 'short',
  }).format(date)
})

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
    // 拉取该日期任务，避免未来日期数据不完整
    await taskStore.fetchDailyTasksRange_DMA(date, date)
  }
)

// 视图到步骤的映射关系
const viewToStep: Record<RightPaneView, WizardStep> = {
  staging: 1,
  projects: 1,
  daily: 1,
  templates: 1,
  calendar: 2,
}

// 工具栏配置：staging 下方增加 projects、templates
const toolbarConfig = computed(() => ({
  staging: { icon: 'Layers' as const, label: t('toolbar.staging') },
  projects: { icon: 'Folder' as const, label: t('toolbar.projects') },
  templates: { icon: 'FileText' as const, label: t('toolbar.templates') },
  daily: { icon: 'List' as const, label: t('toolbar.dailyTasks') },
  calendar: { icon: 'Calendar' as const, label: t('toolbar.calendar') },
}))

// ==================== 双向联动逻辑 ====================

// 工具栏切换 -> 自动更新步骤
function onRightViewChange(viewKey: string | null) {
  if (!viewKey) return
  const view = viewKey as RightPaneView
  currentRightView.value = view
  currentStep.value = viewToStep[view]

  // 离开 projects 视图时，重置项目详情状态
  if (view !== 'projects') {
    selectedProjectId.value = undefined
  } else {
    void ensureProjectsLoaded()
  }

  // 进入当天视图：默认显示明天（且永远跳过今天）
  if (view === 'daily') {
    dailyRightDate.value = getTomorrowDateString()
  }

  logger.info(LogTags.VIEW_HOME, 'Daily Planning: toolbar changed', {
    viewKey,
    step: currentStep.value,
  })
}

// Wizard Next 按钮 -> 切换到日历 + Step 2
function onWizardNext() {
  currentStep.value = 2
  currentRightView.value = 'calendar'
  selectedProjectId.value = undefined
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: wizard next', { step: 2 })
}

// Wizard Back 按钮
function onWizardBack() {
  if (currentStep.value === 1) {
    // Step 1 时返回主页
    router.push('/')
    logger.info(LogTags.VIEW_HOME, 'Daily Planning: returning to home')
  } else {
    // Step 2 时回到 Step 1
    currentStep.value = 1
    currentRightView.value = 'staging'
    selectedProjectId.value = undefined
    logger.info(LogTags.VIEW_HOME, 'Daily Planning: wizard back to step 1')
  }
}

// Wizard Done 按钮 -> 返回主页
function onWizardDone() {
  router.push('/')
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: completed, returning to home')
}

// ==================== 计算属性 ====================
const todayTasks = computed(() => {
  return taskStore.getTasksByDate_Mux(today.value)
})

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: Initializing...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'daily-planning')

  // 加载未完成任务
  await taskStore.fetchAllIncompleteTasks_DMA()

  logger.info(LogTags.VIEW_HOME, 'Daily Planning: Loaded tasks', {
    today: today.value,
    todayCount: todayTasks.value.length,
  })
})
</script>

<template>
  <div class="daily-planning-view">
    <!-- 左栏：每日规划向导 -->
    <div class="pane left-pane">
      <TwoRowLayout>
        <template #top>
          <!-- 上栏留空 -->
        </template>
        <template #bottom>
          <DailyPlanningWizard
            :step="currentStep"
            @next="onWizardNext"
            @back="onWizardBack"
            @done="onWizardDone"
          />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 分割线 -->
    <div class="divider"></div>

    <!-- 中栏：今天的任务列表 -->
    <div class="pane middle-pane">
      <TwoRowLayout>
        <template #top>
          <!-- 上栏留空 -->
        </template>
        <template #bottom>
          <TaskList
            :title="t('time.today')"
            :view-key="dailyViewKey"
            :show-add-input="true"
            :fill-remaining-space="true"
            :collapsible="false"
          />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 分割线 -->
    <div class="divider"></div>

    <!-- 右栏：受工具栏控制 -->
    <div class="pane right-pane">
      <TwoRowLayout>
        <template #top>
          <!-- Projects 详情页：左上角返回图标 -->
          <div v-if="isProjectDetail" class="right-pane-header">
            <button
              class="back-icon-btn"
              @click="backToProjectList"
              :aria-label="t('view.dailyPlanning.back')"
              :title="t('view.dailyPlanning.back')"
            >
              ←
            </button>
          </div>

          <!-- 当天视图控制栏 -->
          <div v-else-if="currentRightView === 'daily'" class="daily-controls">
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
                :title="t('view.dailyPlanning.dailyTasksNav.todayJump')"
                @click="goToDailyTomorrow"
              >
                <span class="today-text">{{ t('time.today') }}</span>
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
          <!-- 暂存区视图 -->
          <StagingTaskGroups v-if="currentRightView === 'staging'" />
          <!-- Projects 视图 -->
          <div v-else-if="currentRightView === 'projects'" class="projects-wrapper">
            <!-- 列表页：直接复用项目页同款 ProjectListPanel（不要在这里手写列表） -->
            <ProjectListPanel
              v-if="isProjectsList"
              :selected-id="undefined"
              @select-project="handleSelectProject"
              @create-project="goToProjectsMainView"
              @edit-project="goToProjectsMainView"
              @add-section="goToProjectsMainView"
            />

            <!-- 详情页 -->
            <ProjectDetailPanel v-else-if="isProjectDetail" :project-id="selectedProjectId" />
          </div>
          <!-- 当天视图（默认明天，跳过今天） -->
          <TaskList
            v-else-if="currentRightView === 'daily'"
            :title="dailyRightLabel"
            :view-key="`daily::${dailyRightDate}`"
            :show-add-input="true"
            :fill-remaining-space="true"
            :collapsible="false"
          />
          <!-- 模板视图 -->
          <HomeTemplatesPanel v-else-if="currentRightView === 'templates'" />
          <!-- 日历视图 -->
          <div v-else-if="currentRightView === 'calendar'" class="calendar-wrapper">
            <CuteCalendar :current-date="today" :view-type="'day'" :days="1" :zoom="1" />
          </div>
        </template>
      </TwoRowLayout>
    </div>

    <!-- 工具栏 -->
    <VerticalToolbar
      :view-config="toolbarConfig"
      :current-view="currentRightView"
      @view-change="onRightViewChange"
    />

    <!-- 任务编辑器弹窗 -->
    <TaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
  </div>
</template>

<style scoped>
/* ==================== 视图容器 ==================== */
.daily-planning-view {
  width: 100%;
  height: 100%;
  display: flex;
  gap: 3rem;
  overflow: hidden;
  background-color: var(--color-background-content, #f0f);
}

/* ==================== 三栏布局 ==================== */
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

/* ==================== 分割线 ==================== */
.divider {
  width: 1px;
  height: 100%;
  background-color: var(--color-border-adaptive-light-subtle-dark-none, #f0f);
  flex-shrink: 0;
}

/* ==================== 日历包装器 ==================== */
.calendar-wrapper {
  height: 100%;
  overflow: hidden;
}

/* ==================== 当天视图（右栏） ==================== */
.daily-controls {
  width: 100%;
  height: 100%;
  padding: 1.2rem 0.8rem 1.2rem 1.6rem;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 1.2rem;
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

.daily-controls .nav-btn:active {
  transform: scale(0.95);
}

.daily-controls .today-nav-btn {
  padding: 0 1.4rem;
}

.daily-controls .today-text {
  font-size: 1.4rem;
  font-weight: 600;
  line-height: 1.4;
}

/* ==================== Projects 右栏 ==================== */
.right-pane-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0 1rem;
  height: 100%;
}

.back-icon-btn {
  width: 3.2rem;
  height: 3.2rem;
  border-radius: 0.8rem;
  border: 1px solid var(--color-border-default, #f0f);
  background: transparent;
  color: var(--color-text-secondary, #f0f);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  font-size: 1.6rem;
  line-height: 1;
}

.back-icon-btn:hover {
  background: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.projects-wrapper {
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>

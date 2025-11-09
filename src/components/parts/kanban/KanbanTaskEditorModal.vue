<script setup lang="ts">
import { computed, ref, watch, onMounted, nextTick } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import { useRecurrenceStore } from '@/stores/recurrence'
import { useViewStore } from '@/stores/view'
import { pipeline } from '@/cpu'
import { RRule } from 'rrule'
import type { TaskDetail } from '@/types/dtos'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import RecurrenceConfigDialog from '@/components/parts/recurrence/RecurrenceConfigDialog.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString, parseDateString, toUtcIsoString } from '@/infra/utils/dateUtils'
import draggable from 'vuedraggable'
import { useRecurrenceOperations } from '@/composables/useRecurrenceOperations'

interface Subtask {
  id: string
  title: string
  is_completed: boolean
  sort_order: string
}

const props = defineProps<{
  taskId: string | null
  viewKey?: string // View context key (e.g., 'daily::2025-10-10', 'misc::staging')
}>()

const emit = defineEmits(['close'])

const taskStore = useTaskStore()
const areaStore = useAreaStore()
const recurrenceStore = useRecurrenceStore()
const viewStore = useViewStore()
const recurrenceOps = useRecurrenceOperations()

// 本地编辑状态
const titleInput = ref('')
const glanceNote = ref('')
const detailNote = ref('')
const selectedAreaId = ref<string | null>(null)
const newSubtaskTitle = ref('')
const isTitleEditing = ref(false)
const showAreaSelector = ref(false)
const showDueDatePicker = ref(false)
const dueDateInput = ref('') // YYYY-MM-DD format
const dueDateType = ref<'SOFT' | 'HARD'>('SOFT')
const glanceNoteTextarea = ref<HTMLTextAreaElement | null>(null)
const detailNoteTextarea = ref<HTMLTextAreaElement | null>(null)
const mouseDownOnOverlay = ref(false)
const showRecurrenceDialog = ref(false)
const currentRecurrence = ref<any>(null)

const task = computed(() => {
  return props.taskId ? taskStore.getTaskById_Mux(props.taskId) : null
})

// 使用 ref 而不是 computed，以便 vuedraggable 可以修改
const subtasks = ref<Subtask[]>([])

// 监听 task 变化，同步 subtasks
watch(
  () => task.value?.subtasks,
  (newSubtasks) => {
    if (newSubtasks) {
      subtasks.value = [...newSubtasks]
    } else {
      subtasks.value = []
    }
  },
  { immediate: true }
)

const selectedArea = computed(() => {
  return selectedAreaId.value ? areaStore.getAreaById(selectedAreaId.value) : null
})

// 获取今天的日期（用于在场状态判断）
const todayDate = computed(() => getTodayDateString())

// 获取今天的 schedule outcome
const currentScheduleOutcome = computed(() => {
  if (!task.value?.schedules || !todayDate.value) return null

  const todaySchedule = task.value.schedules.find((s) => s.scheduled_day === todayDate.value)
  return todaySchedule?.outcome || null
})

// 今天是否已记录在场
const isPresenceLogged = computed(() => {
  return currentScheduleOutcome.value === 'presence_logged'
})

// 循环规则的人类可读描述
const recurrenceDescription = computed(() => {
  if (!currentRecurrence.value) return null
  try {
    const rule = RRule.fromString(currentRecurrence.value.rule)
    return rule.toText()
  } catch (e) {
    return currentRecurrence.value.rule
  }
})

// 自动调整 textarea 高度
function autoResizeTextarea(textarea: HTMLTextAreaElement) {
  textarea.style.height = 'auto'
  textarea.style.height = textarea.scrollHeight + 'px'
}

// 初始化所有 textarea 的高度
function initTextareaHeights() {
  if (glanceNoteTextarea.value) {
    autoResizeTextarea(glanceNoteTextarea.value)
  }
  if (detailNoteTextarea.value) {
    autoResizeTextarea(detailNoteTextarea.value)
  }
}

// 加载循环规则（如果存在）
async function loadRecurrence() {
  if (!task.value) return

  // ✅ 修复：使用 task.recurrence_id 直接查找循环规则
  if (task.value.recurrence_id) {
    // 🔥 使用CPU指令获取所有循环规则
    await pipeline.dispatch('recurrence.fetch_all', {})

    // 直接通过 recurrence_id 查找
    const recurrence = recurrenceStore.getRecurrenceById(task.value.recurrence_id)
    if (recurrence) {
      currentRecurrence.value = recurrence
      logger.info(LogTags.COMPONENT_KANBAN, 'Loaded recurrence for task', {
        taskId: task.value.id,
        recurrenceId: recurrence.id,
      })
    } else {
      logger.warn(LogTags.COMPONENT_KANBAN, 'Recurrence not found', {
        taskId: task.value.id,
        recurrenceId: task.value.recurrence_id,
      })
    }
  } else {
    currentRecurrence.value = null
  }
}

// 当弹窗打开时，获取任务详情
onMounted(async () => {
  if (props.taskId) {
    const detail = (await taskStore.fetchTaskDetail_DMA(props.taskId)) as TaskDetail | null
    if (detail) {
      titleInput.value = detail.title
      glanceNote.value = detail.glance_note || ''
      detailNote.value = detail.detail_note || ''
      selectedAreaId.value = detail.area_id || null

      // 初始化截止日期
      if (detail.due_date) {
        const datePart = detail.due_date.date.split('T')[0]
        dueDateInput.value = datePart || ''
        dueDateType.value = detail.due_date.type
      } else {
        dueDateInput.value = ''
        dueDateType.value = 'SOFT'
      }

      // 等待 DOM 更新后调整 textarea 高度
      await nextTick()
      initTextareaHeights()

      // 加载循环规则
      await loadRecurrence()
    }
  }
})

watch(
  () => props.taskId,
  async (newTaskId) => {
    if (newTaskId) {
      const detail = (await taskStore.fetchTaskDetail_DMA(newTaskId)) as TaskDetail | null
      if (detail) {
        titleInput.value = detail.title
        glanceNote.value = detail.glance_note || ''
        detailNote.value = detail.detail_note || ''
        selectedAreaId.value = detail.area_id || null

        // 初始化截止日期
        if (detail.due_date) {
          const datePart = detail.due_date.date.split('T')[0]
          dueDateInput.value = datePart || ''
          dueDateType.value = detail.due_date.type
        } else {
          dueDateInput.value = ''
          dueDateType.value = 'SOFT'
        }

        // 等待 DOM 更新后调整 textarea 高度
        await nextTick()
        initTextareaHeights()

        // 加载循环规则
        await loadRecurrence()
      }
    }
  }
)

async function handleCompleteChange(isChecked: boolean) {
  if (!props.taskId) return

  if (isChecked) {
    await pipeline.dispatch('task.complete', { id: props.taskId })
  } else {
    await pipeline.dispatch('task.reopen', { id: props.taskId })
  }
}

async function handlePresenceToggle(isChecked: boolean) {
  if (!props.taskId || !todayDate.value) return

  // 使用新的勾选状态来决定 outcome（后端API使用大写格式）
  const newOutcome = isChecked ? 'PRESENCE_LOGGED' : 'PLANNED'

  await pipeline.dispatch('schedule.update', {
    task_id: props.taskId,
    scheduled_day: todayDate.value,
    updates: { outcome: newOutcome },
  })
}

async function updateTitle() {
  if (!props.taskId || !task.value || titleInput.value === task.value.title) return
  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: { title: titleInput.value },
  })
  isTitleEditing.value = false
}

async function updateGlanceNote() {
  if (!props.taskId || !task.value) return
  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: { glance_note: glanceNote.value || null },
  })
}

async function updateDetailNote() {
  if (!props.taskId || !task.value) return
  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: { detail_note: detailNote.value || null },
  })
}

async function updateArea(areaId: string | null) {
  if (!props.taskId || !task.value) return
  selectedAreaId.value = areaId
  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: { area_id: areaId },
  })
  showAreaSelector.value = false
}

// 保存截止日期
async function saveDueDate() {
  if (!props.taskId || !task.value || !dueDateInput.value) return

  // 将日期字符串转为 UTC 当天零点（ISO 格式）
  const dateObj = parseDateString(dueDateInput.value) // 本地时区的当天零点
  const utcDate = new Date(
    Date.UTC(dateObj.getFullYear(), dateObj.getMonth(), dateObj.getDate(), 0, 0, 0, 0)
  )

  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: {
      due_date: toUtcIsoString(utcDate),
      due_date_type: dueDateType.value,
    },
  })

  showDueDatePicker.value = false
}

// 清除截止日期
async function clearDueDate() {
  if (!props.taskId || !task.value) return

  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: {
      due_date: null,
      due_date_type: null,
    },
  })

  dueDateInput.value = ''
  dueDateType.value = 'SOFT'
  showDueDatePicker.value = false
}

async function handleAddSubtask() {
  if (!props.taskId || !newSubtaskTitle.value.trim()) return

  const newSubtask: Subtask = {
    id: crypto.randomUUID(),
    title: newSubtaskTitle.value.trim(),
    is_completed: false,
    sort_order: `subtask_${Date.now()}`,
  }

  const updatedSubtasks = [...subtasks.value, newSubtask]

  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: { subtasks: updatedSubtasks },
  })

  newSubtaskTitle.value = ''
}

async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  if (!props.taskId) return

  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: { subtasks: updatedSubtasks },
  })
}

async function handleDeleteSubtask(subtaskId: string) {
  if (!props.taskId) return

  const updatedSubtasks = subtasks.value.filter((subtask) => subtask.id !== subtaskId)

  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: { subtasks: updatedSubtasks },
  })
}

async function handleSubtaskReorder() {
  if (!props.taskId) return

  // 更新 sort_order
  const updatedSubtasks = subtasks.value.map((subtask, index) => ({
    ...subtask,
    sort_order: `subtask_${Date.now()}_${index}`,
  }))

  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: { subtasks: updatedSubtasks },
  })
}

function handleOverlayMouseDown() {
  mouseDownOnOverlay.value = true
}

function handleOverlayClick() {
  // 只有在 overlay 上按下鼠标时才关闭
  if (mouseDownOnOverlay.value) {
    emit('close')
  }
  mouseDownOnOverlay.value = false
}

function handleCardMouseDown() {
  mouseDownOnOverlay.value = false
}

function handleClose() {
  emit('close')
}

function openRecurrenceDialog() {
  showRecurrenceDialog.value = true
}

async function handleRecurrenceSuccess() {
  // 循环创建成功后，重新加载循环规则以显示
  console.log('Recurrence created successfully')
  await loadRecurrence()
}

async function handleStopRepeating() {
  const taskData = task.value as any
  if (!currentRecurrence.value || !taskData?.recurrence_original_date) return

  const instanceDate = taskData.recurrence_original_date
  if (
    confirm(
      `确定停止此循环吗？\n将从 ${instanceDate} 之后停止生成新任务。\n已生成的任务不会被删除。`
    )
  ) {
    try {
      logger.info(LogTags.STORE_RECURRENCE, 'Stopping recurrence', {
        recurrenceId: currentRecurrence.value.id,
        instanceDate,
      })
      await recurrenceOps.stopRepeating(currentRecurrence.value.id, instanceDate)

      await loadRecurrence()
    } catch (error) {
      console.error('Failed to stop repeating:', error)
      alert('操作失败，请重试')
    }
  }
}

async function handleExtendRecurrence() {
  if (!currentRecurrence.value) return

  if (confirm('确定继续此循环吗？将清除结束日期，继续生成新任务。')) {
    try {
      // 🔥 使用CPU指令更新循环规则
      await pipeline.dispatch('recurrence.update', {
        id: currentRecurrence.value.id,
        end_date: null,
      })
      // 重新加载以更新状态
      await loadRecurrence()
      await viewStore.refreshAllMountedDailyViews()
    } catch (error) {
      console.error('Failed to extend recurrence:', error)
      alert('操作失败，请重试')
    }
  }
}

async function handleDeleteRecurrence() {
  if (!currentRecurrence.value) return

  if (confirm('确定删除这个循环规则吗？已生成的任务不会被删除。')) {
    try {
      await recurrenceOps.deleteAllInstancesAndStop(currentRecurrence.value.id)
      currentRecurrence.value = null
      await loadRecurrence()
    } catch (error) {
      console.error('Failed to delete recurrence:', error)
      alert('删除失败，请重试')
    }
  }
}

async function handleToggleRecurrenceActive() {
  if (!currentRecurrence.value) return

  try {
    // 🔥 使用CPU指令更新循环规则
    await pipeline.dispatch('recurrence.update', {
      id: currentRecurrence.value.id,
      is_active: !currentRecurrence.value.is_active,
    })
    // 重新加载以更新状态
    await loadRecurrence()
    await viewStore.refreshAllMountedDailyViews()
  } catch (error) {
    console.error('Failed to toggle recurrence:', error)
    alert('操作失败，请重试')
  }
}
</script>

<template>
  <div
    class="modal-overlay"
    @mousedown.self="handleOverlayMouseDown"
    @click.self="handleOverlayClick"
  >
    <CuteCard class="editor-card" @mousedown="handleCardMouseDown" @click.stop>
      <div v-if="task" class="content-wrapper">
        <!-- 第一栏：卡片标题栏 -->
        <div class="card-header-row">
          <div class="left-section">
            <!-- 区域标签 -->
            <div class="area-tag-wrapper" @click="showAreaSelector = !showAreaSelector">
              <AreaTag
                v-if="selectedArea"
                :name="selectedArea.name"
                :color="selectedArea.color"
                size="normal"
              />
              <div v-else class="no-area-placeholder">
                <CuteIcon name="Hash" :size="16" />
                <span>无区域</span>
              </div>
            </div>

            <!-- 简易区域选择器 -->
            <div v-if="showAreaSelector" class="area-selector-dropdown">
              <div
                v-for="area in Array.from(areaStore.areas.values())"
                :key="area.id"
                class="area-option"
                @click="updateArea(area.id)"
              >
                <AreaTag :name="area.name" :color="area.color" size="small" />
              </div>
              <div class="area-option" @click="updateArea(null)">
                <span class="no-area-text">清除区域</span>
              </div>
            </div>
          </div>

          <div class="right-section">
            <!-- 截止日期选择器 -->
            <div class="due-date-wrapper">
              <button class="due-date-button" @click="showDueDatePicker = !showDueDatePicker">
                <span v-if="task.due_date">{{
                  new Date(task.due_date.date).toLocaleDateString()
                }}</span>
                <span v-else class="placeholder">设置截止日期</span>
              </button>

              <!-- 截止日期选择器弹窗 -->
              <div v-if="showDueDatePicker" class="due-date-picker-popup" @click.stop>
                <div class="picker-section">
                  <label class="picker-label">日期</label>
                  <input type="date" v-model="dueDateInput" class="date-input" />
                </div>

                <div class="picker-section">
                  <label class="picker-label">类型</label>
                  <div class="deadline-type-buttons">
                    <button
                      class="type-button"
                      :class="{ active: dueDateType === 'SOFT' }"
                      @click="dueDateType = 'SOFT'"
                    >
                      软截止
                    </button>
                    <button
                      class="type-button"
                      :class="{ active: dueDateType === 'HARD' }"
                      @click="dueDateType = 'HARD'"
                    >
                      硬截止
                    </button>
                  </div>
                </div>

                <div class="picker-actions">
                  <button class="action-button save-button" @click="saveDueDate">保存</button>
                  <button
                    v-if="task.due_date"
                    class="action-button clear-button"
                    @click="clearDueDate"
                  >
                    清除
                  </button>
                  <button class="action-button cancel-button" @click="showDueDatePicker = false">
                    取消
                  </button>
                </div>
              </div>
            </div>

            <!-- 循环设置按钮 -->
            <button
              class="recurrence-button"
              :class="{ active: currentRecurrence }"
              @click="openRecurrenceDialog"
              :title="currentRecurrence ? '查看循环规则' : '设置为循环任务'"
            >
              <CuteIcon name="RefreshCw" :size="18" />
            </button>

            <!-- × 按钮 -->
            <button class="close-button" @click="handleClose">×</button>
          </div>
        </div>

        <!-- 循环规则展示区 -->
        <div v-if="currentRecurrence" class="recurrence-info-section">
          <div class="recurrence-header">
            <div class="recurrence-icon-wrapper">
              <CuteIcon name="RefreshCw" :size="16" />
            </div>
            <div class="recurrence-content">
              <div class="recurrence-title">
                循环任务规则
                <span class="status-badge" :class="{ active: currentRecurrence.is_active }">
                  {{ currentRecurrence.is_active ? '激活中' : '已暂停' }}
                </span>
              </div>
              <div class="recurrence-description">{{ recurrenceDescription }}</div>
              <div
                v-if="currentRecurrence.start_date || currentRecurrence.end_date"
                class="recurrence-dates"
              >
                <span v-if="currentRecurrence.start_date"
                  >开始: {{ currentRecurrence.start_date }}</span
                >
                <span v-if="currentRecurrence.end_date" class="date-separator">-</span>
                <span v-if="currentRecurrence.end_date"
                  >结束: {{ currentRecurrence.end_date }}</span
                >
              </div>
            </div>
            <div class="recurrence-actions">
              <!-- Stop Repeating / Extend -->
              <button
                v-if="(task as any)?.recurrence_original_date && !currentRecurrence.end_date"
                class="action-icon-btn stop"
                @click="handleStopRepeating"
                title="停止重复（从此日期之后不再生成）"
              >
                <CuteIcon name="X" :size="14" />
              </button>
              <button
                v-if="currentRecurrence.end_date"
                class="action-icon-btn extend"
                @click="handleExtendRecurrence"
                title="继续循环（清除结束日期）"
              >
                <CuteIcon name="Check" :size="14" />
              </button>
              <!-- 暂停/激活 -->
              <button
                class="action-icon-btn"
                @click="handleToggleRecurrenceActive"
                :title="currentRecurrence.is_active ? '暂停' : '激活'"
              >
                <CuteIcon :name="currentRecurrence.is_active ? 'Pause' : 'Play'" :size="14" />
              </button>
              <!-- 删除 -->
              <button
                class="action-icon-btn danger"
                @click="handleDeleteRecurrence"
                title="删除规则"
              >
                <CuteIcon name="Trash2" :size="14" />
              </button>
            </div>
          </div>
        </div>

        <!-- 第二栏：任务标题栏 -->
        <div class="title-row">
          <CuteCheckbox
            :checked="task.is_completed"
            size="large"
            variant="check"
            @update:checked="handleCompleteChange"
          />
          <CuteCheckbox
            :checked="isPresenceLogged"
            size="large"
            variant="star"
            @update:checked="handlePresenceToggle"
          />
          <input
            v-model="titleInput"
            class="title-input"
            :class="{ completed: task.is_completed }"
            @blur="updateTitle"
            @keydown.enter="updateTitle"
          />
        </div>

        <!-- 第三栏：Glance Note 区域 -->
        <div class="note-area glance-note-area">
          <div
            v-if="!glanceNote && !isTitleEditing"
            class="note-placeholder"
            @click="isTitleEditing = true"
          >
            快速概览笔记...
          </div>
          <textarea
            ref="glanceNoteTextarea"
            v-model="glanceNote"
            class="note-textarea"
            placeholder="快速概览笔记..."
            rows="1"
            @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
            @blur="updateGlanceNote"
          ></textarea>
        </div>

        <!-- 分割线 -->
        <div class="separator"></div>

        <!-- 第四栏：子任务编辑区 -->
        <div class="subtasks-section">
          <div class="subtasks-header">子任务</div>
          <draggable
            v-model="subtasks"
            item-key="id"
            class="subtasks-list"
            handle=".drag-handle"
            @end="handleSubtaskReorder"
          >
            <template #item="{ element: subtask }">
              <div class="subtask-item">
                <div class="drag-handle">⋮⋮</div>
                <CuteCheckbox
                  :checked="subtask.is_completed"
                  size="small"
                  @update:checked="
                    (isChecked: boolean) => handleSubtaskStatusChange(subtask.id, isChecked)
                  "
                />
                <span class="subtask-title" :class="{ completed: subtask.is_completed }">
                  {{ subtask.title }}
                </span>
                <button class="delete-button" @click="handleDeleteSubtask(subtask.id)">×</button>
              </div>
            </template>
          </draggable>
          <div class="add-subtask-form">
            <input
              v-model="newSubtaskTitle"
              class="add-subtask-input"
              placeholder="添加子任务..."
              @keydown.enter="handleAddSubtask"
            />
          </div>
        </div>

        <!-- 分割线 -->
        <div class="separator"></div>

        <!-- 第五栏：细节笔记区 -->
        <div class="note-area detail-note-area">
          <div v-if="!detailNote" class="note-placeholder">详细笔记...</div>
          <textarea
            ref="detailNoteTextarea"
            v-model="detailNote"
            class="note-textarea"
            placeholder="详细笔记..."
            rows="1"
            @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
            @blur="updateDetailNote"
          ></textarea>
        </div>
      </div>
    </CuteCard>

    <!-- 循环配置对话框 -->
    <RecurrenceConfigDialog
      v-if="showRecurrenceDialog && task"
      :task="task"
      :view-key="props.viewKey"
      :open="showRecurrenceDialog"
      @close="showRecurrenceDialog = false"
      @success="handleRecurrenceSuccess"
    />
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgb(0 0 0 / 50%);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.editor-card {
  width: 70rem;
  max-width: 90vw;
  max-height: 90vh;
  padding: 2.5rem;
  border: 1px solid var(--color-border-default);
  background-color: var(--color-card-available);
  border-radius: 0.8rem;
  overflow-y: auto;
}

.content-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

/* 第一栏：卡片标题栏 */
.card-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1.5rem;
  border-bottom: 2px solid var(--color-separator);
}

.left-section {
  display: flex;
  align-items: center;
  position: relative;
}

.area-tag-wrapper {
  cursor: pointer;
  transition: opacity 0.2s;
}

.area-tag-wrapper:hover {
  opacity: 0.7;
}

.area-selector-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 0.5rem;
  background: var(--color-card-available);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  box-shadow: 0 4px 12px rgb(0 0 0 / 15%);
  z-index: 100;
  min-width: 20rem;
  max-height: 30rem;
  overflow-y: auto;
}

.area-option {
  padding: 0.8rem 1.2rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.area-option:hover {
  background-color: var(--color-background-soft, #f9f9f9);
}

.no-area-text {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
}

.no-area-placeholder {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
  padding: 0.4rem 0.8rem;
  border: 1px dashed var(--color-border-default);
  border-radius: 0.4rem;
}

/* ✅ 移除 .hash-symbol 样式，现在使用 CuteIcon 组件 */

.right-section {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.due-date-wrapper {
  position: relative;
}

.due-date-button {
  padding: 0.6rem 1.2rem;
  font-size: 1.3rem;
  color: var(--color-text-secondary);
  background: transparent;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s;
}

.due-date-button:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.due-date-button .placeholder {
  color: var(--color-text-tertiary);
}

/* 截止日期选择器弹窗 */
.due-date-picker-popup {
  position: absolute;
  top: calc(100% + 0.4rem);
  right: 0;
  width: 26rem;
  background: white;
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  box-shadow: 0 4px 16px rgb(0 0 0 / 15%);
  padding: 1.5rem;
  z-index: 1000;
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

.picker-section {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.picker-label {
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.date-input {
  padding: 0.6rem 1rem;
  font-size: 1.3rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  color: var(--color-text-primary);
}

.deadline-type-buttons {
  display: flex;
  gap: 0.8rem;
}

.type-button {
  flex: 1;
  padding: 0.6rem 1rem;
  font-size: 1.3rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.type-button:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.type-button.active {
  background-color: var(--color-primary, #1976d2);
  color: white;
  border-color: var(--color-primary, #1976d2);
}

.picker-actions {
  display: flex;
  gap: 0.8rem;
  margin-top: 0.5rem;
}

.action-button {
  flex: 1;
  padding: 0.6rem 1rem;
  font-size: 1.3rem;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.15s;
}

.save-button {
  background-color: var(--color-primary, #1976d2);
  color: white;
}

.save-button:hover {
  background-color: var(--color-primary-dark, #1565c0);
}

.clear-button {
  background-color: #f44336;
  color: white;
}

.clear-button:hover {
  background-color: #d32f2f;
}

.cancel-button {
  background-color: #e0e0e0;
  color: var(--color-text-primary);
}

.cancel-button:hover {
  background-color: #bdbdbd;
}

.recurrence-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  padding: 0;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  background-color: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.recurrence-button:hover {
  border-color: var(--color-primary);
  background-color: var(--color-primary);
  color: white;
}

.recurrence-button.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background-color: rgb(25 118 210 / 10%);
}

.recurrence-button.active:hover {
  background-color: var(--color-primary);
  color: white;
}

/* 循环规则展示区 */
.recurrence-info-section {
  padding: 1.5rem;
  background: linear-gradient(135deg, #e3f2fd 0%, #f3e5f5 100%);
  border-radius: 0.8rem;
  border: 1px solid #bbdefb;
  margin-top: -0.5rem;
}

.recurrence-header {
  display: flex;
  align-items: flex-start;
  gap: 1.2rem;
}

.recurrence-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  background: var(--color-primary, #1976d2);
  color: white;
  border-radius: 0.6rem;
  flex-shrink: 0;
}

.recurrence-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.recurrence-title {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.status-badge {
  padding: 0.2rem 0.8rem;
  border-radius: 1rem;
  font-size: 1.1rem;
  font-weight: 500;
  background: #e0e0e0;
  color: #666;
}

.status-badge.active {
  background: #4caf50;
  color: white;
}

.recurrence-description {
  font-size: 1.3rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.recurrence-dates {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
}

.date-separator {
  color: var(--color-text-tertiary);
}

.recurrence-actions {
  display: flex;
  gap: 0.6rem;
  flex-shrink: 0;
}

.action-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  padding: 0;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  background: white;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.action-icon-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: rgb(25 118 210 / 10%);
}

.action-icon-btn.danger:hover {
  border-color: #f44336;
  color: #f44336;
  background: rgb(244 67 54 / 10%);
}

.action-icon-btn.stop:hover {
  border-color: #ff9800;
  color: #ff9800;
  background: rgb(255 152 0 / 10%);
}

.action-icon-btn.extend:hover {
  border-color: #4caf50;
  color: #4caf50;
  background: rgb(76 175 80 / 10%);
}

.close-button {
  font-size: 3rem;
  line-height: 1;
  color: var(--color-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.2s;
}

.close-button:hover {
  color: var(--color-text-primary);
}

/* 第二栏：任务标题栏 */
.title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.title-input {
  flex: 1;
  font-size: 2.4rem;
  font-weight: 600;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
  padding: 0.5rem 0;
  border-bottom: 2px solid transparent;
  transition: border-color 0.2s;
}

.title-input:focus {
  border-bottom-color: var(--color-primary);
}

.title-input.completed {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

/* 笔记区域 */
.note-area {
  position: relative;
  min-height: 4rem;
}

.note-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  padding: 1rem;
  font-size: 1.5rem;
  color: var(--color-text-tertiary);
  cursor: text;
  pointer-events: none;
}

.note-textarea {
  width: 100%;
  font-family: inherit;
  font-size: 1.5rem;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  padding: 1rem;
  border-radius: 0.4rem;
  overflow: hidden;
  min-height: 2rem;
}

.note-textarea:hover {
  background: transparent;
}

.note-textarea:focus {
  background: transparent;
}

.note-textarea::placeholder {
  color: transparent;
}

/* 分割线 */
.separator {
  height: 1px;
  background-color: var(--color-separator);
}

/* 第四栏：子任务区 */
.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.subtasks-header {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.subtasks-list {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.6rem;
  border-radius: 0.4rem;
  transition: background-color 0.2s;
  cursor: move;
}

.subtask-item:hover {
  background-color: var(--color-background-soft, #f9f9f9);
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
  line-height: 1;
  user-select: none;
}

.drag-handle:active {
  cursor: grabbing;
}

.subtask-title {
  flex: 1;
  font-size: 1.6rem;
  color: var(--color-text-primary);
}

.subtask-title.completed {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.delete-button {
  font-size: 2rem;
  line-height: 1;
  color: var(--color-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition:
    opacity 0.2s,
    color 0.2s;
}

.delete-button:hover {
  color: var(--color-danger, #ff4d4f);
}

.subtask-item:hover .delete-button {
  opacity: 1;
}

.add-subtask-form {
  margin-top: 0.5rem;
}

.add-subtask-input {
  width: 100%;
  padding: 1rem;
  font-size: 1.5rem;
  border: 1px dashed var(--color-border-default);
  border-radius: 0.4rem;
  background-color: transparent;
  color: var(--color-text-primary);
  transition: all 0.2s;
}

.add-subtask-input:focus {
  outline: none;
  border-style: solid;
  border-color: var(--color-primary);
  background-color: var(--color-background-soft, #f9f9f9);
}

.add-subtask-input::placeholder {
  color: var(--color-text-tertiary);
}
</style>

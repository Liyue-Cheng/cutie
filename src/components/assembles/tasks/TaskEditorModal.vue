<script setup lang="ts">
import { computed, ref, watch, onMounted, nextTick } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import { useRecurrenceStore } from '@/stores/recurrence'
import { pipeline } from '@/cpu'
import { RRule } from 'rrule'
import type { TaskDetail } from '@/types/dtos'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteDualModeCheckbox from '@/components/parts/CuteDualModeCheckbox.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import RecurrenceConfigDialog from '@/components/parts/recurrence/RecurrenceConfigDialog.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'
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

// 🔥 监听任务是否存在，如果任务被删除则自动关闭编辑框
watch(task, (newTask) => {
  // 如果有 taskId 但任务不存在（被删除了），则自动关闭
  if (props.taskId && !newTask) {
    logger.info(LogTags.COMPONENT_KANBAN, 'Task no longer exists, closing editor', {
      taskId: props.taskId,
    })
    emit('close')
  }
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

// 主按钮状态（双模式）
const mainCheckboxState = computed<'completed' | 'present' | null>(() => {
  if (task.value?.is_completed) {
    return 'completed'
  }
  if (isPresenceLogged.value) {
    return 'present'
  }
  return null
})

// 循环规则的人类可读描述
const recurrenceDescription = computed(() => {
  if (!currentRecurrence.value) return null
  try {
    const rule = RRule.fromString(currentRecurrence.value.rule)
    let text = rule.toText()

    // 简单的汉化处理
    const map: Record<string, string> = {
      'every day': '每天',
      'every week': '每周',
      'every month': '每月',
      'every year': '每年',
    }

    const lowerText = text.toLowerCase()
    if (map[lowerText]) {
      return map[lowerText]
    }

    return text
      .replace(/^every day/i, '每天')
      .replace(/^every week/i, '每周')
      .replace(/^every month/i, '每月')
      .replace(/^every year/i, '每年')
      .replace(/ on /gi, ' ')
      .replace(/until/gi, '直到')
  } catch (e) {
    return currentRecurrence.value.rule
  }
})

// 判断循环是否激活（根据end_date）
const isRecurrenceActive = computed(() => {
  if (!currentRecurrence.value) return false

  // 如果没有结束日期，说明循环仍在激活状态
  if (!currentRecurrence.value.end_date) return true

  // 如果有结束日期，比较是否大于今天
  const today = getTodayDateString()
  return currentRecurrence.value.end_date > today
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
    // 先尝试从 store 中获取，避免不必要的网络请求
    let recurrence = recurrenceStore.getRecurrenceById(task.value.recurrence_id)

    // 如果 store 中没有，再异步获取
    if (!recurrence) {
      await pipeline.dispatch('recurrence.fetch_all', {})
      recurrence = recurrenceStore.getRecurrenceById(task.value.recurrence_id)
    }

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
    // 🔥 先尝试同步加载循环规则（如果store中已有数据）
    const cardTask = taskStore.getTaskById_Mux(props.taskId)
    if (cardTask?.recurrence_id) {
      const recurrence = recurrenceStore.getRecurrenceById(cardTask.recurrence_id)
      if (recurrence) {
        currentRecurrence.value = recurrence
      }
    }

    const detail = (await taskStore.fetchTaskDetail_DMA(props.taskId)) as TaskDetail | null
    if (detail) {
      titleInput.value = detail.title
      glanceNote.value = detail.glance_note || ''
      detailNote.value = detail.detail_note || ''
      selectedAreaId.value = detail.area_id || null

      // 初始化截止日期
      if (detail.due_date) {
        // ✅ due_date.date 现在是 YYYY-MM-DD 格式，直接使用
        dueDateInput.value = detail.due_date.date
        dueDateType.value = detail.due_date.type
      } else {
        dueDateInput.value = ''
        dueDateType.value = 'SOFT'
      }

      // 等待 DOM 更新后调整 textarea 高度
      await nextTick()
      initTextareaHeights()

      // 加载循环规则（如果store中没有，这会异步获取）
      await loadRecurrence()
    }
  }
})

watch(
  () => props.taskId,
  async (newTaskId) => {
    if (newTaskId) {
      // 🔥 先尝试同步加载循环规则（如果store中已有数据）
      const cardTask = taskStore.getTaskById_Mux(newTaskId)
      if (cardTask?.recurrence_id) {
        const recurrence = recurrenceStore.getRecurrenceById(cardTask.recurrence_id)
        if (recurrence) {
          currentRecurrence.value = recurrence
        }
      }

      const detail = (await taskStore.fetchTaskDetail_DMA(newTaskId)) as TaskDetail | null
      if (detail) {
        titleInput.value = detail.title
        glanceNote.value = detail.glance_note || ''
        detailNote.value = detail.detail_note || ''
        selectedAreaId.value = detail.area_id || null

        // 初始化截止日期
        if (detail.due_date) {
          // ✅ due_date.date 现在是 YYYY-MM-DD 格式，直接使用
          dueDateInput.value = detail.due_date.date
          dueDateType.value = detail.due_date.type
        } else {
          dueDateInput.value = ''
          dueDateType.value = 'SOFT'
        }

        // 等待 DOM 更新后调整 textarea 高度
        await nextTick()
        initTextareaHeights()

        // 加载循环规则（如果store中没有，这会异步获取）
        await loadRecurrence()
      }
    }
  }
)

// 处理主按钮状态变化（双模式）
async function handleMainCheckboxChange(newState: 'completed' | 'present' | null) {
  if (!props.taskId) return

  if (newState === 'completed') {
    // 标记为完成
    await pipeline.dispatch('task.complete', { id: props.taskId })
  } else if (newState === 'present') {
    // 标记在场（长按）
    if (!todayDate.value) return

    // 如果任务已完成，先重新打开
    if (task.value?.is_completed) {
      await pipeline.dispatch('task.reopen', { id: props.taskId })
    }

    // 更新 schedule outcome 为在场
    await pipeline.dispatch('schedule.update', {
      task_id: props.taskId,
      scheduled_day: todayDate.value,
      updates: { outcome: 'PRESENCE_LOGGED' },
    })
  } else {
    // newState === null，取消选中
    const currentState = mainCheckboxState.value

    if (currentState === 'completed') {
      // 从完成状态恢复：重新打开任务
      await pipeline.dispatch('task.reopen', { id: props.taskId })
    } else if (currentState === 'present') {
      // 从在场状态恢复：设置为仅计划
      if (!todayDate.value) return

      await pipeline.dispatch('schedule.update', {
        task_id: props.taskId,
        scheduled_day: todayDate.value,
        updates: { outcome: 'PLANNED' },
      })
    }
  }
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

  // ✅ 直接发送 YYYY-MM-DD 格式，符合后端 NaiveDate 类型
  await pipeline.dispatch('task.update', {
    id: props.taskId,
    updates: {
      due_date: dueDateInput.value, // YYYY-MM-DD format
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

  // 新子任务添加到最前面
  const updatedSubtasks = [newSubtask, ...subtasks.value]

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

  try {
    logger.info(LogTags.STORE_RECURRENCE, 'Stopping recurrence', {
      recurrenceId: currentRecurrence.value.id,
      instanceDate,
    })
    // ✅ stopRepeating 内部已包含 confirm 确认，无需重复弹窗
    await recurrenceOps.stopRepeating(currentRecurrence.value.id, instanceDate)

    await loadRecurrence()
  } catch (error) {
    console.error('Failed to stop repeating:', error)
    alert('操作失败，请重试')
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
      // ✅ 视图刷新由 CPU 指令的 commit 阶段统一处理
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
</script>

<template>
  <div
    class="modal-overlay"
    @mousedown.self="handleOverlayMouseDown"
    @click.self="handleOverlayClick"
  >
    <CuteCard class="editor-card" @mousedown="handleCardMouseDown" @click.stop>
      <div v-if="task">
        <!-- 卡片头部 -->
        <div class="card-header">
          <div class="header-left">
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

            <!-- 区域选择器下拉 -->
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

          <div class="header-right">
            <!-- 截止日期选择器 -->
            <div class="due-date-wrapper">
              <button class="due-date-button" @click="showDueDatePicker = !showDueDatePicker">
                <span v-if="task.due_date">
                  {{ task.due_date.date }}
                </span>
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

            <!-- 关闭按钮 -->
            <button class="close-button" @click="handleClose">×</button>
          </div>
        </div>

        <!-- 主内容区 -->
        <div class="card-body">
          <!-- 任务标题区域 -->
          <div class="section section-title">
            <div class="section-icon">
              <CuteDualModeCheckbox
                :state="mainCheckboxState"
                size="large"
                @update:state="handleMainCheckboxChange"
              />
            </div>
            <div class="section-body">
              <input
                v-model="titleInput"
                class="title-input"
                :class="{ completed: task.is_completed }"
                @blur="updateTitle"
                @keydown.enter="updateTitle"
              />
            </div>
          </div>

          <!-- 循环规则区域 -->
          <div v-if="currentRecurrence" class="section section-recurrence">
            <div class="section-icon">
              <CuteIcon name="RefreshCw" :size="20" />
            </div>
            <div class="section-body">
              <div class="recurrence-info">
                <span class="recurrence-text">{{ recurrenceDescription }}</span>
                <span v-if="currentRecurrence.end_date" class="recurrence-expiry">
                  直到 {{ currentRecurrence.end_date }}
                </span>
              </div>

              <div class="recurrence-actions">
                <span class="status-badge" :class="{ active: isRecurrenceActive }">
                  {{ isRecurrenceActive ? '激活' : '过期' }}
                </span>
                <div class="action-buttons">
                  <button
                    v-if="(task as any)?.recurrence_original_date && !currentRecurrence.end_date"
                    class="action-btn"
                    @click="handleStopRepeating"
                    title="停止重复"
                  >
                    <CuteIcon name="X" :size="16" />
                  </button>
                  <button
                    v-if="currentRecurrence.end_date"
                    class="action-btn"
                    @click="handleExtendRecurrence"
                    title="继续循环"
                  >
                    <CuteIcon name="Check" :size="16" />
                  </button>
                  <button
                    class="action-btn danger"
                    @click="handleDeleteRecurrence"
                    title="删除规则"
                  >
                    <CuteIcon name="Trash2" :size="16" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 任务描述区域 -->
          <div class="section section-note">
            <div class="section-icon">
              <CuteIcon name="FileText" :size="20" />
            </div>
            <div class="section-body">
              <div
                v-if="!glanceNote && !isTitleEditing"
                class="note-placeholder"
                @click="isTitleEditing = true"
              >
                任务描述...
              </div>
              <textarea
                ref="glanceNoteTextarea"
                v-model="glanceNote"
                class="note-textarea"
                placeholder="任务描述..."
                rows="1"
                @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
                @blur="updateGlanceNote"
              ></textarea>
            </div>
          </div>

          <!-- 子任务区域 -->
          <div class="section section-subtasks">
            <div class="section-header">
              <div class="section-icon">
                <CuteIcon name="List" :size="20" />
              </div>
              <span class="section-title-text">子任务</span>
            </div>
            <div class="section-body">
              <div class="subtasks-input">
                <input
                  v-model="newSubtaskTitle"
                  class="add-subtask-input"
                  placeholder="添加子任务..."
                  @keydown.enter="handleAddSubtask"
                />
              </div>
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
                    <button class="delete-button" @click="handleDeleteSubtask(subtask.id)">
                      ×
                    </button>
                  </div>
                </template>
              </draggable>
            </div>
          </div>

          <!-- 详细笔记区域 -->
          <div class="section section-note">
            <div class="section-icon">
              <CuteIcon name="FileText" :size="20" />
            </div>
            <div class="section-body">
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
        </div>

        <!-- 底栏 -->
        <div class="card-footer">
          <div class="footer-actions">
            <button class="footer-button confirm-footer-button" @click="handleClose">完成</button>
          </div>
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
/* ==================== 模态框基础 ==================== */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: var(--color-overlay-heavy);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.editor-card {
  width: 63rem;
  max-width: 90vw;
  max-height: 90vh;
  border: 1px solid var(--color-border-default);
  background-color: var(--color-card-available);
  border-radius: 0.8rem;
  overflow-y: auto;
  padding: 0; /* Override CuteCard's default 1.6rem padding */
}

/* ==================== 卡片头部 ==================== */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4.1rem 4.1rem 1.5rem; /* Top and horizontal +1.6rem, bottom unchanged */
  border-bottom: 1px solid var(--color-border-default);
}

.header-left {
  display: flex;
  align-items: center;
  position: relative;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 1rem;
}

/* 区域标签 */
.area-tag-wrapper {
  cursor: pointer;
  transition: opacity 0.2s;
}

.area-tag-wrapper:hover {
  opacity: 0.7;
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
  background-color: var(--color-background-hover);
}

.no-area-text {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
}

/* 截止日期按钮 */
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
  border-color: var(--color-button-primary-bg);
  color: var(--color-button-primary-bg);
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
  border-color: var(--color-button-primary-bg);
  color: var(--color-button-primary-bg);
}

.type-button.active {
  background-color: var(--color-button-primary-bg);
  color: white;
  border-color: var(--color-button-primary-bg);
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
  background-color: var(--color-button-primary-bg);
  color: white;
}

.save-button:hover {
  background-color: var(--color-primary-dark, #1565c0);
}

.clear-button {
  background-color: var(--color-danger);
  color: var(--color-text-on-accent);
}

.clear-button:hover {
  background-color: var(--c-red-500);
  filter: brightness(0.9);
}

.cancel-button {
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.cancel-button:hover {
  background-color: var(--color-background-hover);
}

/* 循环按钮 */
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
  border-color: var(--color-button-primary-bg);
  background-color: var(--color-button-primary-bg);
  color: white;
}

.recurrence-button.active {
  border-color: var(--color-button-primary-bg);
  color: white;
  background-color: var(--color-button-primary-bg);
}

.recurrence-button.active:hover {
  background-color: var(--color-button-primary-bg);
  color: white;
}

/* 关闭按钮 */
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

/* ==================== 主内容区 ==================== */
.card-body {
  padding: 0 4.1rem; /* Increased by 1.6rem to compensate for removed CuteCard padding */
}

/* ==================== 统一Section样式 ==================== */
.section {
  display: flex;
  align-items: center; /* 统一使用中线对齐 */
  gap: 1rem;
  padding: 1.7rem 0 0 0; /* 增加到 1.7rem */
}

/* 第一个section无特殊样式 */
.section:first-child {
  padding-top: 2.5rem; /* Increased for more breathing room */
}

.section-icon {
  flex-shrink: 0;
  width: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary);
}

.section-body {
  flex: 1;
  min-width: 0;
}

/* ==================== 任务标题区域 ==================== */
.title-input {
  width: 100%;
  font-size: 2rem;
  font-weight: 600;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
  padding: 0;
  border-bottom: 2px solid transparent;
  transition: border-color 0.2s;
}

.title-input:focus {
  border-bottom-color: var(--color-border-default);
}

.title-input.completed {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

/* ==================== 循环规则区域 ==================== */
.section-recurrence .section-body {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.recurrence-info {
  display: flex;
  flex-direction: row; /* 改为横向排列 */
  flex-wrap: wrap; /* 允许换行 */
  align-items: baseline; /* 底部基线对齐 */
  gap: 0.8rem; /* 增大间距 */
  overflow: hidden;
  flex: 1;
}

.recurrence-text {
  font-size: 1.6rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.recurrence-expiry {
  font-size: 1.3rem; /* 稍小 */
  font-weight: 400; /* 正常字重 */
  color: var(--color-text-secondary); /* 次要文字颜色 */
}

.recurrence-actions {
  display: flex;
  align-items: center;
  gap: 1.2rem;
  flex-shrink: 0;
}

.status-badge {
  font-size: 1.2rem;
  font-weight: 600;
  padding: 0.4rem 0.8rem;
  border-radius: 0.4rem;
  height: 2.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  /* 默认过期状态样式 */
  color: var(--color-info-text);
  background-color: var(--color-info-light);
}

.status-badge.active {
  color: var(--color-success-text);
  background-color: var(--color-success-light);
}

.action-buttons {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 0.4rem;
  background: white;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 1px 2px rgb(0 0 0 / 5%);
}

.action-btn:hover {
  border-color: var(--color-button-primary-bg);
  color: var(--color-button-primary-bg);
  background: white;
}

.action-btn.danger:hover {
  border-color: var(--color-danger);
  color: var(--color-danger);
}

/* ==================== 笔记区域 ==================== */
.section-note {
  border-bottom: 1px solid var(--color-border-default);
  align-items: flex-start; /* 笔记区域使用顶部对齐，因为是多行内容 */
  padding-top: 0.7rem; /* 1.7rem - 1rem，补偿图标向下移动 */
}

.section-note .section-icon {
  margin-top: 1rem; /* 对齐 textarea 的第一行文本：padding-top (1rem) */
}

.note-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  padding: 1rem 0;
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
  padding: 1rem 0;
  border-radius: 0.4rem;
  overflow: hidden;
  min-height: 2rem;
}

.note-textarea:hover,
.note-textarea:focus {
  background: transparent;
}

.note-textarea::placeholder {
  color: transparent;
}

.section-note .section-body {
  position: relative;
  min-height: 10rem;
}

/* ==================== 子任务区域 ==================== */
.section-subtasks {
  flex-direction: column;
  align-items: stretch;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  margin-bottom: 1rem;
}

.section-title-text {
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.subtasks-input {
  padding: 0.5rem 0;
}

.add-subtask-input {
  width: 100%;
  padding: 0.2rem 0;
  font-size: 1.5rem;
  border: none;
  background-color: transparent;
  color: var(--color-text-primary);
  outline: none;
  transition: all 0.2s;
}

.add-subtask-input::placeholder {
  color: var(--color-text-tertiary);
}

.subtasks-list {
  display: flex;
  flex-direction: column;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.5rem 0;
  border-radius: 0.4rem;
  transition: background-color 0.2s;
  cursor: move;
  position: relative;
}

.subtask-item:hover {
  background-color: var(--color-background-hover, #f5f5f5);
}

.drag-handle {
  position: absolute;
  left: -2.8rem;
  top: 0;
  bottom: 0;
  margin: auto 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 2.8rem;
  cursor: grab;
  color: var(--color-text-tertiary);
  font-size: 1.6rem;
  line-height: 1;
  user-select: none;
  opacity: 0;
  transition:
    opacity 0.2s ease,
    color 0.2s ease,
    transform 0.2s ease;
  border-radius: 0.4rem;
}

.drag-handle:hover {
  color: var(--color-text-secondary);
  background-color: var(--color-background-hover, #f5f5f5);
}

.drag-handle:active {
  cursor: grabbing;
  color: var(--color-text-primary);
  transform: scale(0.95);
}

.subtask-item:hover .drag-handle {
  opacity: 1;
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
  color: var(--color-danger);
}

.subtask-item:hover .delete-button {
  opacity: 1;
}

/* ==================== 底栏 ==================== */
.card-footer {
  padding: 1.5rem 4.1rem 3.1rem; /* Top unchanged, horizontal +1.6rem, bottom +1.6rem */
  display: flex;
  justify-content: flex-end;
}

.footer-actions {
  display: flex;
  gap: 1rem;
}

.footer-button {
  padding: 0.8rem 1.6rem;
  border-radius: 0.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  min-width: 8rem;
}

.cancel-footer-button {
  background-color: transparent;
  color: var(--color-text-primary);
  border: 1px solid var(--color-border-default);
}

.cancel-footer-button:hover {
  background-color: var(--color-background-hover, #f5f5f5);
  border-color: var(--color-text-secondary);
}

.confirm-footer-button {
  background-color: var(--color-button-primary-bg);
  color: white;
  border: 1px solid var(--color-button-primary-bg);
}

.confirm-footer-button:hover {
  background-color: var(--color-primary-dark, #1565c0);
}
</style>

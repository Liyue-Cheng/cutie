<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="modal-overlay" @click.self="handleClose">
        <div class="modal-container">
          <!-- 标题栏 -->
          <div class="modal-header">
            <h2 class="modal-title">
              <CuteIcon name="RefreshCw" :size="20" />
              <span>{{ $t('recurrence.title.manager') }}</span>
            </h2>
            <button class="close-btn" @click="handleClose">
              <CuteIcon name="X" :size="20" />
            </button>
          </div>

          <!-- 内容区 -->
          <div class="modal-content">
            <!-- 加载状态 -->
            <div v-if="isLoading" class="loading-state">
              <CuteIcon name="Loader" :size="32" class="spinner" />
              <p>{{ $t('common.state.loading') }}</p>
            </div>

            <!-- 空状态 -->
            <div v-else-if="recurrences.length === 0" class="empty-state">
              <CuteIcon name="CalendarX" :size="48" />
              <p>{{ $t('recurrence.empty.title') }}</p>
              <p class="empty-hint">{{ $t('recurrence.empty.hint') }}</p>
            </div>

            <!-- 循环规则列表 -->
            <div v-else class="recurrence-list">
              <div
                v-for="recurrence in recurrences"
                :key="recurrence.id"
                class="recurrence-item"
                :class="{ inactive: !recurrence.is_active }"
              >
                <!-- 左侧：规则信息 -->
                <div class="recurrence-info">
                  <div class="recurrence-header">
                    <span class="recurrence-title">{{
                      getTemplateTitle(recurrence.template_id)
                    }}</span>
                    <span v-if="!recurrence.is_active" class="inactive-badge">{{ $t('recurrence.status.inactive') }}</span>
                  </div>
                  <div class="recurrence-details">
                    <span class="detail-item">
                      <CuteIcon name="RefreshCw" :size="12" />
                      {{ formatRule(recurrence.rule) }}
                    </span>
                    <span v-if="recurrence.start_date" class="detail-item">
                      <CuteIcon name="CalendarDays" :size="12" />
                      {{ $t('recurrence.label.start') }}: {{ recurrence.start_date }}
                    </span>
                    <span v-if="recurrence.end_date" class="detail-item">
                      <CuteIcon name="CalendarX" :size="12" />
                      {{ $t('recurrence.label.end') }}: {{ recurrence.end_date }}
                    </span>
                    <span class="detail-item">
                      <CuteIcon name="Clock" :size="12" />
                      {{ formatTimeType(recurrence.time_type) }}
                    </span>
                    <span class="detail-item">
                      <CuteIcon name="Archive" :size="12" />
                      {{ formatExpiryBehavior(recurrence.expiry_behavior) }}
                    </span>
                  </div>
                </div>

                <!-- 右侧：操作按钮 -->
                <div class="recurrence-actions">
                  <!-- 只有已开始的循环规则才显示暂停/启用按钮 -->
                  <button
                    v-if="!isFutureRecurrence(recurrence)"
                    class="action-btn toggle-btn"
                    :title="recurrence.is_active ? $t('recurrence.action.pause') : $t('recurrence.action.resume')"
                    @click="toggleActive(recurrence)"
                  >
                    <CuteIcon :name="recurrence.is_active ? 'Pause' : 'Play'" :size="16" />
                  </button>
                  <button
                    class="action-btn edit-btn"
                    :title="$t('common.action.edit')"
                    @click="editRecurrence(recurrence)"
                  >
                    <CuteIcon name="Pencil" :size="16" />
                  </button>
                  <button
                    class="action-btn delete-btn"
                    :title="$t('common.action.delete')"
                    @click="deleteRecurrence(recurrence)"
                  >
                    <CuteIcon name="Trash2" :size="16" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部操作栏 -->
          <div class="modal-footer">
            <button class="footer-btn secondary-btn" @click="handleClose">{{ $t('common.action.close') }}</button>
            <button class="footer-btn primary-btn" @click="refreshRecurrences">
              <CuteIcon name="RefreshCw" :size="16" />
              {{ $t('common.action.refresh') }}
            </button>
          </div>
        </div>

        <!-- 编辑对话框 -->
        <RecurrenceEditDialog
          v-if="editingRecurrence"
          :recurrence="editingRecurrence"
          @close="editingRecurrence = null"
          @save="handleSaveEdit"
        />
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import RecurrenceEditDialog from './RecurrenceEditDialog.vue'
import { useRecurrenceStore } from '@/stores/recurrence'
import { useTemplateStore } from '@/stores/template'
import { useTaskStore } from '@/stores/task'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import type { TaskRecurrence } from '@/types/dtos'
import { dialog } from '@/composables/useDialog'

interface Props {
  show: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()
const recurrenceStore = useRecurrenceStore()
const templateStore = useTemplateStore()
const taskStore = useTaskStore()
const isLoading = ref(false)
const editingRecurrence = ref<TaskRecurrence | null>(null)

// 获取所有循环规则（按激活状态排序）
const recurrences = computed(() => {
  const all = recurrenceStore.allRecurrences
  return [...all].sort((a, b) => {
    // 激活的排在前面
    if (a.is_active && !b.is_active) return -1
    if (!a.is_active && b.is_active) return 1
    // 同样状态按创建时间倒序
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  })
})

// 监听 show 变化，打开时加载数据
watch(
  () => props.show,
  async (newShow) => {
    if (newShow) {
      await loadRecurrences()
    }
  },
  { immediate: true }
)

// 加载循环规则和模板
async function loadRecurrences() {
  isLoading.value = true
  try {
    // 并行加载循环规则和模板
    await Promise.all([
      pipeline.dispatch('recurrence.fetch_all', {}),
      templateStore.fetchAllTemplates(),
    ])
    logger.info(LogTags.COMPONENT_RECURRENCE_MANAGER, 'Loaded recurrences and templates', {
      recurrenceCount: recurrences.value.length,
      templateCount: templateStore.allTemplates.length,
    })
  } catch (error) {
    logger.error(LogTags.COMPONENT_RECURRENCE_MANAGER, 'Failed to load recurrences', error as Error)
  } finally {
    isLoading.value = false
  }
}

// 刷新循环规则
async function refreshRecurrences() {
  await loadRecurrences()
}

// 获取模板标题
function getTemplateTitle(templateId: string): string {
  const template = templateStore.getTemplateById(templateId)
  return template ? template.title : t('recurrence.unknownTask')
}

// 判断是否是未来的循环规则（开始日期在今天之后）
function isFutureRecurrence(recurrence: TaskRecurrence): boolean {
  if (!recurrence.start_date) return false
  const today = getTodayDateString()
  return recurrence.start_date > today
}

// 格式化循环规则
function formatRule(rule: string): string {
  if (rule.includes('FREQ=DAILY')) {
    return t('recurrence.description.daily')
  } else if (rule.includes('FREQ=WEEKLY')) {
    const match = rule.match(/BYDAY=([A-Z,]+)/)
    if (match && match[1]) {
      const days = match[1].split(',').map((d) => {
        const dayMap: Record<string, string> = {
          MO: t('recurrence.weekday.mon'),
          TU: t('recurrence.weekday.tue'),
          WE: t('recurrence.weekday.wed'),
          TH: t('recurrence.weekday.thu'),
          FR: t('recurrence.weekday.fri'),
          SA: t('recurrence.weekday.sat'),
          SU: t('recurrence.weekday.sun'),
        }
        return dayMap[d] || d
      })
      return `${t('recurrence.description.weekly')} (${days.join(', ')})`
    }
    return t('recurrence.description.weekly')
  } else if (rule.includes('FREQ=MONTHLY')) {
    return t('recurrence.description.monthly')
  } else if (rule.includes('FREQ=YEARLY')) {
    return t('recurrence.description.yearly')
  }
  return rule
}

// 格式化时间类型
function formatTimeType(timeType: string): string {
  const typeMap: Record<string, string> = {
    FLOATING: t('recurrence.timeType.floating'),
    FIXED: t('recurrence.timeType.fixed'),
  }
  return typeMap[timeType] || timeType
}

// 格式化过期行为
function formatExpiryBehavior(behavior: string): string {
  const behaviorMap: Record<string, string> = {
    CARRYOVER_TO_STAGING: t('recurrence.expiry.carryover'),
    EXPIRE: t('recurrence.expiry.expire'),
  }
  return behaviorMap[behavior] || behavior
}

// 切换激活状态
async function toggleActive(recurrence: TaskRecurrence) {
  const willBeActive = !recurrence.is_active

  // 如果是暂停（设置为 false），需要确认并删除今天之后的未完成任务
  if (!willBeActive) {
    const confirmed = await dialog.confirm(t('confirm.pauseRecurrence'))
    if (!confirmed) return
  }

  try {
    const today = getTodayDateString()

    // 更新激活状态和结束日期
    await pipeline.dispatch('recurrence.update', {
      id: recurrence.id,
      is_active: willBeActive,
      // 暂停时设置结束日期为今天，启用时清除结束日期
      end_date: willBeActive ? null : today,
    })

    // 如果是暂停，删除今天之后的未完成任务
    if (!willBeActive) {
      await cleanupFutureTasks(recurrence.id)
    }

    logger.info(LogTags.COMPONENT_RECURRENCE_MANAGER, 'Toggled recurrence active state', {
      id: recurrence.id,
      is_active: willBeActive,
      end_date: willBeActive ? null : today,
    })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_RECURRENCE_MANAGER,
      'Failed to toggle recurrence',
      error as Error
    )
  }
}

// 清理今天之后的未完成任务
async function cleanupFutureTasks(recurrenceId: string) {
  const today = getTodayDateString()
  const allTasks = taskStore.allTasks

  // 找出所有属于该循环规则且在今天之后的未完成任务
  const taskIdsToRemove = allTasks
    .filter((task: any) => {
      if (task.recurrence_id !== recurrenceId) return false
      if (task.is_completed) return false

      // 检查任务的原始日期是否在今天之后
      const originalDate = task.recurrence_original_date
      if (!originalDate) return false

      return originalDate > today
    })
    .map((task: any) => task.id)

  if (taskIdsToRemove.length > 0) {
    // 从本地 store 中移除
    taskStore.batchRemoveTasks_mut(taskIdsToRemove)

    logger.info(LogTags.COMPONENT_RECURRENCE_MANAGER, 'Removed future recurrence tasks', {
      recurrenceId,
      count: taskIdsToRemove.length,
    })

    // 重新获取任务数据以保持同步
    try {
      await taskStore.fetchAllIncompleteTasks_DMA()
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_RECURRENCE_MANAGER,
        'Failed to refetch tasks after cleanup',
        error as Error
      )
    }
  }
}

// 编辑循环规则
function editRecurrence(recurrence: TaskRecurrence) {
  editingRecurrence.value = { ...recurrence }
}

// 保存编辑
async function handleSaveEdit(updates: Partial<TaskRecurrence>) {
  if (!editingRecurrence.value) return

  try {
    await pipeline.dispatch('recurrence.update', {
      id: editingRecurrence.value.id,
      ...updates,
    })
    logger.info(LogTags.COMPONENT_RECURRENCE_MANAGER, 'Updated recurrence', {
      id: editingRecurrence.value.id,
      updates,
    })
    editingRecurrence.value = null
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_RECURRENCE_MANAGER,
      'Failed to update recurrence',
      error as Error
    )
  }
}

// 删除循环规则
async function deleteRecurrence(recurrence: TaskRecurrence) {
  const confirmed = await dialog.confirm(
    t('confirm.deleteRecurrence', { rule: formatRule(recurrence.rule) })
  )
  if (!confirmed) return

  try {
    // 删除循环规则
    await pipeline.dispatch('recurrence.delete', { id: recurrence.id })

    // 清理本地未完成任务（从今天开始的所有未完成任务）
    const today = getTodayDateString()
    await cleanupRecurrenceTasks(recurrence.id, today)

    logger.info(LogTags.COMPONENT_RECURRENCE_MANAGER, 'Deleted recurrence', {
      id: recurrence.id,
    })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_RECURRENCE_MANAGER,
      'Failed to delete recurrence',
      error as Error
    )
  }
}

// 清理循环规则的所有未完成任务（从指定日期开始）
async function cleanupRecurrenceTasks(recurrenceId: string, fromDate: string) {
  const allTasks = taskStore.allTasks

  // 找出所有属于该循环规则且从指定日期开始的未完成任务
  const taskIdsToRemove = allTasks
    .filter((task: any) => {
      if (task.recurrence_id !== recurrenceId) return false
      if (task.is_completed) return false

      // 检查任务的原始日期
      const originalDate = task.recurrence_original_date
      if (!originalDate) return true // 没有原始日期的也删除

      return originalDate >= fromDate
    })
    .map((task: any) => task.id)

  if (taskIdsToRemove.length > 0) {
    // 从本地 store 中移除
    taskStore.batchRemoveTasks_mut(taskIdsToRemove)

    logger.info(LogTags.COMPONENT_RECURRENCE_MANAGER, 'Removed recurrence tasks', {
      recurrenceId,
      count: taskIdsToRemove.length,
      fromDate,
    })

    // 重新获取任务数据以保持同步
    try {
      await taskStore.fetchAllIncompleteTasks_DMA()
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_RECURRENCE_MANAGER,
        'Failed to refetch tasks after cleanup',
        error as Error
      )
    }
  }
}

// 关闭模态框
function handleClose() {
  emit('close')
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay-heavy, #f0f);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 2rem;
}

.modal-container {
  width: 100%;
  max-width: 80rem;
  height: 70vh;
  background-color: var(--color-background-content, #f0f);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-xl, #f0f);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 标题栏 */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2rem 2.4rem;
  border-bottom: 1px solid var(--color-border-default, #f0f);
  flex-shrink: 0;
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 1.2rem;
  font-size: 2rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.close-btn {
  all: unset;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  border-radius: 0.6rem;
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.2s ease;
}

.close-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary);
}

/* 内容区 */
.modal-content {
  flex: 1;
  overflow-y: auto;
  padding: 2.4rem;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 1.6rem;
  color: var(--color-text-secondary);
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 1.2rem;
  color: var(--color-text-secondary);
}

.empty-state p {
  margin: 0;
  font-size: 1.6rem;
}

.empty-hint {
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
}

/* 循环规则列表 */
.recurrence-list {
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

.recurrence-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.6rem 2rem;
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  transition: all 0.2s ease;
}

.recurrence-item:hover {
  border-color: var(--color-border-hover, #f0f);
  box-shadow: var(--shadow-sm, #f0f);
}

.recurrence-item.inactive {
  opacity: 0.6;
}

/* 规则信息 */
.recurrence-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.recurrence-header {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.recurrence-title {
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.inactive-badge {
  padding: 0.2rem 0.8rem;
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-tertiary);
  background-color: var(--color-background-hover, #f0f);
  border-radius: 0.4rem;
}

.recurrence-details {
  display: flex;
  align-items: center;
  gap: 1.2rem;
  flex-wrap: wrap;
}

.detail-item {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 1.3rem;
  color: var(--color-text-secondary);
}

/* 操作按钮 */
.recurrence-actions {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.action-btn {
  all: unset;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.toggle-btn {
  color: var(--color-text-accent, #f0f);
}

.toggle-btn:hover {
  background-color: var(--color-background-accent-light, #f0f);
}

.edit-btn {
  color: var(--color-text-secondary);
}

.edit-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary);
}

.delete-btn {
  color: var(--color-danger, #f0f);
}

.delete-btn:hover {
  background-color: var(--color-danger-light, #f0f);
}

/* 底部操作栏 */
.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 1.2rem;
  padding: 1.6rem 2.4rem;
  border-top: 1px solid var(--color-border-default, #f0f);
  flex-shrink: 0;
}

.footer-btn {
  all: unset;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.8rem;
  padding: 0.8rem 1.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.secondary-btn {
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
}

.secondary-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover, #f0f);
}

.primary-btn {
  color: var(--color-button-primary-text, #f0f);
  background-color: var(--color-button-primary-bg, #f0f);
}

.primary-btn:hover {
  background-color: var(--color-button-primary-hover, #f0f);
}

/* 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform 0.3s ease;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.95);
}
</style>

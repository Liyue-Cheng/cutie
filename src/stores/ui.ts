import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * UI Store - 管理全局 UI 状态
 *
 * 职责：
 * - 任务编辑器的打开/关闭状态
 * - 循环规则编辑对话框的打开/关闭状态
 * - 传递必要的上下文信息（taskId, viewKey, recurrenceId）
 * - 解耦组件之间的事件传递
 */

export const useUIStore = defineStore('ui', () => {
  // ==================== 任务编辑器状态 ====================

  /**
   * 当前正在编辑的任务 ID
   * null 表示编辑器关闭
   */
  const editorTaskId = ref<string | null>(null)

  /**
   * 编辑器的视图上下文 key
   * 用于循环任务创建时确定 start_date
   */
  const editorViewKey = ref<string | null>(null)

  /**
   * 编辑器是否打开
   */
  const isEditorOpen = computed(() => editorTaskId.value !== null)

  // ==================== 循环规则编辑对话框状态 ====================

  /**
   * 当前正在编辑的循环规则 ID
   * null 表示对话框关闭
   */
  const recurrenceEditDialogId = ref<string | null>(null)

  /**
   * 循环规则编辑对话框是否打开
   */
  const isRecurrenceEditDialogOpen = computed(() => recurrenceEditDialogId.value !== null)

  // ==================== 时间块创建对话框状态 ====================

  /**
   * 时间块创建对话框上下文
   * 存储选择的时间信息以及锚点位置（用于在日历左侧浮出面板）
   */
  interface TimeBlockCreateContext {
    // 时间信息
    startISO: string
    endISO: string
    startTimeLocal?: string
    endTimeLocal?: string
    isAllDay: boolean

    // UI 锚点（视口坐标）
    anchorTop?: number
    anchorLeft?: number
  }

  const timeBlockCreateContext = ref<TimeBlockCreateContext | null>(null)

  /**
   * 时间块创建对话框是否打开
   */
  const isTimeBlockCreateDialogOpen = computed(() => timeBlockCreateContext.value !== null)

  // ==================== 操作方法 ====================

  /**
   * 打开任务编辑器
   * @param taskId 任务 ID
   * @param viewKey 视图上下文 key (如 'daily::2025-10-10', 'misc::staging')
   */
  function openEditor(taskId: string, viewKey?: string) {
    editorTaskId.value = taskId
    editorViewKey.value = viewKey ?? null

    logger.info(LogTags.STORE_UI, 'Opening task editor', {
      taskId,
      viewKey: viewKey ?? 'none',
    })
  }

  /**
   * 关闭任务编辑器
   */
  function closeEditor() {
    const previousTaskId = editorTaskId.value

    editorTaskId.value = null
    editorViewKey.value = null

    logger.info(LogTags.STORE_UI, 'Closing task editor', {
      previousTaskId,
    })
  }

  /**
   * 打开循环规则编辑对话框
   * @param recurrenceId 循环规则 ID
   */
  function openRecurrenceEditDialog(recurrenceId: string) {
    recurrenceEditDialogId.value = recurrenceId

    logger.info(LogTags.STORE_UI, 'Opening recurrence edit dialog', {
      recurrenceId,
    })
  }

  /**
   * 关闭循环规则编辑对话框
   */
  function closeRecurrenceEditDialog() {
    const previousRecurrenceId = recurrenceEditDialogId.value

    recurrenceEditDialogId.value = null

    logger.info(LogTags.STORE_UI, 'Closing recurrence edit dialog', {
      previousRecurrenceId,
    })
  }

  /**
   * 打开时间块创建对话框
   * @param context 时间选择上下文
   */
  function openTimeBlockCreateDialog(context: TimeBlockCreateContext) {
    timeBlockCreateContext.value = context

    logger.info(LogTags.STORE_UI, 'Opening time block create dialog', {
      startISO: context.startISO,
      endISO: context.endISO,
      isAllDay: context.isAllDay,
    })
  }

  /**
   * 关闭时间块创建对话框
   */
  function closeTimeBlockCreateDialog() {
    timeBlockCreateContext.value = null

    logger.info(LogTags.STORE_UI, 'Closing time block create dialog')
  }

  return {
    // 任务编辑器状态
    editorTaskId,
    editorViewKey,
    isEditorOpen,

    // 循环规则编辑对话框状态
    recurrenceEditDialogId,
    isRecurrenceEditDialogOpen,

    // 时间块创建对话框状态
    timeBlockCreateContext,
    isTimeBlockCreateDialogOpen,

    // 操作方法
    openEditor,
    closeEditor,
    openRecurrenceEditDialog,
    closeRecurrenceEditDialog,
    openTimeBlockCreateDialog,
    closeTimeBlockCreateDialog,
  }
})

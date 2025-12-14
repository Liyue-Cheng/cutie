<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Template } from '@/types/dtos'
import { useAreaStore } from '@/stores/area'
import { useContextMenu } from '@/composables/useContextMenu'
import { logger, LogTags } from '@/infra/logging/logger'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import TimeDurationPicker from '@/components/parts/TimeDurationPicker.vue'
import TemplateCardMenu from '@/components/assembles/ContextMenu/TemplateCardMenu.vue'
import { pipeline } from '@/cpu'

const props = defineProps<{
  template: Template
}>()

const areaStore = useAreaStore()
const contextMenu = useContextMenu()
const emit = defineEmits<{
  openEditor: []
}>()

// ✅ 时间选择器状态
const showTimePicker = ref(false)

// 使用模板的subtasks_template字段
const subtasks = computed(() => props.template.subtasks_template || [])

// ✅ 通过 area_id 从 store 获取完整 area 信息
const area = computed(() => {
  return props.template.area_id ? areaStore.getAreaById(props.template.area_id) : null
})

// ✅ 格式化时间显示（显示预期时间模板）
const formattedDuration = computed(() => {
  const duration = props.template.estimated_duration_template

  if (duration === null || duration === 0) {
    return 'tiny'
  }

  const minutes = duration
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60

  // ✅ 统一格式为 x:xx
  return `${hours}:${mins.toString().padStart(2, '0')}`
})

// ✅ 切换时间选择器显示
function toggleTimePicker(event: Event) {
  event.stopPropagation()
  showTimePicker.value = !showTimePicker.value
}

// ✅ 更新预期时间模板
async function updateEstimatedDuration(duration: number | null) {
  try {
    await pipeline.dispatch('template.update', {
      id: props.template.id,
      estimated_duration_template: duration ?? undefined,
    })
    showTimePicker.value = false
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN_COLUMN,
      'Error updating estimated duration template',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

// ✅ 子任务状态变更（模板的子任务模板）
async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  // 更新subtask模板状态
  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  // 更新模板的subtasks_template
  await pipeline.dispatch('template.update', {
    id: props.template.id,
    subtasks_template: updatedSubtasks,
  })
}

// ✅ 显示右键菜单
function showContextMenu(event: MouseEvent) {
  contextMenu.show(
    TemplateCardMenu,
    {
      template: props.template,
      onOpenEditor: () => emit('openEditor'),
    },
    event
  )
}
</script>

<template>
  <CuteCard class="task-card" @click="emit('openEditor')" @contextmenu="showContextMenu">
    <div class="main-content">
      <!-- 第一行：标题 + 预期时间 -->
      <div class="card-header">
        <span class="title">{{ template.title }}</span>

        <!-- 预期时间显示 -->
        <div class="estimated-duration-wrapper">
          <button class="estimated-duration" @click="toggleTimePicker">
            {{ formattedDuration }}
          </button>

          <!-- 时间选择器弹窗 -->
          <div v-if="showTimePicker" class="time-picker-popup">
            <TimeDurationPicker
              :model-value="template.estimated_duration_template"
              @update:model-value="updateEstimatedDuration"
              @close="showTimePicker = false"
            />
          </div>
        </div>
      </div>

      <div v-if="template.glance_note_template" class="notes-section">
        <CuteIcon name="CornerDownRight" :size="14" />
        <span class="note-text">{{ template.glance_note_template }}</span>
      </div>

      <div v-if="subtasks.length > 0" class="subtasks-section">
        <div v-for="subtask in subtasks" :key="subtask.id" class="subtask-item">
          <CuteCheckbox
            :checked="subtask.is_completed"
            size="small"
            @update:checked="
              (isChecked: boolean) => handleSubtaskStatusChange(subtask.id, isChecked)
            "
            @click.stop
          />
          <span class="subtask-title">{{ subtask.title }}</span>
        </div>
      </div>

      <!-- 第二行：复选框（仅展示，无功能） + Area标签 -->
      <div class="card-footer">
        <div class="main-checkbox-wrapper">
          <!-- 完成按钮：仅展示，无点击功能 -->
          <CuteCheckbox
            class="main-checkbox always-visible"
            :checked="false"
            size="large"
            :disabled="true"
            @click.stop
          ></CuteCheckbox>
        </div>

        <AreaTag v-if="area" :name="area.name" :color="area.color" size="normal" />
      </div>
    </div>
  </CuteCard>
</template>

<style scoped>
.task-card {
  display: flex;
  flex-direction: column;
  padding: 1rem;
  margin-bottom: 0.75rem;
  border: 1px solid var(--color-border-default);
  background-color: var(--color-card-available);
  border-radius: 0.4rem;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
  cursor: pointer;
}

.task-card:hover {
  border-color: var(--color-border-hover);
  box-shadow: var(--shadow-md, #f0f);
}

.main-content {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

/* 第一行：标题 + 预期时间 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 1rem;
}

.title {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
}

.notes-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-text-primary);
}

.note-text {
  font-size: 1.3rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.subtask-title {
  font-size: 1.4rem;
  color: var(--color-text-primary);
}

/* 第二行：复选框 + Area标签 */
.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  margin-top: 0.5rem;
}

.estimated-duration-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.estimated-duration {
  padding: 0.4rem 0.8rem;
  background-color: var(--color-bg-secondary, #f0f);
  border: none;
  border-radius: 0.4rem;
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.estimated-duration:hover {
  background-color: var(--color-bg-hover, #f0f);
  color: var(--color-text-primary);
}

.time-picker-popup {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 0.4rem;
  z-index: 1000;
}

.main-checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  align-self: flex-start;
}

/* 始终显示的按钮 */
.always-visible {
  opacity: 1;
}

/* Area 标签位置调整 */
.card-footer :deep(.area-tag) {
  flex-shrink: 0;
}

/* 子任务选中时，只划子任务的线 */
/* stylelint-disable-next-line selector-class-pattern */
.subtask-item:has(.n-checkbox--checked) .subtask-title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}
</style>

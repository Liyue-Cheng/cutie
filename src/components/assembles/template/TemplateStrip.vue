<template>
  <div class="template-strip" @click="handleClick" @contextmenu="showContextMenu">
    <!-- 顶部：标题 + 预期时间 -->
    <div class="template-header">
      <div class="template-title">
        {{ template.title || '新模板' }}
      </div>

      <!-- 所属 Area 标签 -->
      <AreaTag
        v-if="area"
        class="area-tag-inline"
        :name="area.name"
        :color="area.color"
        size="normal"
      />

      <!-- 预期时间显示 -->
      <div class="estimated-duration-wrapper">
        <button class="estimated-duration" @click.stop="toggleTimePicker">
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

    <!-- 概览笔记 -->
    <div v-if="template.glance_note_template" class="template-note">
      <span class="icon-wrapper">
        <CuteIcon name="FileText" size="1.4rem" />
      </span>
      <span class="note-text">{{ template.glance_note_template }}</span>
    </div>

    <!-- 子任务显示区 -->
    <div v-if="subtasks.length > 0" class="subtasks-section">
      <div v-for="subtask in subtasks" :key="subtask.id" class="subtask-item">
        <CuteCheckbox
          :checked="subtask.is_completed"
          size="1.4rem"
          @update:checked="() => handleSubtaskStatusChange(subtask.id, !subtask.is_completed)"
          @click.stop
        />
        <span class="subtask-title" :class="{ completed: subtask.is_completed }">
          {{ subtask.title }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Template } from '@/types/dtos'
import { useAreaStore } from '@/stores/area'
import { useContextMenu } from '@/composables/useContextMenu'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import TimeDurationPicker from '@/components/parts/TimeDurationPicker.vue'
import TemplateCardMenu from '@/components/assembles/ContextMenu/TemplateCardMenu.vue'

// Props
interface Props {
  template: Template
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  openEditor: []
}>()

// Stores
const areaStore = useAreaStore()
const contextMenu = useContextMenu()

// 时间选择器状态
const showTimePicker = ref(false)

// 使用模板的subtasks_template字段
const subtasks = computed(() => props.template.subtasks_template || [])

// 通过 area_id 从 store 获取完整 area 信息
const area = computed(() => {
  return props.template.area_id ? areaStore.getAreaById(props.template.area_id) : null
})

// 格式化时间显示（显示预期时间模板）
const formattedDuration = computed(() => {
  const duration = props.template.estimated_duration_template

  if (duration === null || duration === 0) {
    return 'tiny'
  }

  const minutes = duration
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60

  return `${hours}:${mins.toString().padStart(2, '0')}`
})

// 切换时间选择器显示
function toggleTimePicker(event: Event) {
  event.stopPropagation()
  showTimePicker.value = !showTimePicker.value
}

// 更新预期时间模板
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

// 子任务状态变更（模板的子任务模板）
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

// 点击打开编辑器
function handleClick() {
  emit('openEditor')
}

// 显示右键菜单
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

<style scoped>
.template-strip {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  padding: 1.2rem;
  background-color: var(--color-card-available);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.template-strip:hover {
  background-color: var(--color-background-hover);
  border-color: var(--color-border-hover);
  box-shadow: 0 2px 8px rgb(0 0 0 / 8%);
}

/* 顶部：标题 + 预期时间 */
.template-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.8rem;
  min-height: 2.4rem;
}

.template-title {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  overflow-wrap: break-word;
}

/* Area 标签 */
.area-tag-inline {
  flex-shrink: 0;
}

/* 预期时间 */
.estimated-duration-wrapper {
  position: relative;
  flex-shrink: 0;
}

.estimated-duration {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 4.5rem;
  height: 2.4rem;
  padding: 0 0.8rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary);
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.estimated-duration:hover {
  background-color: var(--color-background-hover);
  border-color: var(--color-border-hover);
}

.time-picker-popup {
  position: absolute;
  top: calc(100% + 0.4rem);
  right: 0;
  z-index: 1000;
}

/* 概览笔记 */
.template-note {
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
  font-size: 1.3rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 0.2rem;
  color: var(--color-text-tertiary);
}

.note-text {
  flex: 1;
  overflow-wrap: break-word;
}

/* 子任务 */
.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  padding-top: 0.4rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.4rem 0;
}

.subtask-title {
  flex: 1;
  font-size: 1.3rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
  overflow-wrap: break-word;
  transition: all 0.2s ease;
}

.subtask-title.completed {
  color: var(--color-text-tertiary);
  text-decoration: line-through;
  opacity: 0.6;
}
</style>

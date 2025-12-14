<script setup lang="ts">
/**
 * CellItemTask - 时间线单元格任务项
 *
 * 用于在 TimelineDayCell 中显示单个任务
 * 支持 CuteDualModeCheckbox 进行状态切换
 */
import { computed } from 'vue'
import type { TaskCard } from '@/types/dtos'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteDualModeCheckbox from '@/components/parts/CuteDualModeCheckbox.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import { useAreaStore } from '@/stores/area'

// Checkbox状态类型
type CheckboxState = null | 'completed' | 'present'

interface Props {
  task: TaskCard
  scheduleDay: string // YYYY-MM-DD，用于计算 presence 状态
}

const props = defineProps<Props>()

const areaStore = useAreaStore()

const emit = defineEmits<{
  click: []
  contextmenu: [event: MouseEvent]
  'checkbox-change': [newState: CheckboxState]
}>()

const area = computed(() => {
  return props.task.area_id ? areaStore.getAreaById(props.task.area_id) : null
})

// 计算任务的checkbox状态
const checkboxState = computed<CheckboxState>(() => {
  if (props.task.is_completed) {
    return 'completed'
  }

  // 检查当前日期的outcome
  if (props.task.schedules) {
    const schedule = props.task.schedules.find((s) => s.scheduled_day === props.scheduleDay)
    if (schedule && schedule.outcome === 'presence_logged') {
      return 'present'
    }
  }

  return null
})

function handleClick() {
  emit('click')
}

function handleContextMenu(event: MouseEvent) {
  emit('contextmenu', event)
}

function handleCheckboxChange(newState: CheckboxState) {
  emit('checkbox-change', newState)
}
</script>

<template>
  <div
    class="cell-item-task"
    :class="{ 'is-completed': task.is_completed }"
    @click="handleClick"
    @contextmenu="handleContextMenu"
  >
    <CuteDualModeCheckbox
      class="task-checkbox"
      :state="checkboxState"
      size="large"
      @update:state="handleCheckboxChange"
      @click.stop
    />
    <div class="task-content">
      <div class="task-text">
        <div class="task-title" :class="{ completed: task.is_completed }">
          {{ task.title }}
        </div>
        <div v-if="task.recurrence_id" class="task-meta">
          <span class="meta-badge">
            <CuteIcon name="Repeat" :size="12" />
            <span>循环</span>
          </span>
        </div>
      </div>
      <AreaTag
        v-if="area"
        class="task-area-tag"
        :name="area.name"
        :color="area.color"
        size="normal"
      />
    </div>
  </div>
</template>

<style scoped>
.cell-item-task {
  display: flex;
  align-items: flex-start;
  gap: 0.8rem;
  padding: 0.8rem;
  border-radius: 0.6rem;
  transition: background-color 0.15s ease;
  cursor: pointer;
}

.cell-item-task:hover {
  background: var(--color-background-hover, #f0f);
}

.task-checkbox {
  flex-shrink: 0;
  margin-top: 0.1rem;
}

.task-content {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.task-text {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  flex: 1;
  min-width: 0;
}

.task-title {
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  overflow-wrap: break-word;
}

.task-title.completed {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.task-meta {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.meta-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.2rem 0.6rem;
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background: var(--color-background-secondary);
  border-radius: 0.4rem;
}

.task-area-tag {
  flex-shrink: 0;
}
</style>

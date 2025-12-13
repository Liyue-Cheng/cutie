<template>
  <div v-if="show" class="modal-overlay" @click="$emit('close')">
    <div class="modal-container" @click.stop>
      <div class="modal-header">
        <h2 class="modal-title">{{ $t('dailyShutdown.editor.title') }}</h2>
        <button class="icon-btn" @click="$emit('close')" :title="$t('common.action.close')">
          <CuteIcon name="X" :size="20" />
        </button>
      </div>

      <div class="modal-body">
        <div class="settings-section">
          <div class="field-label">{{ $t('dailyShutdown.editor.ritualTitleLabel') }}</div>
          <input
            v-model="localRitualTitle"
            class="title-input"
            type="text"
            :placeholder="$t('dailyShutdown.editor.ritualTitlePlaceholder')"
            @blur="saveRitualTitle"
            @keydown.enter.prevent="saveRitualTitle"
          />
        </div>

        <div class="step-list">
          <div
            v-for="step in localSteps"
            :key="step.id"
            class="step-row"
            :class="{ dragging: draggingId === step.id }"
            draggable="true"
            @dragstart="(e) => onDragStart(e, step.id)"
            @dragover.prevent="() => onDragOver(step.id)"
            @drop.prevent="() => onDrop(step.id)"
          >
            <span class="drag-handle" title="Drag to reorder">⋮⋮</span>
            <input
              class="step-input"
              :value="step.title"
              @input="(e) => onTitleInput(step.id, e)"
              @blur="() => onTitleBlur(step.id)"
              @keydown.enter.prevent="() => onTitleBlur(step.id)"
            />
            <button class="icon-btn danger" @click="() => confirmDelete(step.id)" :title="$t('common.action.delete')">
              <CuteIcon name="Trash2" :size="18" />
            </button>
          </div>
        </div>

        <div class="add-row">
          <input
            v-model="newTitle"
            class="add-input"
            type="text"
            :placeholder="$t('dailyShutdown.editor.addPlaceholder')"
            @keydown.enter.prevent="addStep"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'
import { useShutdownRitualStore } from '@/stores/shutdown-ritual'
import type { ShutdownRitualStep } from '@/types/dtos'

defineProps<{ show: boolean }>()
defineEmits<{ close: [] }>()

const { t } = useI18n()
const store = useShutdownRitualStore()

const storeSteps = computed(() => store.allStepsOrdered)

const localSteps = ref<ShutdownRitualStep[]>([])
const originalTitles = ref(new Map<string, string>())

watch(
  storeSteps,
  (steps) => {
    // keep local list in sync when store changes
    localSteps.value = steps.map((s) => ({ ...s }))
    const map = new Map<string, string>()
    for (const s of steps) map.set(s.id, s.title)
    originalTitles.value = map
  },
  { immediate: true }
)

const newTitle = ref('')

const localRitualTitle = ref('')
const originalRitualTitle = ref<string | null>(null)

watch(
  () => store.ritualTitle,
  (title) => {
    originalRitualTitle.value = title ?? null
    localRitualTitle.value = title ?? ''
  },
  { immediate: true }
)

const draggingId = ref<string | null>(null)
const dragOverId = ref<string | null>(null)

function onDragStart(e: DragEvent, id: string) {
  draggingId.value = id
  e.dataTransfer?.setData('text/plain', id)
  e.dataTransfer?.setDragImage(new Image(), 0, 0)
}

function onDragOver(id: string) {
  dragOverId.value = id
}

async function onDrop(targetId: string) {
  const sourceId = draggingId.value
  draggingId.value = null
  dragOverId.value = null
  if (!sourceId || sourceId === targetId) return

  const list = [...localSteps.value]
  const fromIndex = list.findIndex((s) => s.id === sourceId)
  const toIndex = list.findIndex((s) => s.id === targetId)
  if (fromIndex === -1 || toIndex === -1) return

  const [moved] = list.splice(fromIndex, 1)
  list.splice(toIndex, 0, moved!)
  localSteps.value = list

  const newIndex = list.findIndex((s) => s.id === sourceId)
  const prev = newIndex > 0 ? list[newIndex - 1] : null
  const next = newIndex < list.length - 1 ? list[newIndex + 1] : null

  await pipeline.dispatch('shutdown_ritual.step.reorder', {
    step_id: sourceId,
    prev_step_id: prev?.id ?? null,
    next_step_id: next?.id ?? null,
  })
}

function onTitleInput(stepId: string, e: Event) {
  const value = (e.target as HTMLInputElement).value
  const list = [...localSteps.value]
  const idx = list.findIndex((s) => s.id === stepId)
  if (idx === -1) return
  list[idx] = { ...list[idx]!, title: value }
  localSteps.value = list
}

async function onTitleBlur(stepId: string) {
  const step = localSteps.value.find((s) => s.id === stepId)
  if (!step) return
  const nextTitle = step.title.trim()
  const oldTitle = originalTitles.value.get(stepId) ?? ''
  if (!nextTitle || nextTitle === oldTitle) {
    // revert empty edits
    if (!nextTitle) {
      onTitleInput(stepId, { target: { value: oldTitle } } as any)
    }
    return
  }

  await pipeline.dispatch('shutdown_ritual.step.update', {
    id: stepId,
    title: nextTitle,
  })
}

async function addStep() {
  const title = newTitle.value.trim()
  if (!title) return
  newTitle.value = ''
  await pipeline.dispatch('shutdown_ritual.step.create', { title })
}

async function confirmDelete(stepId: string) {
  if (!confirm(t('dailyShutdown.editor.deleteConfirm'))) return
  await pipeline.dispatch('shutdown_ritual.step.delete', { id: stepId })
}

async function saveRitualTitle() {
  const next = localRitualTitle.value.trim()
  const prev = (originalRitualTitle.value ?? '').trim()

  // Treat empty as "use default" (clear DB title -> fallback to i18n)
  if (!next) {
    if (!prev) return
    await pipeline.dispatch('shutdown_ritual.settings.update', { title: null })
    return
  }

  if (next === prev) return
  await pipeline.dispatch('shutdown_ritual.settings.update', { title: next })
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
  backdrop-filter: blur(4px);
}

.modal-container {
  width: 56rem;
  max-width: 95vw;
  background-color: var(--color-background-primary, #f0f);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-xl, #f0f);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.6rem 2rem;
  border-bottom: 1px solid var(--color-border-default, #f0f);
  background-color: var(--color-background-secondary, #f0f);
}

.modal-title {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 3.4rem;
  height: 3.4rem;
  border: none;
  border-radius: 0.8rem;
  background: transparent;
  color: var(--color-text-secondary, #f0f);
  cursor: pointer;
  transition: all 0.2s ease;
}

.icon-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.icon-btn.danger {
  color: var(--color-danger, #f0f);
}

.modal-body {
  padding: 1.6rem 2rem 2rem;
}

.settings-section {
  margin-bottom: 1.2rem;
}

.field-label {
  font-size: 1.25rem;
  color: var(--color-text-tertiary, #f0f);
  line-height: 1.4;
  margin-bottom: 0.6rem;
}

.title-input {
  width: 100%;
  height: 4rem;
  padding: 0 1.2rem;
  border-radius: 0.9rem;
  border: 1px solid var(--color-border-default, #f0f);
  background: var(--color-background-primary, #f0f);
  color: var(--color-text-primary, #f0f);
  font-size: 1.5rem;
  line-height: 1.4;
}

.title-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

.step-list {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.step-row {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem 1rem;
  border-radius: 0.9rem;
  border: 1px solid var(--color-border-default, #f0f);
  background-color: var(--color-background-primary, #f0f);
}

.step-row.dragging {
  opacity: 0.6;
}

.drag-handle {
  user-select: none;
  cursor: grab;
  color: var(--color-text-tertiary, #f0f);
  font-size: 1.8rem;
  line-height: 1;
  padding: 0 0.4rem;
}

.step-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  color: var(--color-text-primary, #f0f);
  font-size: 1.5rem;
  line-height: 1.4;
}

.add-row {
  margin-top: 1.2rem;
  padding-top: 1.2rem;
  border-top: 1px solid var(--color-border-subtle, #f0f);
}

.add-input {
  width: 100%;
  height: 4rem;
  padding: 0 1.2rem;
  border-radius: 0.9rem;
  border: 1px solid var(--color-border-default, #f0f);
  background: var(--color-background-primary, #f0f);
  color: var(--color-text-primary, #f0f);
  font-size: 1.5rem;
  line-height: 1.4;
}

.add-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}
</style>



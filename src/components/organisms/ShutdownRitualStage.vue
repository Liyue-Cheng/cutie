<template>
  <div class="shutdown-ritual-stage">
    <div class="topbar">
      <button class="icon-btn" @click="$emit('back')" :title="$t('dailyShutdown.back')">
        <span class="back-icon">←</span>
      </button>

      <div class="topbar-spacer"></div>

      <button class="icon-btn" @click="editorOpen = true" :title="$t('common.action.edit')">
        <span class="menu-icon">☰</span>
      </button>
    </div>

    <div class="content">
      <div v-if="steps.length === 0" class="empty-state">
        <div class="empty-circle">○</div>
        <div class="empty-title">{{ $t('dailyShutdown.stage2.emptyTitle') }}</div>
        <div class="empty-hint">{{ $t('dailyShutdown.stage2.emptyHint') }}</div>
      </div>

      <div v-else class="steps-wrapper">
        <!-- 顶部大圆 -->
        <div class="timeline-start">
          <div class="start-circle-col">
            <div class="start-circle"></div>
          </div>
          <span class="start-title">{{ $t('dailyShutdown.stage2.title') }}</span>
        </div>
        <!-- 顶部连接线 -->
        <div class="dots-connector">
          <span class="dot"></span>
          <span class="dot"></span>
          <span class="dot"></span>
        </div>

        <template v-for="(step, index) in steps" :key="step.id">
          <div class="step-item">
            <div class="step-checkbox-col">
              <CuteCheckbox
                :checked="isCompleted(step.id)"
                size="3.2rem"
                @update:checked="() => toggle(step.id)"
              />
            </div>
            <span class="step-title">{{ step.title }}</span>
          </div>
          <!-- 三个点连接线（除了最后一项） -->
          <div v-if="index < steps.length - 1" class="dots-connector">
            <span class="dot"></span>
            <span class="dot"></span>
            <span class="dot"></span>
          </div>
        </template>
      </div>

      <div class="bottom-hint">
        {{ $t('dailyShutdown.stage2.hint') }}
      </div>
    </div>

    <ShutdownRitualEditorModal v-if="editorOpen" :show="editorOpen" @close="editorOpen = false" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { pipeline } from '@/cpu'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import ShutdownRitualEditorModal from '@/components/organisms/ShutdownRitualEditorModal.vue'
import { useShutdownRitualStore } from '@/stores/shutdown-ritual'

const props = defineProps<{ date: string }>()
defineEmits<{ back: [] }>()

const store = useShutdownRitualStore()
const editorOpen = ref(false)

const steps = computed(() => store.allStepsOrdered)

async function ensureLoaded(date: string) {
  if (store.currentDate === date) return
  await store.fetchState_DMA(date)
}

onMounted(async () => {
  await ensureLoaded(props.date)
})

watch(
  () => props.date,
  async (date) => {
    await ensureLoaded(date)
  }
)

function isCompleted(stepId: string) {
  return Boolean(store.getProgressByStepId_Mux(stepId)?.completed_at)
}

async function toggle(stepId: string) {
  await pipeline.dispatch('shutdown_ritual.progress.toggle', {
    step_id: stepId,
    date: props.date,
  })
}
</script>

<style scoped>
.shutdown-ritual-stage {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--color-background-content, #f0f);
}

.topbar {
  height: 5.2rem;
  display: flex;
  align-items: center;
  padding: 0 1.2rem;
  background-color: var(--color-background-secondary, #f0f);
}

.topbar-spacer {
  flex: 1;
}

.icon-btn {
  width: 3.6rem;
  height: 3.6rem;
  border-radius: 0.9rem;
  border: 1px solid transparent;
  background: transparent;
  color: var(--color-text-secondary, #f0f);
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn:hover {
  background: var(--color-background-hover, #f0f);
  border-color: var(--color-border-default, #f0f);
  color: var(--color-text-primary, #f0f);
}

.back-icon {
  font-size: 1.8rem;
  line-height: 1;
}

.menu-icon {
  font-size: 2rem;
  line-height: 1;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 2.4rem 1.6rem 2rem;
  overflow: auto;
}

.steps-wrapper {
  width: 100%;
  max-width: 56rem;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 0;
}

.timeline-start {
  display: flex;
  align-items: center;
  gap: 2.4rem;
  padding: 0.4rem 0;
}

.start-circle-col {
  width: 3.2rem;
  display: flex;
  justify-content: center;
  flex-shrink: 0;
}

.start-circle {
  width: 2.4rem;
  height: 2.4rem;
  border-radius: 50%;
  background-color: var(--color-text-tertiary, #f0f);
  opacity: 0.6;
}

.start-title {
  font-size: 3.2rem;
  color: var(--color-text-tertiary, #f0f);
  line-height: 1.4;
  font-weight: 500;
}

.step-item {
  display: flex;
  align-items: center;
  gap: 2.4rem;
  padding: 0.4rem 0;
}

.step-checkbox-col {
  width: 3.2rem;
  display: flex;
  justify-content: center;
  flex-shrink: 0;
}

.step-title {
  font-size: 3.2rem;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

.dots-connector {
  width: 3.2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.6rem;
  padding: 0.8rem 0;
}

.dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  background-color: var(--color-text-tertiary, #f0f);
  opacity: 0.6;
}

.bottom-hint {
  margin-top: auto;
  padding-top: 2.4rem;
  text-align: center;
  font-size: 1.35rem;
  color: var(--color-text-tertiary, #f0f);
  line-height: 1.6;
  font-style: italic;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.8rem;
  color: var(--color-text-tertiary, #f0f);
}

.empty-circle {
  font-size: 3.6rem;
  opacity: 0.6;
}

.empty-title {
  font-size: 1.8rem;
  font-weight: 600;
  line-height: 1.4;
  color: var(--color-text-primary, #f0f);
}

.empty-hint {
  font-size: 1.4rem;
  line-height: 1.6;
  opacity: 0.85;
}
</style>

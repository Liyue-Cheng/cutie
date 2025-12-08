<script setup lang="ts">
import { computed } from 'vue'
import type { AssistantToolCall } from '@/types/ai'
import CuteIcon from '@/components/parts/CuteIcon.vue'

const props = defineProps<{
  call: AssistantToolCall
}>()

const statusInfo = computed(() => {
  if (props.call.status === 'success') {
    return {
      icon: 'CheckCircle2',
      label: '执行成功',
      className: 'success',
    }
  }
  return {
    icon: 'AlertTriangle',
    label: '执行失败',
    className: 'error',
  }
})
</script>

<template>
  <div class="tool-card" :class="statusInfo.className">
    <div class="tool-card-header">
      <div class="tool-card-badge">
        <CuteIcon :name="statusInfo.icon" :size="16" />
      </div>
      <div class="tool-card-title">
        <span class="tool-name">{{ call.tool_name }}</span>
        <span class="tool-status">{{ statusInfo.label }}</span>
      </div>
      <span class="tool-id" v-if="call.id">#{{ call.id }}</span>
    </div>

    <div v-if="Object.keys(call.params).length > 0" class="tool-card-section">
      <div class="section-title">参数</div>
      <div class="params-grid">
        <div v-for="(value, key) in call.params" :key="key" class="param-item">
          <span class="param-key">{{ key }}</span>
          <span class="param-value">{{ value }}</span>
        </div>
      </div>
    </div>

    <div class="tool-card-section">
      <div class="section-title">结果</div>
      <p class="result-text">{{ call.message }}</p>
    </div>
  </div>
</template>

<style scoped>
.tool-card {
  border: 1px solid var(--color-border-default);
  border-radius: 1rem;
  padding: 1.2rem;
  background: var(--color-background-content);
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.tool-card.success {
  border-color: var(--color-border-success, #f0f);
  background: var(--color-success-light, #f0f);
}

.tool-card.error {
  border-color: var(--color-border-error, #f0f);
  background: var(--color-danger-light, #f0f);
}

.tool-card-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.tool-card-badge {
  width: 2.6rem;
  height: 2.6rem;
  border-radius: 0.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-overlay-light, #f0f);
}

.tool-card-title {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.tool-name {
  font-weight: 600;
  font-size: 1.4rem;
}

.tool-status {
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
}

.tool-id {
  font-size: 1.1rem;
  color: var(--color-text-tertiary);
}

.tool-card-section {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.section-title {
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-tertiary);
}

.params-grid {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.param-item {
  display: flex;
  gap: 0.6rem;
  font-size: 1.3rem;
}

.param-key {
  min-width: 8rem;
  color: var(--color-text-tertiary);
  font-weight: 500;
}

.param-value {
  color: var(--color-text-primary);
  word-break: break-all;
}

.result-text {
  margin: 0;
  font-size: 1.3rem;
  line-height: 1.5;
  white-space: pre-wrap;
}
</style>

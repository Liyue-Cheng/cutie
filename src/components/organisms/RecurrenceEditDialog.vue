<template>
  <div class="edit-dialog-overlay" @click.self="handleClose">
    <div class="edit-dialog">
      <div class="dialog-header">
        <h3>{{ $t('recurrence.title.edit') }}</h3>
        <button class="close-btn" @click="handleClose">
          <CuteIcon name="X" :size="18" />
        </button>
      </div>

      <div class="dialog-content">
        <!-- 日期范围 -->
        <div class="form-group">
          <label>{{ $t('recurrence.label.startDate') }}</label>
          <input
            v-model="formData.start_date"
            type="date"
            class="form-input"
            :placeholder="$t('recurrence.label.startDateHint')"
          />
        </div>

        <div class="form-group">
          <label>{{ $t('recurrence.label.endDate') }}</label>
          <input
            v-model="formData.end_date"
            type="date"
            class="form-input"
            :placeholder="$t('recurrence.label.endDateHint')"
          />
        </div>

        <!-- 过期行为 -->
        <div class="form-group">
          <label>{{ $t('recurrence.label.expiryBehavior') }}</label>
          <select v-model="formData.expiry_behavior" class="form-select">
            <option value="CARRYOVER_TO_STAGING">{{ $t('recurrence.expiry.carryover') }}</option>
            <option value="EXPIRE">{{ $t('recurrence.expiry.expire') }}</option>
          </select>
          <p class="form-hint">{{ $t('recurrence.label.expiryBehaviorHint') }}</p>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="dialog-btn secondary-btn" @click="handleClose">{{ $t('common.action.cancel') }}</button>
        <button class="dialog-btn primary-btn" @click="handleSave">{{ $t('common.action.save') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import type { TaskRecurrence } from '@/types/dtos'

interface Props {
  recurrence: TaskRecurrence
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  save: [updates: Partial<TaskRecurrence>]
}>()

// 表单数据
const formData = ref({
  start_date: props.recurrence.start_date || '',
  end_date: props.recurrence.end_date || '',
  expiry_behavior: props.recurrence.expiry_behavior || 'CARRYOVER_TO_STAGING',
})

function handleClose() {
  emit('close')
}

function handleSave() {
  const updates: Partial<TaskRecurrence> = {
    start_date: formData.value.start_date || null,
    end_date: formData.value.end_date || null,
    expiry_behavior: formData.value.expiry_behavior as 'CARRYOVER_TO_STAGING' | 'EXPIRE',
  }
  emit('save', updates)
}
</script>

<style scoped>
.edit-dialog-overlay {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay-medium, #f0f);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
}

.edit-dialog {
  width: 100%;
  max-width: 50rem;
  background-color: var(--color-background-primary, #f0f);
  border-radius: 1rem;
  box-shadow: var(--shadow-xl, #f0f);
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.6rem 2rem;
  border-bottom: 1px solid var(--color-border-default, #f0f);
}

.dialog-header h3 {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.close-btn {
  all: unset;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  border-radius: 0.4rem;
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.2s ease;
}

.close-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary);
}

.dialog-content {
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.6rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.form-group label {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.form-input,
.form-select {
  padding: 0.8rem 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  transition: all 0.2s ease;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--color-primary, #f0f);
  background-color: var(--color-background-primary, #f0f);
}

.form-hint {
  margin: 0;
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
}

.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 1.2rem;
  padding: 1.6rem 2rem;
  border-top: 1px solid var(--color-border-default, #f0f);
}

.dialog-btn {
  all: unset;
  box-sizing: border-box;
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
</style>

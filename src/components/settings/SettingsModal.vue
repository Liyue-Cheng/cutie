<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <div class="settings-modal">
      <!-- 头部 -->
      <div class="modal-header">
        <h2 class="modal-title">Debug Settings</h2>
        <button class="close-button" @click="emit('close')">
          <CuteIcon name="X" :size="20" />
        </button>
      </div>

      <!-- 内容 -->
      <div class="modal-content">
        <div class="settings-list">
          <!-- String 测试 -->
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Debug String</label>
              <span class="setting-type">string</span>
            </div>
            <input
              type="text"
              :value="store.getSettingValue('debug.test_string', 'Hello World')"
              @change="updateSetting('debug.test_string', $event, 'string')"
              class="setting-input"
            />
          </div>

          <!-- Number 测试 -->
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Debug Number</label>
              <span class="setting-type">number</span>
            </div>
            <input
              type="number"
              :value="store.getSettingValue('debug.test_number', 42)"
              @change="updateSetting('debug.test_number', $event, 'number')"
              class="setting-input"
            />
          </div>

          <!-- Boolean 测试 (Checkbox) -->
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Debug Boolean</label>
              <span class="setting-type">boolean</span>
            </div>
            <label class="checkbox-wrapper">
              <input
                type="checkbox"
                :checked="store.getSettingValue('debug.test_boolean', false)"
                @change="updateSetting('debug.test_boolean', $event, 'boolean')"
                class="setting-checkbox"
              />
              <span class="checkbox-label">Enable</span>
            </label>
          </div>

          <!-- Float 测试 -->
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Debug Float</label>
              <span class="setting-type">number (float)</span>
            </div>
            <input
              type="number"
              step="0.01"
              :value="store.getSettingValue('debug.test_float', 3.14)"
              @change="updateSetting('debug.test_float', $event, 'number')"
              class="setting-input"
            />
          </div>

          <!-- Toggle 测试 (Switch) -->
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Debug Toggle</label>
              <span class="setting-type">boolean (toggle)</span>
            </div>
            <label class="toggle-switch">
              <input
                type="checkbox"
                :checked="store.getSettingValue('debug.test_toggle', true)"
                @change="updateSetting('debug.test_toggle', $event, 'boolean')"
              />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="modal-footer">
        <button @click="resetAllSettings" class="reset-button">
          Reset All
        </button>
        <button @click="emit('close')" class="close-action-button">
          Close
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { pipeline } from '@/cpu'
import { useUserSettingsStore } from '@/stores/user-settings'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import type { ValueType } from '@/types/user-settings'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const store = useUserSettingsStore()

function updateSetting(key: string, event: Event, valueType: ValueType) {
  const target = event.target as HTMLInputElement
  let value: any

  if (valueType === 'boolean') {
    value = target.checked
  } else if (valueType === 'number') {
    value = target.valueAsNumber
  } else {
    value = target.value
  }

  pipeline.dispatch('user_settings.update', {
    key,
    value,
    value_type: valueType,
  })
}

async function resetAllSettings() {
  if (confirm('Reset all settings to defaults?')) {
    await pipeline.dispatch('user_settings.reset', {})
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 50%);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.settings-modal {
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  background: var(--color-surface);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgb(0 0 0 / 20%);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 头部 */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.5rem;
  border-bottom: 1px solid var(--color-border);
}

.modal-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.close-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  border: none;
  background: transparent;
  border-radius: 6px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-button:hover {
  background: var(--color-surface-hover);
  color: var(--color-text);
}

/* 内容 */
.modal-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.settings-list {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.5rem;
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.setting-label {
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--color-text);
}

.setting-type {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
  font-family: monospace;
}

.setting-input {
  width: 200px;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.9375rem;
  transition: border-color 0.2s ease;
}

.setting-input:hover {
  border-color: var(--color-primary);
}

.setting-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-subtle);
}

/* Checkbox */
.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.setting-checkbox {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.checkbox-label {
  font-size: 0.9375rem;
  color: var(--color-text);
}

/* Toggle Switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 26px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--color-border);
  transition: 0.3s;
  border-radius: 26px;
}

.toggle-slider::before {
  position: absolute;
  content: '';
  height: 18px;
  width: 18px;
  left: 4px;
  bottom: 4px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .toggle-slider {
  background-color: var(--color-primary);
}

input:checked + .toggle-slider::before {
  transform: translateX(22px);
}

/* 底部 */
.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-top: 1px solid var(--color-border);
  gap: 1rem;
}

.reset-button {
  padding: 0.5rem 1rem;
  border: 1px solid #ef4444;
  border-radius: 6px;
  background: transparent;
  color: #ef4444;
  font-size: 0.9375rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.reset-button:hover {
  background: #fef2f2;
}

.close-action-button {
  padding: 0.5rem 1.5rem;
  border: none;
  border-radius: 6px;
  background: var(--color-primary);
  color: white;
  font-size: 0.9375rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-action-button:hover {
  opacity: 0.9;
}
</style>


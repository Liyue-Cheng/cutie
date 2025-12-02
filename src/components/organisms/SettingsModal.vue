<template>
  <div v-if="show" class="modal-overlay" @click="$emit('close')">
    <div class="settings-container" @click.stop>
      <!-- 头部 -->
      <div class="settings-header">
        <h2 class="settings-title">{{ $t('settings.title') }}</h2>
        <button class="close-btn" @click="$emit('close')" :title="$t('common.action.close')">
          <CuteIcon name="X" :size="20" />
        </button>
      </div>

      <!-- 主体：左右两栏 -->
      <div class="settings-body">
        <!-- 左侧：分类导航 -->
        <div class="settings-sidebar">
          <nav class="category-nav">
            <button
              v-for="category in categories"
              :key="category.id"
              :class="['category-item', { active: activeCategory === category.id }]"
              @click="activeCategory = category.id"
            >
              <CuteIcon :name="category.icon" :size="18" />
              <span class="category-name">{{ category.name }}</span>
            </button>
          </nav>
        </div>

        <!-- 右侧：设置内容 -->
        <div class="settings-content">
          <!-- AI 分类 -->
          <div v-if="activeCategory === 'ai'" class="settings-panel">
            <div class="panel-header">
              <h3 class="panel-title">{{ $t('settings.ai.title') }}</h3>
              <p class="panel-description">{{ $t('settings.ai.description') }}</p>
            </div>

            <div class="settings-list">
              <div class="settings-subsection">
                <h4 class="subsection-title">{{ $t('settings.ai.conversation.title') }}</h4>
                <p class="subsection-description">
                  {{ $t('settings.ai.conversation.description') }}
                </p>

                <div class="setting-item">
                  <div class="setting-info">
                    <label class="setting-label">{{ $t('settings.ai.field.apiBaseUrl') }}</label>
                    <span class="setting-description">{{ $t('settings.ai.field.apiBaseUrlDesc') }}</span>
                  </div>
                  <input
                    type="text"
                    :value="store.getSettingValue('ai.conversation.api_base_url', '')"
                    @change="updateSetting('ai.conversation.api_base_url', $event, 'string')"
                    class="setting-input"
                    placeholder="https://..."
                    autocomplete="off"
                  />
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <label class="setting-label">{{ $t('settings.ai.field.apiKey') }}</label>
                    <span class="setting-description">{{ $t('settings.ai.field.apiKeyDesc') }}</span>
                  </div>
                  <input
                    type="password"
                    :value="store.getSettingValue('ai.conversation.api_key', '')"
                    @change="updateSetting('ai.conversation.api_key', $event, 'string')"
                    class="setting-input"
                    placeholder="sk-..."
                    autocomplete="off"
                  />
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <label class="setting-label">{{ $t('settings.ai.field.model') }}</label>
                    <span class="setting-description">{{ $t('settings.ai.field.modelDesc') }}</span>
                  </div>
                  <input
                    type="text"
                    :value="store.getSettingValue('ai.conversation.model', '')"
                    @change="updateSetting('ai.conversation.model', $event, 'string')"
                    class="setting-input"
                    :placeholder="$t('settings.ai.field.model')"
                    autocomplete="off"
                  />
                </div>
              </div>

              <div class="settings-subsection">
                <h4 class="subsection-title">{{ $t('settings.ai.quick.title') }}</h4>
                <p class="subsection-description">
                  {{ $t('settings.ai.quick.description') }}
                </p>

                <div class="setting-item">
                  <div class="setting-info">
                    <label class="setting-label">{{ $t('settings.ai.field.apiBaseUrl') }}</label>
                    <span class="setting-description">{{ $t('settings.ai.field.apiBaseUrlDesc') }}</span>
                  </div>
                  <input
                    type="text"
                    :value="store.getSettingValue('ai.quick.api_base_url', '')"
                    @change="updateSetting('ai.quick.api_base_url', $event, 'string')"
                    class="setting-input"
                    placeholder="https://..."
                    autocomplete="off"
                  />
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <label class="setting-label">{{ $t('settings.ai.field.apiKey') }}</label>
                    <span class="setting-description">{{ $t('settings.ai.field.apiKeyDesc') }}</span>
                  </div>
                  <input
                    type="password"
                    :value="store.getSettingValue('ai.quick.api_key', '')"
                    @change="updateSetting('ai.quick.api_key', $event, 'string')"
                    class="setting-input"
                    placeholder="sk-..."
                    autocomplete="off"
                  />
                </div>

                <div class="setting-item">
                  <div class="setting-info">
                    <label class="setting-label">{{ $t('settings.ai.field.model') }}</label>
                    <span class="setting-description">{{ $t('settings.ai.field.modelDesc') }}</span>
                  </div>
                  <input
                    type="text"
                    :value="store.getSettingValue('ai.quick.model', '')"
                    @change="updateSetting('ai.quick.model', $event, 'string')"
                    class="setting-input"
                    :placeholder="$t('settings.ai.field.model')"
                    autocomplete="off"
                  />
                </div>
              </div>
            </div>
          </div>

          <!-- Debug 分类 -->
          <div v-else-if="activeCategory === 'debug'" class="settings-panel">
            <div class="panel-header">
              <h3 class="panel-title">{{ $t('settings.debug.title') }}</h3>
              <p class="panel-description">{{ $t('settings.debug.description') }}</p>
            </div>

            <div class="settings-list">
              <!-- String 测试 -->
              <div class="setting-item">
                <div class="setting-info">
                  <label class="setting-label">Test String</label>
                  <span class="setting-description">字符串类型测试</span>
                </div>
                <input
                  type="text"
                  :value="store.getSettingValue('debug.test_string', 'Hello World')"
                  @change="updateSetting('debug.test_string', $event, 'string')"
                  class="setting-input"
                  placeholder="Enter string..."
                />
              </div>

              <!-- Number 测试 -->
              <div class="setting-item">
                <div class="setting-info">
                  <label class="setting-label">Test Number</label>
                  <span class="setting-description">整数类型测试</span>
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
                  <label class="setting-label">Test Boolean</label>
                  <span class="setting-description">布尔类型测试（复选框）</span>
                </div>
                <label class="checkbox-wrapper">
                  <CuteCheckbox
                    :checked="store.getSettingValue('debug.test_boolean', false)"
                    size="small"
                    @update:checked="
                      (val) =>
                        pipeline.dispatch('user_settings.update', {
                          key: 'debug.test_boolean',
                          value: val,
                          value_type: 'boolean',
                        })
                    "
                  />
                  <span class="checkbox-label">Enable</span>
                </label>
              </div>

              <!-- Float 测试 -->
              <div class="setting-item">
                <div class="setting-info">
                  <label class="setting-label">Test Float</label>
                  <span class="setting-description">浮点数类型测试</span>
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
                  <label class="setting-label">Test Toggle</label>
                  <span class="setting-description">布尔类型测试（开关）</span>
                </div>
                <CuteCheckbox
                  :checked="store.getSettingValue('debug.test_toggle', true)"
                  size="small"
                  @update:checked="
                    (val) =>
                      pipeline.dispatch('user_settings.update', {
                        key: 'debug.test_toggle',
                        value: val,
                        value_type: 'boolean',
                      })
                  "
                />
              </div>
            </div>
          </div>

          <!-- Appearance 分类 -->
          <div v-else-if="activeCategory === 'appearance'" class="settings-panel">
            <div class="panel-header">
              <h3 class="panel-title">{{ $t('settings.appearance.title') }}</h3>
              <p class="panel-description">{{ $t('settings.appearance.description') }}</p>
            </div>

            <div class="settings-list">
              <!-- 主题选择 -->
              <div class="setting-item">
                <div class="setting-info">
                  <label class="setting-label">{{ $t('settings.appearance.theme') }}</label>
                  <span class="setting-description">{{ $t('settings.appearance.themeDesc') }}</span>
                </div>
                <select
                  :value="store.getSettingValue('appearance.theme', 'business')"
                  @change="updateSetting('appearance.theme', $event, 'string')"
                  class="setting-select"
                >
                  <option value="rose-pine">{{ $t('settings.theme.rosePine') }}</option>
                  <option value="rose-pine-dawn">{{ $t('settings.theme.rosePineDawn') }}</option>
                  <option value="rose-pine-moon">{{ $t('settings.theme.rosePineMoon') }}</option>
                  <option value="cutie">{{ $t('settings.theme.cutie') }}</option>
                  <option value="business">{{ $t('settings.theme.business') }}</option>
                </select>
              </div>
            </div>
          </div>

          <!-- 其他分类的占位 -->
          <div v-else class="settings-panel">
            <div class="panel-header">
              <h3 class="panel-title">{{ getCategoryName(activeCategory) }}</h3>
              <p class="panel-description">{{ $t('settings.empty.comingSoon') }}</p>
            </div>
            <div class="empty-state">
              <CuteIcon name="Settings" :size="48" />
              <p>{{ $t('settings.empty.title') }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="settings-footer">
        <button @click="resetAllSettings" class="reset-btn">
          <CuteIcon name="RotateCcw" :size="16" />
          <span>{{ $t('settings.action.resetAll') }}</span>
        </button>
        <button @click="$emit('close')" class="close-action-btn">{{ $t('settings.action.close') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { pipeline } from '@/cpu'
import { useUserSettingsStore } from '@/stores/user-settings'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import type { ValueType } from '@/types/user-settings'

defineProps<{
  show: boolean
}>()

defineEmits(['close'])

const { t } = useI18n()
const store = useUserSettingsStore()

// 当前激活的分类
const activeCategory = ref<string>('ai')

// 分类定义
const categories = [
  { id: 'ai', name: t('settings.category.ai'), icon: 'Sparkles' as const },
  { id: 'appearance', name: t('settings.category.appearance'), icon: 'Palette' as const },
  { id: 'behavior', name: t('settings.category.behavior'), icon: 'SlidersHorizontal' as const },
  { id: 'data', name: t('settings.category.data'), icon: 'Database' as const },
  { id: 'account', name: t('settings.category.account'), icon: 'User' as const },
  { id: 'debug', name: t('settings.category.debug'), icon: 'Bug' as const },
  { id: 'system', name: t('settings.category.system'), icon: 'Settings' as const },
]

onMounted(async () => {
  // 加载所有设置
  await pipeline.dispatch('user_settings.fetch_all', {})
})

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
  if (confirm(t('confirm.resetSettings'))) {
    await pipeline.dispatch('user_settings.reset', {})
  }
}

function getCategoryName(categoryId: string): string {
  return categories.find((c) => c.id === categoryId)?.name || categoryId
}
</script>

<style scoped>
/* ==================== 模态框遮罩 ==================== */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: var(--color-overlay-heavy);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

/* ==================== 设置容器 ==================== */
.settings-container {
  width: 90rem;
  max-width: 95vw;
  height: 80vh; /* 固定高度 */
  background-color: var(--color-background-primary, #faf4ed);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ==================== 头部 ==================== */
.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2rem 2.4rem;
  background-color: var(--color-background-secondary, #fffaf3);
  border-bottom: 1px solid var(--color-border-default, rgb(0 0 0 / 10%));
}

.settings-title {
  font-size: 2.2rem;
  font-weight: 600;
  color: var(--color-text-primary, #575279);
  margin: 0;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: 0.6rem;
  color: var(--color-text-secondary, #797593);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary, #575279);
}

/* ==================== 主体：左右两栏 ==================== */
.settings-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* ==================== 左侧：分类导航 ==================== */
.settings-sidebar {
  width: 22rem;
  background-color: var(--color-background-secondary, #fffaf3);
  border-right: 1px solid var(--color-border-default, rgb(0 0 0 / 10%));
  overflow-y: auto;
}

.category-nav {
  display: flex;
  flex-direction: column;
  padding: 1.2rem;
  gap: 0.4rem;
}

.category-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.2rem;
  background: transparent;
  border: none;
  border-radius: 0.8rem;
  color: var(--color-text-secondary, #797593);
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.category-item:hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary, #575279);
}

.category-item.active {
  background-color: var(--color-button-primary-bg);
  color: white;
}

.category-name {
  flex: 1;
}

/* ==================== 右侧：设置内容 ==================== */
.settings-content {
  flex: 1;
  overflow-y: auto;
  background-color: var(--color-background-primary, #faf4ed);
}

.settings-panel {
  padding: 2.4rem;
}

.panel-header {
  margin-bottom: 2.4rem;
}

.panel-title {
  font-size: 2rem;
  font-weight: 600;
  color: var(--color-text-primary, #575279);
  margin: 0 0 0.8rem;
}

.panel-description {
  font-size: 1.4rem;
  color: var(--color-text-secondary, #797593);
  margin: 0;
}

.settings-list {
  display: flex;
  flex-direction: column;
  gap: 1.6rem;
}

.settings-subsection {
  display: flex;
  flex-direction: column;
  gap: 1.6rem;
  padding: 1.6rem;
  background-color: var(--color-background-secondary, #fffaf3);
  border: 1px solid var(--color-border-soft, rgb(0 0 0 / 10%));
  border-radius: 0.8rem;
}

.settings-subsection + .settings-subsection {
  margin-top: 1.6rem;
}

.subsection-title {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary, #575279);
}

.subsection-description {
  margin: 0;
  font-size: 1.3rem;
  color: var(--color-text-tertiary, #9893a5);
  line-height: 1.6;
}

/* ==================== 空状态 ==================== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  color: var(--color-text-tertiary, #9893a5);
  gap: 1.2rem;
}

.empty-state p {
  font-size: 1.5rem;
  margin: 0;
}

/* ==================== 设置项 ==================== */
.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 2rem;
  padding: 1.4rem;
  background-color: var(--color-background-secondary, #fffaf3);
  border: 1.5px solid var(--color-border-default, rgb(0 0 0 / 10%));
  border-radius: 0.8rem;
  transition: all 0.2s ease;
}

.setting-item:hover {
  border-color: rgb(0 0 0 / 15%);
  box-shadow: var(--shadow-sm);
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.setting-label {
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary, #575279);
}

.setting-description {
  font-size: 1.3rem;
  color: var(--color-text-tertiary, #9893a5);
}

/* ==================== 输入框 ==================== */
.setting-input {
  width: 20rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  border: 1.5px solid var(--color-border-default, rgb(0 0 0 / 10%));
  border-radius: 0.8rem;
  background: var(--color-background-primary, #faf4ed);
  color: var(--color-text-primary, #575279);
  font-size: 1.4rem;
  transition: all 0.2s ease;
}

.setting-input::placeholder {
  color: var(--color-text-tertiary, #9893a5);
}

.setting-input:hover {
  border-color: rgb(0 0 0 / 15%);
}

.setting-input:focus {
  outline: none;
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

/* ==================== 下拉选择框 ==================== */
.setting-select {
  width: 20rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  border: 1.5px solid var(--color-border-default, rgb(0 0 0 / 10%));
  border-radius: 0.8rem;
  background: var(--color-background-primary, #faf4ed);
  color: var(--color-text-primary, #575279);
  font-size: 1.4rem;
  transition: all 0.2s ease;
  cursor: pointer;
}

.setting-select:hover {
  border-color: rgb(0 0 0 / 15%);
}

.setting-select:focus {
  outline: none;
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

/* ==================== Checkbox ==================== */
.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  cursor: pointer;
}

.setting-checkbox {
  width: 2rem;
  height: 2rem;
  cursor: pointer;
  accent-color: var(--color-button-primary-bg);
}

.checkbox-label {
  font-size: 1.4rem;
  color: var(--color-text-primary, #575279);
  font-weight: 500;
}

/* ==================== Toggle Switch ==================== */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 5.2rem;
  height: 2.8rem;
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
  background-color: var(--color-border-default, rgb(0 0 0 / 15%));
  transition: 0.3s;
  border-radius: 2.8rem;
}

.toggle-slider::before {
  position: absolute;
  content: '';
  height: 2rem;
  width: 2rem;
  left: 0.4rem;
  bottom: 0.4rem;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
  box-shadow: var(--shadow-sm);
}

input:checked + .toggle-slider {
  background-color: var(--color-button-primary-bg);
}

input:checked + .toggle-slider::before {
  transform: translateX(2.4rem);
}

/* ==================== 底部 ==================== */
.settings-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2rem 2.4rem;
  border-top: 1px solid var(--color-border-default, rgb(0 0 0 / 10%));
  background-color: var(--color-background-secondary, #fffaf3);
  gap: 1rem;
}

.reset-btn {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.4rem;
  border: 1.5px solid var(--color-danger);
  border-radius: 0.8rem;
  background: transparent;
  color: var(--color-danger);
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.reset-btn:hover {
  background-color: var(--color-danger-light);
  transform: translateY(-1px);
}

.reset-btn:active {
  transform: translateY(0);
}

.close-action-btn {
  height: 3.6rem;
  padding: 0 2rem;
  border: none;
  border-radius: 0.8rem;
  background-color: var(--color-button-primary-bg);
  color: white;
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-action-btn:hover {
  background-color: var(--color-status-completed);
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.close-action-btn:active {
  transform: translateY(0);
}
</style>

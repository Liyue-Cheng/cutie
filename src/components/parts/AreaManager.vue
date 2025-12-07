<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAreaStore } from '@/stores/area'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'

defineEmits(['close'])

const { t } = useI18n()
const areaStore = useAreaStore()

const newAreaName = ref('')
const newAreaColor = ref('#4A90E2')
const editingArea = ref<{ id: string; name: string; color: string } | null>(null)
const isAiColorLoading = ref(false)
const isEditAiColorLoading = ref(false)

onMounted(async () => {
  await areaStore.fetchAll()
})

async function handleCreate() {
  if (!newAreaName.value.trim()) return

  try {
    await pipeline.dispatch('area.create', {
      name: newAreaName.value.trim(),
      color: newAreaColor.value,
    })

    newAreaName.value = ''
    newAreaColor.value = '#4A90E2'
  } catch (error) {
    console.error('创建 Area 失败:', error)
    alert(t('message.error.createAreaFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
  }
}

async function handleAiSuggestColor() {
  if (!newAreaName.value.trim()) {
    alert(t('area.message.enterName'))
    return
  }

  isAiColorLoading.value = true
  try {
    const result = await pipeline.dispatch('area.suggest_color', {
      area_name: newAreaName.value.trim(),
    })
    newAreaColor.value = result.suggested_color
  } catch (error) {
    console.error('AI 染色失败:', error)
    alert(t('message.error.aiColorFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
  } finally {
    isAiColorLoading.value = false
  }
}

function startEdit(area: (typeof areaStore.allAreas)[0]) {
  editingArea.value = {
    id: area.id,
    name: area.name,
    color: area.color,
  }
}

function cancelEdit() {
  editingArea.value = null
}

async function saveEdit() {
  if (!editingArea.value) return

  try {
    await pipeline.dispatch('area.update', {
      id: editingArea.value.id,
      name: editingArea.value.name,
      color: editingArea.value.color,
    })

    editingArea.value = null
  } catch (error) {
    console.error('更新 Area 失败:', error)
    alert(t('message.error.updateAreaFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
  }
}

async function handleEditAiSuggestColor() {
  if (!editingArea.value?.name.trim()) {
    alert(t('area.message.enterName'))
    return
  }

  isEditAiColorLoading.value = true
  try {
    const result = await pipeline.dispatch('area.suggest_color', {
      area_name: editingArea.value.name.trim(),
    })
    editingArea.value.color = result.suggested_color
  } catch (error) {
    console.error('AI 染色失败:', error)
    alert(t('message.error.aiColorFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
  } finally {
    isEditAiColorLoading.value = false
  }
}

async function handleDelete(id: string) {
  if (confirm(t('confirm.deleteArea'))) {
    try {
      await pipeline.dispatch('area.delete', { id })
    } catch (error) {
      console.error('删除 Area 失败:', error)
      alert(t('message.error.deleteAreaFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
    }
  }
}
</script>

<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="manager-container" @click.stop>
      <!-- 头部 -->
      <div class="manager-header">
        <h2 class="manager-title">{{ $t('area.title.manager') }}</h2>
        <button class="close-btn" @click="$emit('close')" :title="$t('common.action.close')">
          <CuteIcon name="X" :size="20" />
        </button>
      </div>

      <!-- 创建新 Area 区域 -->
      <div class="create-section">
        <h3 class="section-title">{{ $t('area.title.create') }}</h3>
        <div class="create-form">
          <input
            v-model="newAreaName"
            type="text"
            :placeholder="$t('area.placeholder.name')"
            class="name-input"
            @keyup.enter="handleCreate"
          />
          <div class="color-picker-wrapper">
            <input v-model="newAreaColor" type="color" class="color-input" :title="$t('area.action.selectColor')" />
            <div class="color-preview" :style="{ backgroundColor: newAreaColor }"></div>
          </div>
          <button
            class="ai-color-btn"
            @click="handleAiSuggestColor"
            :disabled="!newAreaName.trim() || isAiColorLoading"
            :title="$t('area.action.aiColor')"
          >
            <CuteIcon name="Sparkles" :size="16" />
            <span v-if="!isAiColorLoading">AI</span>
            <span v-else>...</span>
          </button>
          <button
            class="add-btn"
            @click="handleCreate"
            :disabled="!newAreaName.trim()"
            :title="$t('area.action.add')"
          >
            <CuteIcon name="Plus" :size="18" />
            <span>{{ $t('area.action.add') }}</span>
          </button>
        </div>
      </div>

      <!-- Area 列表区域 -->
      <div class="areas-section">
        <div class="section-header">
          <h3 class="section-title">{{ $t('area.title.all') }}</h3>
          <span class="area-count">{{ $t('area.count', { n: areaStore.allAreas.length }) }}</span>
        </div>

        <div class="areas-list">
          <div v-if="areaStore.allAreas.length === 0" class="empty-state">
            <CuteIcon name="Tag" :size="48" class="empty-icon" />
            <p class="empty-text">{{ $t('area.empty.title') }}</p>
            <p class="empty-hint">{{ $t('area.empty.hint') }}</p>
          </div>

          <div v-else class="areas-grid">
            <div
              v-for="area in areaStore.allAreas"
              :key="area.id"
              class="area-card"
              :class="{ editing: editingArea?.id === area.id }"
            >
              <!-- 编辑模式 -->
              <div v-if="editingArea?.id === area.id" class="edit-mode">
                <div class="edit-form">
                  <div class="edit-color-wrapper">
                    <input v-model="editingArea.color" type="color" class="edit-color-input" />
                    <div
                      class="edit-color-preview"
                      :style="{ backgroundColor: editingArea.color }"
                    ></div>
                  </div>
                  <input
                    v-model="editingArea.name"
                    type="text"
                    class="edit-name-input"
                    placeholder="Area 名称"
                    @keyup.enter="saveEdit"
                    @keyup.esc="cancelEdit"
                  />
                </div>
                <div class="edit-actions">
                  <button
                    class="edit-btn ai"
                    @click="handleEditAiSuggestColor"
                    :disabled="!editingArea.name.trim() || isEditAiColorLoading"
                    :title="$t('area.action.aiColor')"
                  >
                    <CuteIcon name="Sparkles" :size="14" />
                  </button>
                  <button class="edit-btn save" @click="saveEdit" :title="$t('common.action.save')">
                    <CuteIcon name="Check" :size="16" />
                  </button>
                  <button class="edit-btn cancel" @click="cancelEdit" :title="$t('common.action.cancel')">
                    <CuteIcon name="X" :size="16" />
                  </button>
                </div>
              </div>

              <!-- 查看模式 -->
              <div v-else class="view-mode">
                <div class="area-info">
                  <div class="color-indicator" :style="{ backgroundColor: area.color }"></div>
                  <span class="area-name">{{ area.name }}</span>
                </div>
                <div class="area-actions">
                  <button class="action-btn edit" @click="startEdit(area)" :title="$t('common.action.edit')">
                    <CuteIcon name="Pencil" :size="16" />
                  </button>
                  <button class="action-btn delete" @click="handleDelete(area.id)" :title="$t('common.action.delete')">
                    <CuteIcon name="Trash2" :size="16" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

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

/* ==================== 管理器容器 ==================== */
.manager-container {
  width: 70rem;
  max-width: 90vw;
  max-height: 85vh;
  background-color: var(--color-background-primary, #f0f);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ==================== 头部 ==================== */
.manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2rem 2.4rem;
  background-color: var(--color-background-secondary, #f0f);
  border-bottom: 1px solid var(--color-border-default, #f0f);
}

.manager-title {
  font-size: 2.2rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
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
  color: var(--color-text-secondary, #f0f);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary, #f0f);
}

/* ==================== 创建区域 ==================== */
.create-section {
  padding: 2rem 2.4rem;
  background-color: var(--color-background-secondary, #f0f);
  border-bottom: 1px solid var(--color-border-default, #f0f);
}

.section-title {
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-secondary, #f0f);
  margin: 0 0 1.2rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.create-form {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.name-input {
  flex: 1;
  height: 4rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-primary, #f0f);
  border: 1.5px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  outline: none;
  transition: all 0.2s ease;
}

.name-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

.name-input:focus {
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

.color-picker-wrapper {
  position: relative;
  width: 4.8rem;
  height: 4rem;
}

.color-input {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}

.color-preview {
  width: 100%;
  height: 100%;
  border: 1.5px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
  pointer-events: none;
}

.color-picker-wrapper:hover .color-preview {
  transform: scale(1.05);
  box-shadow: var(--shadow-md, #f0f);
}

.ai-color-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  height: 4rem;
  padding: 0 1.2rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
  background: linear-gradient(135deg, #f5a623 0%, #e94b8b 50%, #9b59b6 100%);
  background-clip: text;
  -webkit-background-clip: text; /* stylelint-disable-line property-no-vendor-prefix */
  -webkit-text-fill-color: transparent;
  border: 1.5px solid transparent;
  background-origin: border-box;
  background-image:
    linear-gradient(
      var(--color-background-secondary, #f0f),
      var(--color-background-secondary, #f0f)
    ),
    linear-gradient(135deg, #f5a623 0%, #e94b8b 50%, #9b59b6 100%);
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.ai-color-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ai-color-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.ai-color-btn:active:not(:disabled) {
  transform: translateY(0);
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  height: 4rem;
  padding: 0 1.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-button-primary-text, #f0f);
  background-color: var(--color-button-primary-bg);
  border: none;
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.add-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.add-btn:hover:not(:disabled) {
  background-color: var(--color-status-completed);
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.add-btn:active:not(:disabled) {
  transform: translateY(0);
}

/* ==================== Areas 列表区域 ==================== */
.areas-section {
  flex: 1;
  overflow-y: auto;
  padding: 2rem 2.4rem;
  background-color: var(--color-background-primary, #faf4ed);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.2rem;
}

.area-count {
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-tertiary, #9893a5);
  padding: 0.4rem 1rem;
  background-color: var(--color-background-secondary, #fffaf3);
  border-radius: 1.2rem;
}

/* ==================== 空状态 ==================== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  text-align: center;
}

.empty-icon {
  color: var(--color-text-tertiary, #9893a5);
  opacity: 0.5;
  margin-bottom: 1.6rem;
}

.empty-text {
  font-size: 1.6rem;
  font-weight: 500;
  color: var(--color-text-secondary, #797593);
  margin: 0 0 0.8rem;
}

.empty-hint {
  font-size: 1.4rem;
  color: var(--color-text-tertiary, #f0f);
  margin: 0;
}

/* ==================== Areas 网格 ==================== */
.areas-grid {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

/* ==================== Area 卡片 ==================== */
.area-card {
  background-color: var(--color-background-secondary, #f0f);
  border: 1.5px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  padding: 1.2rem 1.4rem;
  transition: all 0.2s ease;
}

.area-card:hover {
  border-color: var(--color-border-strong, #f0f);
  box-shadow: var(--shadow-sm);
}

.area-card.editing {
  border-color: var(--color-button-primary-bg);
  box-shadow: var(--shadow-focus);
}

/* ==================== 查看模式 ==================== */
.view-mode {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.area-info {
  display: flex;
  align-items: center;
  gap: 1.2rem;
  flex: 1;
}

.color-indicator {
  width: 2rem;
  height: 2rem;
  border-radius: 50%;
  box-shadow: var(--shadow-sm);
  flex-shrink: 0;
}

.area-name {
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
}

.area-actions {
  display: flex;
  gap: 0.4rem;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3rem;
  height: 3rem;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: 0.6rem;
  color: var(--color-text-secondary, #f0f);
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary, #f0f);
}

.action-btn.delete:hover {
  background-color: var(--color-danger-light);
  color: var(--color-danger);
}

/* ==================== 编辑模式 ==================== */
.edit-mode {
  display: flex;
  gap: 0.8rem;
  align-items: center;
}

.edit-form {
  display: flex;
  gap: 0.8rem;
  align-items: center;
  flex: 1;
}

.edit-color-wrapper {
  position: relative;
  width: 3.6rem;
  height: 3.6rem;
  flex-shrink: 0;
}

.edit-color-input {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}

.edit-color-preview {
  width: 100%;
  height: 100%;
  border: 1.5px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  pointer-events: none;
}

.edit-name-input {
  flex: 1;
  height: 3.6rem;
  padding: 0 1rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-primary, #f0f);
  border: 1.5px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  outline: none;
  transition: all 0.2s ease;
}

.edit-name-input:focus {
  border-color: var(--color-border-focus, #f0f);
}

.edit-actions {
  display: flex;
  gap: 0.4rem;
}

.edit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3rem;
  height: 3rem;
  padding: 0;
  background-color: transparent;
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.edit-btn.ai {
  background: linear-gradient(135deg, #f5a623 0%, #e94b8b 50%, #9b59b6 100%);
  background-clip: text;
  -webkit-background-clip: text; /* stylelint-disable-line property-no-vendor-prefix */
  -webkit-text-fill-color: transparent;
}

.edit-btn.ai:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.edit-btn.ai:hover:not(:disabled) {
  background-color: var(--color-danger-light, #f0f);
}

.edit-btn.save {
  color: var(--color-success);
}

.edit-btn.save:hover {
  background-color: var(--color-success-light, #f0f);
}

.edit-btn.cancel {
  color: var(--color-text-secondary, #f0f);
}

.edit-btn.cancel:hover {
  background-color: var(--color-background-hover, #f0f);
}
</style>

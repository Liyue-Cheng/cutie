<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAreaStore } from '@/stores/area'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'
import { dialog } from '@/composables/useDialog'

defineEmits(['close'])

const { t } = useI18n()
const areaStore = useAreaStore()

// === 预设颜色定义 (46种: 6 Rose Pine + 40 扩展) ===
interface PresetColor {
  color: string
  name: string
  category: string
}

const PRESET_COLORS: PresetColor[] = [
  // === Rose Pine Moon 原色 (6) ===
  { color: '#eb6f92', name: 'Love', category: 'rose-pine' },
  { color: '#f6c177', name: 'Gold', category: 'rose-pine' },
  { color: '#ea9a97', name: 'Rose', category: 'rose-pine' },
  { color: '#3e8fb0', name: 'Pine', category: 'rose-pine' },
  { color: '#9ccfd8', name: 'Foam', category: 'rose-pine' },
  { color: '#c4a7e7', name: 'Iris', category: 'rose-pine' },
  
  // === 红/粉色系 (8) - 运动、活力 ===
  { color: '#e5484d', name: '鲜红', category: 'red-pink' },
  { color: '#e06c75', name: '柔红', category: 'red-pink' },
  { color: '#d76c6c', name: '暖红', category: 'red-pink' },
  { color: '#be5046', name: '赤陶', category: 'red-pink' },
  { color: '#ff6b6b', name: '珊瑚', category: 'red-pink' },
  { color: '#d4a5a5', name: '裸粉', category: 'red-pink' },
  { color: '#e8b4b4', name: '蜜桃', category: 'red-pink' },
  { color: '#f0b6c2', name: '樱花', category: 'red-pink' },
  
  // === 橙/黄色系 (8) - 运动、活力 ===
  { color: '#ff8c42', name: '活力橙', category: 'orange-yellow' },
  { color: '#e5a06e', name: '杏橙', category: 'orange-yellow' },
  { color: '#d19a66', name: '焦糖', category: 'orange-yellow' },
  { color: '#cc7832', name: '琥珀', category: 'orange-yellow' },
  { color: '#ffb347', name: '芒果', category: 'orange-yellow' },
  { color: '#e2c08d', name: '麦芽', category: 'orange-yellow' },
  { color: '#dbc074', name: '柠黄', category: 'orange-yellow' },
  { color: '#c9a86c', name: '金沙', category: 'orange-yellow' },
  
  // === 绿/青色系 (8) ===
  { color: '#a9b665', name: '橄榄', category: 'green-cyan' },
  { color: '#98c379', name: '草绿', category: 'green-cyan' },
  { color: '#7ec699', name: '薄荷', category: 'green-cyan' },
  { color: '#50c878', name: '翡翠', category: 'green-cyan' },
  { color: '#66cdaa', name: '海绿', category: 'green-cyan' },
  { color: '#56b6c2', name: '青碧', category: 'green-cyan' },
  { color: '#20b2aa', name: '浅海绿', category: 'green-cyan' },
  { color: '#5f9ea0', name: '军蓝', category: 'green-cyan' },
  
  // === 蓝色系 (8) ===
  { color: '#73b8bf', name: '湖蓝', category: 'blue' },
  { color: '#5c8dc6', name: '钴蓝', category: 'blue' },
  { color: '#6495ed', name: '矢车菊', category: 'blue' },
  { color: '#82aaff', name: '淡蓝', category: 'blue' },
  { color: '#4682b4', name: '钢蓝', category: 'blue' },
  { color: '#7986cb', name: '薰衣草蓝', category: 'blue' },
  { color: '#6a9fb5', name: '天青', category: 'blue' },
  { color: '#87ceeb', name: '天蓝', category: 'blue' },
  
  // === 紫/中性色系 (8) ===
  { color: '#b48ead', name: '丁香', category: 'purple-neutral' },
  { color: '#c792ea', name: '兰花紫', category: 'purple-neutral' },
  { color: '#a389d4', name: '紫罗兰', category: 'purple-neutral' },
  { color: '#9370db', name: '中紫', category: 'purple-neutral' },
  { color: '#8a6bbf', name: '葡萄', category: 'purple-neutral' },
  { color: '#a8a8a8', name: '银灰', category: 'purple-neutral' },
  { color: '#8a9ba8', name: '青灰', category: 'purple-neutral' },
  { color: '#708090', name: '石板灰', category: 'purple-neutral' },
]

// 扩展色系分类
const EXTENDED_CATEGORIES = [
  { key: 'red-pink', name: '红/粉色系' },
  { key: 'orange-yellow', name: '橙/黄色系' },
  { key: 'green-cyan', name: '绿/青色系' },
  { key: 'blue', name: '蓝色系' },
  { key: 'purple-neutral', name: '紫/中性色系' },
]

const newAreaName = ref('')
const newAreaColor = ref('#eb6f92')
const editingArea = ref<{ id: string; name: string; color: string } | null>(null)
const isAiColorLoading = ref(false)
const isEditAiColorLoading = ref(false)

// 颜色面板显示状态
const showColorPalette = ref(false)
const showEditColorPalette = ref(false)

// 检查颜色是否被选中
const isPresetColorSelected = computed(() => {
  return PRESET_COLORS.some(p => p.color.toLowerCase() === newAreaColor.value.toLowerCase())
})

const isEditPresetColorSelected = computed(() => {
  if (!editingArea.value) return false
  return PRESET_COLORS.some(p => p.color.toLowerCase() === editingArea.value!.color.toLowerCase())
})

onMounted(async () => {
  await areaStore.fetchAll()
})

function selectPresetColor(color: string) {
  newAreaColor.value = color
  showColorPalette.value = false
}

function selectEditPresetColor(color: string) {
  if (editingArea.value) {
    editingArea.value.color = color
  }
  showEditColorPalette.value = false
}

function toggleColorPalette() {
  showColorPalette.value = !showColorPalette.value
}

function toggleEditColorPalette() {
  showEditColorPalette.value = !showEditColorPalette.value
}

async function handleCreate() {
  if (!newAreaName.value.trim()) return

  try {
    await pipeline.dispatch('area.create', {
      name: newAreaName.value.trim(),
      color: newAreaColor.value,
    })

    newAreaName.value = ''
    newAreaColor.value = '#eb6f92'
    showColorPalette.value = false
  } catch (error) {
    console.error('创建 Area 失败:', error)
    await dialog.alert(t('message.error.createAreaFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
  }
}

async function handleAiSuggestColor() {
  if (!newAreaName.value.trim()) {
    await dialog.alert(t('area.message.enterName'))
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
    await dialog.alert(t('message.error.aiColorFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
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
  showEditColorPalette.value = false
}

function cancelEdit() {
  editingArea.value = null
  showEditColorPalette.value = false
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
    showEditColorPalette.value = false
  } catch (error) {
    console.error('更新 Area 失败:', error)
    await dialog.alert(t('message.error.updateAreaFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
  }
}

async function handleEditAiSuggestColor() {
  if (!editingArea.value?.name.trim()) {
    await dialog.alert(t('area.message.enterName'))
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
    await dialog.alert(t('message.error.aiColorFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
  } finally {
    isEditAiColorLoading.value = false
  }
}

async function handleDelete(id: string) {
  const confirmed = await dialog.confirm(t('confirm.deleteArea'))
  if (confirmed) {
    try {
      await pipeline.dispatch('area.delete', { id })
    } catch (error) {
      console.error('删除 Area 失败:', error)
      await dialog.alert(t('message.error.deleteAreaFailed') + ': ' + (error instanceof Error ? error.message : String(error)))
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
            autocomplete="off"
            @keyup.enter="handleCreate"
          />
          
          <!-- 颜色选择器触发器 -->
          <div 
            class="color-trigger" 
            :style="{ backgroundColor: newAreaColor }"
            @click="toggleColorPalette"
            :title="$t('area.action.selectColor')"
          ></div>
          
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
                  <!-- 编辑颜色选择器触发器 -->
                  <div 
                    class="edit-color-trigger" 
                    :style="{ backgroundColor: editingArea.color }"
                    @click="toggleEditColorPalette"
                  ></div>
                  
                  <input
                    v-model="editingArea.name"
                    type="text"
                    class="edit-name-input"
                    placeholder="Area 名称"
                    autocomplete="off"
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
    
    <!-- 居中颜色选择面板 (新建) -->
    <Transition name="palette-fade">
      <div v-if="showColorPalette" class="color-palette-overlay" @click="showColorPalette = false">
        <div class="color-palette-modal" @click.stop>
          <div class="palette-header">
            <span class="palette-title">选择颜色</span>
            <span class="palette-count">{{ PRESET_COLORS.length }} 种预设</span>
            <button class="palette-close-btn" @click="showColorPalette = false">
              <CuteIcon name="X" :size="18" />
            </button>
          </div>
          
          <!-- 当前选中预览 -->
          <div class="current-color-preview">
            <div class="preview-swatch" :style="{ backgroundColor: newAreaColor }"></div>
            <input 
              v-model="newAreaColor"
              type="text"
              class="color-hex-input"
              placeholder="#000000"
              maxlength="7"
            />
          </div>
          
          <!-- 可滚动内容区域 -->
          <div class="palette-content">
            <!-- Rose Pine 原色区域 -->
            <div class="palette-section">
              <div class="palette-section-title">Rose Pine Moon</div>
              <div class="color-grid rose-pine-grid">
                <button
                  v-for="preset in PRESET_COLORS.filter(p => p.category === 'rose-pine')"
                  :key="preset.color"
                  class="color-swatch"
                  :class="{ selected: newAreaColor.toLowerCase() === preset.color.toLowerCase() }"
                  :style="{ backgroundColor: preset.color }"
                  :title="preset.name"
                  @click="selectPresetColor(preset.color)"
                >
                  <CuteIcon v-if="newAreaColor.toLowerCase() === preset.color.toLowerCase()" name="Check" :size="14" class="check-icon" />
                </button>
              </div>
            </div>
            
            <!-- 扩展颜色区域 - 按色系分组 -->
            <div 
              v-for="category in EXTENDED_CATEGORIES" 
              :key="category.key"
              class="palette-section"
            >
              <div class="palette-section-title">{{ category.name }}</div>
              <div class="color-grid extended-grid">
                <button
                  v-for="preset in PRESET_COLORS.filter(p => p.category === category.key)"
                  :key="preset.color"
                  class="color-swatch"
                  :class="{ selected: newAreaColor.toLowerCase() === preset.color.toLowerCase() }"
                  :style="{ backgroundColor: preset.color }"
                  :title="preset.name"
                  @click="selectPresetColor(preset.color)"
                >
                  <CuteIcon v-if="newAreaColor.toLowerCase() === preset.color.toLowerCase()" name="Check" :size="14" class="check-icon" />
                </button>
              </div>
            </div>
          </div>
          
          <!-- 自定义颜色 -->
          <div class="palette-section custom-section">
            <div class="palette-section-title">自定义颜色</div>
            <div class="custom-color-wrapper">
              <input 
                v-model="newAreaColor" 
                type="color" 
                class="custom-color-input"
              />
              <div 
                class="custom-color-preview"
                :class="{ 'is-custom': !isPresetColorSelected }"
                :style="{ backgroundColor: newAreaColor }"
              >
                <CuteIcon name="Pipette" :size="18" class="pipette-icon" />
                <span class="custom-label">点击选取</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
    
    <!-- 居中颜色选择面板 (编辑) -->
    <Transition name="palette-fade">
      <div v-if="showEditColorPalette && editingArea" class="color-palette-overlay" @click="showEditColorPalette = false">
        <div class="color-palette-modal" @click.stop>
          <div class="palette-header">
            <span class="palette-title">选择颜色</span>
            <span class="palette-count">{{ PRESET_COLORS.length }} 种预设</span>
            <button class="palette-close-btn" @click="showEditColorPalette = false">
              <CuteIcon name="X" :size="18" />
            </button>
          </div>
          
          <!-- 当前选中预览 -->
          <div class="current-color-preview">
            <div class="preview-swatch" :style="{ backgroundColor: editingArea.color }"></div>
            <input 
              v-model="editingArea.color"
              type="text"
              class="color-hex-input"
              placeholder="#000000"
              maxlength="7"
            />
          </div>
          
          <!-- 可滚动内容区域 -->
          <div class="palette-content">
            <!-- Rose Pine 原色区域 -->
            <div class="palette-section">
              <div class="palette-section-title">Rose Pine Moon</div>
              <div class="color-grid rose-pine-grid">
                <button
                  v-for="preset in PRESET_COLORS.filter(p => p.category === 'rose-pine')"
                  :key="preset.color"
                  class="color-swatch"
                  :class="{ selected: editingArea.color.toLowerCase() === preset.color.toLowerCase() }"
                  :style="{ backgroundColor: preset.color }"
                  :title="preset.name"
                  @click="selectEditPresetColor(preset.color)"
                >
                  <CuteIcon v-if="editingArea.color.toLowerCase() === preset.color.toLowerCase()" name="Check" :size="14" class="check-icon" />
                </button>
              </div>
            </div>
            
            <!-- 扩展颜色区域 - 按色系分组 -->
            <div 
              v-for="category in EXTENDED_CATEGORIES" 
              :key="category.key"
              class="palette-section"
            >
              <div class="palette-section-title">{{ category.name }}</div>
              <div class="color-grid extended-grid">
                <button
                  v-for="preset in PRESET_COLORS.filter(p => p.category === category.key)"
                  :key="preset.color"
                  class="color-swatch"
                  :class="{ selected: editingArea.color.toLowerCase() === preset.color.toLowerCase() }"
                  :style="{ backgroundColor: preset.color }"
                  :title="preset.name"
                  @click="selectEditPresetColor(preset.color)"
                >
                  <CuteIcon v-if="editingArea.color.toLowerCase() === preset.color.toLowerCase()" name="Check" :size="14" class="check-icon" />
                </button>
              </div>
            </div>
          </div>
          
          <!-- 自定义颜色 -->
          <div class="palette-section custom-section">
            <div class="palette-section-title">自定义颜色</div>
            <div class="custom-color-wrapper">
              <input 
                v-model="editingArea.color" 
                type="color" 
                class="custom-color-input"
              />
              <div 
                class="custom-color-preview"
                :class="{ 'is-custom': !isEditPresetColorSelected }"
                :style="{ backgroundColor: editingArea.color }"
              >
                <CuteIcon name="Pipette" :size="18" class="pipette-icon" />
                <span class="custom-label">点击选取</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
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
  background-color: var(--color-background-content, #f0f);
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
  line-height: 1.4;
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
  line-height: 1.4;
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

/* ==================== 颜色选择器触发器 ==================== */
.color-trigger {
  width: 4rem;
  height: 4rem;
  border: 2px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.color-trigger:hover {
  transform: scale(1.05);
  box-shadow: var(--shadow-md);
  border-color: var(--color-border-strong, #f0f);
}

/* ==================== 颜色调色板弹出层 ==================== */
.color-palette-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 2000;
}

.color-palette-modal {
  width: 46rem;
  max-width: 90vw;
  max-height: 85vh;
  background-color: var(--color-background-content, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-xl);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.palette-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.4rem 1.6rem;
  background-color: var(--color-background-secondary, #f0f);
  border-bottom: 1px solid var(--color-border-default, #f0f);
}

.palette-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

.palette-close-btn {
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

.palette-close-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.palette-count {
  font-size: 1.2rem;
  color: var(--color-text-tertiary, #f0f);
  margin-left: auto;
  margin-right: 1rem;
  line-height: 1.4;
}

/* ==================== 可滚动内容区域 ==================== */
.palette-content {
  flex: 1;
  overflow-y: auto;
  max-height: 45vh;
}

/* ==================== 当前颜色预览 ==================== */
.current-color-preview {
  display: flex;
  gap: 1rem;
  align-items: center;
  padding: 1.2rem 1.6rem;
  background-color: var(--color-background-secondary, #f0f);
  border-bottom: 1px solid var(--color-border-default, #f0f);
}

.preview-swatch {
  width: 4rem;
  height: 4rem;
  border: 2px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  flex-shrink: 0;
}

.color-hex-input {
  flex: 1;
  height: 4rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-family: 'JetBrains Mono', monospace;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-primary, #f0f);
  border: 1.5px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  outline: none;
  text-transform: uppercase;
}

.color-hex-input:focus {
  border-color: var(--color-border-focus, #f0f);
}

/* ==================== 调色板区域 ==================== */
.palette-section {
  padding: 1.2rem 1.6rem;
}

.palette-section:not(:last-child) {
  border-bottom: 1px solid var(--color-border-light, #f0f);
}

.palette-section-title {
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary, #f0f);
  margin-bottom: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  line-height: 1.4;
}

.color-grid {
  display: grid;
  gap: 0.8rem;
}

.rose-pine-grid {
  grid-template-columns: repeat(6, 1fr);
}

.extended-grid {
  grid-template-columns: repeat(8, 1fr);
}

.color-swatch {
  width: 100%;
  aspect-ratio: 1;
  min-height: 3.2rem;
  border: 2px solid transparent;
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  background: none;
}

.color-swatch:hover {
  transform: scale(1.12);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
  z-index: 1;
}

.color-swatch.selected {
  border-color: var(--color-text-primary, #f0f);
  box-shadow: 0 0 0 3px var(--color-background-content, #f0f), 0 0 0 5px currentColor;
  transform: scale(1.05);
}

.check-icon {
  color: rgba(255, 255, 255, 0.95);
  filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.6));
}

/* ==================== 自定义颜色 ==================== */
.custom-section {
  background-color: var(--color-background-secondary, #f0f);
}

.custom-color-wrapper {
  position: relative;
  width: 100%;
  height: 5rem;
}

.custom-color-input {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: pointer;
}

.custom-color-preview {
  width: 100%;
  height: 100%;
  border: 2px dashed var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  cursor: pointer;
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.8rem;
  transition: all 0.2s ease;
}

.custom-color-preview.is-custom {
  border-style: solid;
  border-color: var(--color-text-primary, #f0f);
}

.pipette-icon {
  color: rgba(255, 255, 255, 0.9);
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.5));
}

.custom-label {
  font-size: 1.3rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.5));
}

/* ==================== 调色板动画 ==================== */
.palette-fade-enter-active,
.palette-fade-leave-active {
  transition: all 0.25s ease;
}

.palette-fade-enter-from,
.palette-fade-leave-to {
  opacity: 0;
}

.palette-fade-enter-from .color-palette-modal,
.palette-fade-leave-to .color-palette-modal {
  transform: scale(0.95);
}

/* ==================== AI 按钮 ==================== */
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
  background-color: var(--color-background-primary, #f0f);
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
  color: var(--color-text-tertiary, #f0f);
  padding: 0.4rem 1rem;
  background-color: var(--color-background-secondary, #f0f);
  border-radius: 1.2rem;
  line-height: 1.4;
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
  color: var(--color-text-tertiary, #f0f);
  opacity: 0.5;
  margin-bottom: 1.6rem;
}

.empty-text {
  font-size: 1.6rem;
  font-weight: 500;
  color: var(--color-text-secondary, #f0f);
  margin: 0 0 0.8rem;
  line-height: 1.4;
}

.empty-hint {
  font-size: 1.4rem;
  color: var(--color-text-tertiary, #f0f);
  margin: 0;
  line-height: 1.4;
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
  line-height: 1.4;
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

.edit-color-trigger {
  width: 3.6rem;
  height: 3.6rem;
  border: 2px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.edit-color-trigger:hover {
  transform: scale(1.05);
  border-color: var(--color-border-strong, #f0f);
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

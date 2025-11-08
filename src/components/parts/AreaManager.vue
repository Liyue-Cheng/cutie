<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAreaStore } from '@/stores/area'
import CuteIcon from '@/components/parts/CuteIcon.vue'

defineEmits(['close'])

const areaStore = useAreaStore()

const newAreaName = ref('')
const newAreaColor = ref('#4A90E2')
const editingArea = ref<{ id: string; name: string; color: string } | null>(null)

onMounted(async () => {
  await areaStore.fetchAreas()
})

async function handleCreate() {
  if (!newAreaName.value.trim()) return

  await areaStore.createArea({
    name: newAreaName.value.trim(),
    color: newAreaColor.value,
  })

  newAreaName.value = ''
  newAreaColor.value = '#4A90E2'
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

  await areaStore.updateArea(editingArea.value.id, {
    name: editingArea.value.name,
    color: editingArea.value.color,
  })

  editingArea.value = null
}

async function handleDelete(id: string) {
  if (confirm('确定要删除这个 Area 吗？这将影响所有关联的任务。')) {
    await areaStore.deleteArea(id)
  }
}
</script>

<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="manager-container" @click.stop>
      <!-- 头部 -->
      <div class="manager-header">
        <h2 class="manager-title">Area 管理器</h2>
        <button class="close-btn" @click="$emit('close')" title="关闭">
          <CuteIcon name="X" :size="20" />
        </button>
      </div>

      <!-- 创建新 Area 区域 -->
      <div class="create-section">
        <h3 class="section-title">创建新 Area</h3>
        <div class="create-form">
          <input
            v-model="newAreaName"
            type="text"
            placeholder="输入 Area 名称..."
            class="name-input"
            @keyup.enter="handleCreate"
          />
          <div class="color-picker-wrapper">
            <input v-model="newAreaColor" type="color" class="color-input" title="选择颜色" />
            <div class="color-preview" :style="{ backgroundColor: newAreaColor }"></div>
          </div>
          <button
            class="add-btn"
            @click="handleCreate"
            :disabled="!newAreaName.trim()"
            title="添加 Area"
          >
            <CuteIcon name="Plus" :size="18" />
            <span>添加</span>
          </button>
        </div>
      </div>

      <!-- Area 列表区域 -->
      <div class="areas-section">
        <div class="section-header">
          <h3 class="section-title">所有 Areas</h3>
          <span class="area-count">{{ areaStore.allAreas.length }} 个</span>
        </div>

        <div class="areas-list">
          <div v-if="areaStore.allAreas.length === 0" class="empty-state">
            <CuteIcon name="Tag" :size="48" class="empty-icon" />
            <p class="empty-text">还没有创建任何 Area</p>
            <p class="empty-hint">Area 可以帮助你组织和分类任务</p>
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
                  <button class="edit-btn save" @click="saveEdit" title="保存">
                    <CuteIcon name="Check" :size="16" />
                  </button>
                  <button class="edit-btn cancel" @click="cancelEdit" title="取消">
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
                  <button class="action-btn edit" @click="startEdit(area)" title="编辑">
                    <CuteIcon name="Pencil" :size="16" />
                  </button>
                  <button class="action-btn delete" @click="handleDelete(area.id)" title="删除">
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
  background-color: rgb(0 0 0 / 50%);
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
  background-color: var(--color-background-primary, #faf4ed);
  border-radius: 1.2rem;
  box-shadow:
    0 2rem 6rem rgb(0 0 0 / 15%),
    0 0.8rem 1.6rem rgb(0 0 0 / 10%);
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
  background-color: var(--color-background-secondary, #fffaf3);
  border-bottom: 1px solid var(--color-border-default, rgb(0 0 0 / 10%));
}

.manager-title {
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
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  color: var(--color-text-primary, #575279);
}

/* ==================== 创建区域 ==================== */
.create-section {
  padding: 2rem 2.4rem;
  background-color: var(--color-background-secondary, #fffaf3);
  border-bottom: 1px solid var(--color-border-default, rgb(0 0 0 / 10%));
}

.section-title {
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-secondary, #797593);
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
  color: var(--color-text-primary, #575279);
  background-color: var(--color-background-primary, #faf4ed);
  border: 1.5px solid var(--color-border-default, rgb(0 0 0 / 10%));
  border-radius: 0.8rem;
  outline: none;
  transition: all 0.2s ease;
}

.name-input::placeholder {
  color: var(--color-text-tertiary, #9893a5);
}

.name-input:focus {
  border-color: var(--color-primary, #d7827e);
  box-shadow: 0 0 0 3px rgb(215 130 126 / 10%);
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
  border: 1.5px solid var(--color-border-default, rgb(0 0 0 / 10%));
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
  pointer-events: none;
}

.color-picker-wrapper:hover .color-preview {
  transform: scale(1.05);
  box-shadow: 0 0.4rem 1.2rem rgb(0 0 0 / 10%);
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
  color: white;
  background-color: var(--color-primary, #d7827e);
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
  background-color: var(--rose-pine-rose, #d7827e);
  transform: translateY(-1px);
  box-shadow: 0 0.4rem 1.2rem rgb(215 130 126 / 30%);
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
  color: var(--color-text-tertiary, #9893a5);
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
  background-color: var(--color-background-secondary, #fffaf3);
  border: 1.5px solid var(--color-border-default, rgb(0 0 0 / 10%));
  border-radius: 0.8rem;
  padding: 1.2rem 1.4rem;
  transition: all 0.2s ease;
}

.area-card:hover {
  border-color: rgb(0 0 0 / 15%);
  box-shadow: 0 0.2rem 0.8rem rgb(0 0 0 / 5%);
}

.area-card.editing {
  border-color: var(--color-primary, #d7827e);
  box-shadow: 0 0 0 3px rgb(215 130 126 / 10%);
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
  border: 2px solid rgb(255 255 255 / 50%);
  box-shadow: 0 0.2rem 0.8rem rgb(0 0 0 / 15%);
  flex-shrink: 0;
}

.area-name {
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary, #575279);
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
  color: var(--color-text-secondary, #797593);
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  color: var(--color-text-primary, #575279);
}

.action-btn.delete:hover {
  background-color: rgb(239 68 68 / 10%);
  color: #ef4444;
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
  border: 1.5px solid var(--color-border-default, rgb(0 0 0 / 10%));
  border-radius: 0.6rem;
  cursor: pointer;
  pointer-events: none;
}

.edit-name-input {
  flex: 1;
  height: 3.6rem;
  padding: 0 1rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #575279);
  background-color: var(--color-background-primary, #faf4ed);
  border: 1.5px solid var(--color-border-default, rgb(0 0 0 / 10%));
  border-radius: 0.6rem;
  outline: none;
  transition: all 0.2s ease;
}

.edit-name-input:focus {
  border-color: var(--color-primary, #d7827e);
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

.edit-btn.save {
  color: #22c55e;
}

.edit-btn.save:hover {
  background-color: rgb(34 197 94 / 10%);
}

.edit-btn.cancel {
  color: var(--color-text-secondary, #797593);
}

.edit-btn.cancel:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
}
</style>

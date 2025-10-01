<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAreaStore } from '@/stores/area'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteButton from '@/components/parts/CuteButton.vue'
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

function startEdit(area: typeof areaStore.allAreas[0]) {
  editingArea.value = {
    id: area.id,
    name: area.name,
    color: area.color,
  }
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
  if (confirm('确定要删除这个 Area 吗？')) {
    await areaStore.deleteArea(id)
  }
}
</script>

<template>
  <div class="modal-overlay" @click="$emit('close')">
    <CuteCard class="manager-card" @click.stop>
      <div class="header">
        <h2 class="title">Area 管理器</h2>
        <CuteButton class="close-btn" @click="$emit('close')">
          <CuteIcon name="X" :size="20" />
        </CuteButton>
      </div>

      <div class="separator"></div>

      <!-- 创建新 Area -->
      <div class="create-section">
        <h3 class="section-title">创建新 Area</h3>
        <div class="create-form">
          <input
            v-model="newAreaName"
            type="text"
            placeholder="Area 名称"
            class="name-input"
            @keyup.enter="handleCreate"
          />
          <input
            v-model="newAreaColor"
            type="color"
            class="color-input"
            title="选择颜色"
          />
          <CuteButton @click="handleCreate">添加</CuteButton>
        </div>
      </div>

      <div class="separator"></div>

      <!-- Area 列表 -->
      <div class="areas-list">
        <h3 class="section-title">所有 Areas</h3>
        <div class="areas-grid">
          <div v-for="area in areaStore.allAreas" :key="area.id" class="area-item">
            <div v-if="editingArea?.id === area.id" class="edit-mode">
              <input
                v-model="editingArea.name"
                type="text"
                class="edit-name-input"
              />
              <input
                v-model="editingArea.color"
                type="color"
                class="edit-color-input"
              />
              <CuteButton size="small" @click="saveEdit">保存</CuteButton>
              <CuteButton size="small" @click="editingArea = null">取消</CuteButton>
            </div>
            <div v-else class="view-mode">
              <div class="area-info">
                <div class="color-dot" :style="{ backgroundColor: area.color }"></div>
                <span class="area-name">{{ area.name }}</span>
              </div>
              <div class="area-actions">
                <button class="icon-btn" @click="startEdit(area)">
                  <CuteIcon name="Edit" :size="16" />
                </button>
                <button class="icon-btn delete" @click="handleDelete(area.id)">
                  <CuteIcon name="Trash2" :size="16" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </CuteCard>
  </div>
</template>

<style scoped>
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
}

.manager-card {
  width: 70rem;
  max-height: 80vh;
  padding: 2.5rem;
  overflow-y: auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.title {
  font-size: 2.4rem;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  padding: 0.5rem;
  background: transparent;
  border: none;
}

.separator {
  height: 1px;
  background-color: var(--color-separator);
  margin: 2rem 0;
}

.section-title {
  font-size: 1.6rem;
  font-weight: 500;
  margin-bottom: 1rem;
  color: var(--color-text-secondary);
}

.create-form {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.name-input {
  flex: 1;
  padding: 0.8rem;
  font-size: 1.5rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
}

.color-input {
  width: 5rem;
  height: 3.8rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  cursor: pointer;
}

.areas-grid {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.area-item {
  padding: 1rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background-color: var(--color-background-secondary);
}

.view-mode {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.area-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.color-dot {
  width: 1.6rem;
  height: 1.6rem;
  border-radius: 50%;
}

.area-name {
  font-size: 1.5rem;
  font-weight: 500;
}

.area-actions {
  display: flex;
  gap: 0.5rem;
}

.icon-btn {
  padding: 0.5rem;
  background: transparent;
  border: none;
  cursor: pointer;
  border-radius: 4px;
  color: var(--color-text-secondary);
}

.icon-btn:hover {
  background-color: rgb(0 0 0 / 5%);
  color: var(--color-text-primary);
}

.icon-btn.delete:hover {
  color: var(--color-error, #ef4444);
}

.edit-mode {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.edit-name-input {
  flex: 1;
  padding: 0.6rem;
  font-size: 1.4rem;
  border: 1px solid var(--color-border-default);
  border-radius: 4px;
}

.edit-color-input {
  width: 4rem;
  height: 3rem;
  border: 1px solid var(--color-border-default);
  border-radius: 4px;
  cursor: pointer;
}
</style>


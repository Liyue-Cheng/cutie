<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useAreaStore } from '@/stores/area'
import { useTaskStore } from '@/stores/task'
import SimpleKanbanColumn from '@/components/parts/kanban/SimpleKanbanColumn.vue'
import type { TaskCard } from '@/types/dtos'

const areaStore = useAreaStore()
const taskStore = useTaskStore()

onMounted(async () => {
  await Promise.all([areaStore.fetchAreas(), taskStore.fetchAllTasks()])
})

// 为每个 Area 创建看板列
const areaColumns = computed(() => {
  return areaStore.allAreas.map((area) => ({
    area,
    tasks: taskStore.getTasksByArea(area.id),
  }))
})
</script>

<template>
  <div class="area-test-view">
    <h1 class="page-title">Area 测试页面</h1>
    <div class="area-kanbans">
      <SimpleKanbanColumn
        v-for="column in areaColumns"
        :key="column.area.id"
        :title="column.area.name"
        :subtitle="`颜色: ${column.area.color}`"
        :tasks="column.tasks"
        @open-editor="() => {}"
      />
    </div>
  </div>
</template>

<style scoped>
.area-test-view {
  padding: 2rem;
  height: 100vh;
  overflow: auto;
}

.page-title {
  font-size: 2.4rem;
  margin-bottom: 2rem;
  color: var(--color-text-primary);
}

.area-kanbans {
  display: flex;
  gap: 1rem;
  overflow-x: auto;
  padding-bottom: 2rem;
}
</style>


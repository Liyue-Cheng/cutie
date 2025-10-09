<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useAreaStore } from '@/stores/area'
import { useViewOperations } from '@/composables/useViewOperations'
import SimpleKanbanColumn from '@/components/parts/kanban/SimpleKanbanColumn.vue'

const areaStore = useAreaStore()
const viewOps = useViewOperations()

onMounted(async () => {
  // ✅ 加载区域和任务数据
  await Promise.all([areaStore.fetchAreas(), viewOps.loadAllTasks()])
})

// 🆕 为每个 Area 创建看板列（使用 viewKey 模式）
const areaColumns = computed(() => {
  return areaStore.allAreas.map((area) => {
    return {
      area,
      viewKey: `area::${area.id}`, // ✅ 遵循 VIEW_CONTEXT_KEY_SPEC 规范
    }
  })
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
        :view-key="column.viewKey"
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

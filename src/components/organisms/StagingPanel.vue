<template>
  <div class="staging-panel">
    <!-- 分类列表 -->
    <div class="staging-list-section">
      <StagingListPanel
        :selected-category="selectedCategoryId"
        @select-category="handleSelectCategory"
      />
    </div>

    <!-- 任务详情 -->
    <div class="staging-detail-section">
      <StagingDetailPanel :category-id="selectedCategoryId" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import { pipeline } from '@/cpu'
import StagingListPanel from '@/components/organisms/StagingListPanel.vue'
import StagingDetailPanel from '@/components/organisms/StagingDetailPanel.vue'

const taskStore = useTaskStore()
const areaStore = useAreaStore()

// 当前选中的分类 ID
// undefined = 未选择任何分类
// 'recent-carryover' = 最近结转
// 'no-area' = 无区域
// 其他 = 区域 ID
const selectedCategoryId = ref<string | undefined>(undefined)

// 选择分类
const handleSelectCategory = (id: string | null) => {
  selectedCategoryId.value = id ?? undefined
}

// 初始化时加载数据
onMounted(async () => {
  console.log('🚀 StagingPanel mounted')
  try {
    // 加载任务
    console.log('📥 Loading tasks...')
    await taskStore.fetchAllIncompleteTasks_DMA()
    console.log('✅ Tasks loaded:', taskStore.allTasks.length)

    // 加载区域
    console.log('📥 Loading areas...')
    await pipeline.dispatch('area.fetch_all', {})
    console.log('✅ Areas loaded:', areaStore.allAreas.length)

    // 默认选择第一个有任务的分类
    const stagingTasks = taskStore.stagingTasks
    const recentCarryoverTasks = taskStore.getTasksByViewKey_Mux('misc::staging::recent-carryover')

    if (recentCarryoverTasks.length > 0) {
      // 有最近结转任务，选择它
      selectedCategoryId.value = 'recent-carryover'
    } else if (stagingTasks.some(task => !task.area_id)) {
      // 有无区域任务，选择它
      selectedCategoryId.value = 'no-area'
    } else {
      // 选择第一个有任务的区域
      const areaWithTasks = stagingTasks.find(task => task.area_id)
      if (areaWithTasks?.area_id) {
        selectedCategoryId.value = areaWithTasks.area_id
      } else {
        // 没有任何 staging 任务，默认显示无区域
        selectedCategoryId.value = 'no-area'
      }
    }
    console.log('📌 Selected category:', selectedCategoryId.value)
  } catch (error) {
    console.error('❌ Failed to load data:', error)
    selectedCategoryId.value = 'no-area'
  }
})
</script>

<style scoped>
.staging-panel {
  display: flex;
  width: 100%;
  height: 100%;
  background: var(--color-background-content, #f0f);
  gap: 1px;
}

.staging-list-section {
  width: 32rem;
  flex-shrink: 0;
  height: 100%;
  background: var(--color-background-content, #f0f);
  border-right: 1px solid var(--color-border-adaptive-light-normal-dark-soft, #f0f);
}

.staging-detail-section {
  flex: 1;
  height: 100%;
  overflow: hidden;
}
</style>

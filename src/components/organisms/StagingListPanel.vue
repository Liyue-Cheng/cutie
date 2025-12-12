<template>
  <TwoRowLayout class="staging-list-panel">
    <template #top>
      <!-- 控制栏 -->
      <div class="control-bar">
        <h2 class="title">{{ $t('nav.staging') }}</h2>
      </div>
    </template>

    <template #bottom>
      <!-- 分类列表 -->
      <div class="category-list">
        <!-- 最近结转（仅在有任务时显示） -->
        <div
          v-if="recentCarryoverCount > 0"
          class="category-card"
          :class="{ active: selectedCategory === 'recent-carryover' }"
          @click="emit('select-category', 'recent-carryover')"
        >
          <div class="category-row">
            <div class="category-left">
              <div class="category-icon">
                <CuteIcon name="History" :size="16" />
              </div>
              <div class="category-name">{{ $t('task.label.recentCarryover') }}</div>
            </div>
            <div class="category-right">
              <span class="task-count">{{ recentCarryoverCount }}</span>
            </div>
          </div>
        </div>

        <!-- 无区域 -->
        <div
          class="category-card"
          :class="{ active: selectedCategory === 'no-area' }"
          @click="emit('select-category', 'no-area')"
        >
          <div class="category-row">
            <div class="category-left">
              <div class="category-icon">
                <CuteIcon name="Inbox" :size="16" />
              </div>
              <div class="category-name">{{ $t('task.label.noArea') }}</div>
            </div>
            <div class="category-right">
              <span class="task-count">{{ noAreaCount }}</span>
            </div>
          </div>
        </div>

        <!-- 区域列表 -->
        <div
          v-for="area in allAreas"
          :key="area.id"
          class="category-card"
          :class="{ active: selectedCategory === area.id }"
          @click="emit('select-category', area.id)"
        >
          <div class="category-row">
            <div class="category-left">
              <div class="category-icon">
                <span
                  class="area-dot"
                  :style="{ backgroundColor: area.color || 'var(--color-text-tertiary)' }"
                ></span>
              </div>
              <div class="category-name">{{ area.name }}</div>
            </div>
            <div class="category-right">
              <span class="task-count">{{ area.taskCount }}</span>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div
          v-if="allAreas.length === 0 && noAreaCount === 0 && recentCarryoverCount === 0"
          class="empty-state"
        >
          <p>{{ $t('task.empty.noStagingTasks') }}</p>
        </div>
      </div>
    </template>
  </TwoRowLayout>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'

interface Props {
  selectedCategory?: string | null
}

defineProps<Props>()

const emit = defineEmits<{
  'select-category': [id: string | null]
}>()

const taskStore = useTaskStore()
const areaStore = useAreaStore()

// 获取 staging 任务
const stagingTasks = computed(() => taskStore.stagingTasks)

// 最近结转任务数量
const recentCarryoverCount = computed(() => {
  return taskStore.getTasksByViewKey_Mux('misc::staging::recent-carryover').length
})

// 无区域任务数量
const noAreaCount = computed(() => {
  return stagingTasks.value.filter((task) => !task.area_id).length
})

// 所有区域列表（带任务数量统计）
const allAreas = computed(() => {
  // 统计每个区域的 staging 任务数量
  const areaTaskMap = new Map<string, number>()
  for (const task of stagingTasks.value) {
    if (task.area_id) {
      const count = areaTaskMap.get(task.area_id) || 0
      areaTaskMap.set(task.area_id, count + 1)
    }
  }

  // 获取所有区域并附加任务数量
  const result: Array<{ id: string; name: string; color: string | null; taskCount: number }> = []
  for (const area of areaStore.allAreas) {
    result.push({
      id: area.id,
      name: area.name,
      color: area.color,
      taskCount: areaTaskMap.get(area.id) || 0,
    })
  }

  // 按名称排序
  return result.sort((a, b) => a.name.localeCompare(b.name))
})
</script>

<style scoped>
.staging-list-panel {
  background: var(--color-background-content, #f0f);
}

.control-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 1.2rem 0 2.4rem;
}

.title {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  margin: 0;
  line-height: 1.4;
}

.category-list {
  padding: 1.2rem;
}

.category-card {
  position: relative;
  display: flex;
  align-items: center;
  padding: 1.2rem;
  margin-bottom: 0.8rem;
  background: var(--color-background-content, #f0f);
  border: 2px solid transparent;
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s;
}

.category-card:hover {
  background: var(--color-background-hover, #f0f);
}

.category-card.active {
  background: var(--color-background-active, #f0f);
}

.category-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: 1.2rem;
}

.category-left {
  display: flex;
  align-items: center;
  gap: 1.2rem;
  min-width: 0;
}

.category-right {
  display: flex;
  align-items: center;
  gap: 1rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-secondary, #f0f);
  white-space: nowrap;
}

.category-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 2.4rem;
  color: var(--color-text-primary, #f0f);
  flex-shrink: 0;
}

.category-name {
  font-size: 1.6rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
}

.task-count {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
}

/* 区域彩色圆点 */
.area-dot {
  width: 1.2rem;
  height: 1.2rem;
  border-radius: 50%;
  flex-shrink: 0;
}

.empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--color-text-secondary, #f0f);
}

.empty-state p {
  margin: 0.8rem 0;
}
</style>

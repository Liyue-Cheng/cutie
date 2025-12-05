<template>
  <div class="staging-list">
    <TaskList
      ref="taskListRef"
      :title="$t('task.label.scheduled')"
      view-key="misc::staging"
      :show-add-input="true"
      :default-collapsed="false"
      :collapsible="false"
      :hide-header="props.hideHeader"
      fill-remaining-space
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import TaskList from './TaskList.vue'

// Props
interface Props {
  hideHeader?: boolean // 是否隐藏标题栏（用于外部自定义标题）
}

const props = withDefaults(defineProps<Props>(), {
  hideHeader: false,
})

// TaskList 引用
const taskListRef = ref<InstanceType<typeof TaskList> | null>(null)

// 暴露给父组件
defineExpose({
  taskCount: computed(() => taskListRef.value?.taskCount ?? 0),
})
</script>

<style scoped>
.staging-list {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}
</style>

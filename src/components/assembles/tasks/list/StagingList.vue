<template>
  <div class="staging-list">
    <TaskList
      :title="$t('task.label.scheduled')"
      view-key="misc::staging"
      :show-add-input="true"
      :default-collapsed="false"
      :collapsible="false"
      fill-remaining-space
    />
  </div>
</template>

<script setup lang="ts">
import TaskList from './TaskList.vue'
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

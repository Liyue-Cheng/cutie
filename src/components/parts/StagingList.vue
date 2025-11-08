<template>
  <div class="staging-list">
    <TaskBar
      title="待安排"
      view-key="misc::staging"
      :show-add-input="true"
      :default-collapsed="false"
      fill-remaining-space
    />
  </div>
</template>

<script setup lang="ts">
import TaskBar from './TaskBar.vue'
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

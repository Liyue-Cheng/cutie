<template>
  <div class="upcoming-list">
    <TaskBar
      title="即将到期"
      view-key="misc::upcoming"
      :show-add-input="false"
      :default-collapsed="false"
      fill-remaining-space
    />
  </div>
</template>

<script setup lang="ts">
import TaskBar from './TaskBar.vue'
</script>

<style scoped>
.upcoming-list {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}
</style>

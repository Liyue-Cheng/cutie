<template>
  <div class="home-upcoming-panel">
    <UpcomingVerticalPanel />
  </div>
</template>

<script setup lang="ts">
import UpcomingVerticalPanel from '@/components/organisms/UpcomingVerticalPanel.vue'
</script>

<style scoped>
.home-upcoming-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>

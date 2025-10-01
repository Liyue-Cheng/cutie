<template>
  <div class="staging-view-dummy">
    <div class="dummy-content">
      <h1>🚧 Staging View</h1>
      <p class="subtitle">此视图已废弃</p>
      <p class="description">请使用 HomeView 查看 Staging 任务</p>
      <router-link to="/" class="go-home-btn">前往 HomeView</router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
// Dummy component - StagingView has been deprecated
// Use HomeView instead for viewing staging tasks
</script>

<style scoped>
.staging-view-dummy {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background-color: var(--color-background-content);
}

.dummy-content {
  text-align: center;
  padding: 4rem;
  max-width: 50rem;
}

.dummy-content h1 {
  font-size: 4rem;
  color: var(--color-text-primary);
  margin-bottom: 1rem;
}

.subtitle {
  font-size: 1.8rem;
  color: var(--color-text-secondary);
  margin-bottom: 1rem;
}

.description {
  font-size: 1.4rem;
  color: var(--color-text-secondary);
  margin-bottom: 3rem;
}

.go-home-btn {
  display: inline-block;
  padding: 1.2rem 2.4rem;
  background-color: var(--color-button-primary, #4a90e2);
  color: white;
  text-decoration: none;
  border-radius: 0.8rem;
  font-size: 1.6rem;
  font-weight: 500;
  transition: background-color 0.2s;
}

.go-home-btn:hover {
  background-color: var(--color-button-primary-hover, #357abd);
}
</style>

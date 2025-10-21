<template>
  <div class="home-view">
    <div class="home-content">
      <!-- 全新原创设计从这里开始 -->
      <div class="placeholder">
        <h1>Welcome to Cutie</h1>
        <p>全新原创设计</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRegisterStore } from '@/stores/register'
import { logger, LogTags } from '@/infra/logging/logger'

const registerStore = useRegisterStore()

// 初始化
onMounted(() => {
  logger.info(LogTags.VIEW_HOME, 'Initializing brand new home view...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'home')
})
</script>

<style scoped>
.home-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--color-background-primary);
}

.home-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.placeholder {
  text-align: center;
  color: var(--color-text-secondary);
}

.placeholder h1 {
  font-size: 3rem;
  font-weight: 600;
  margin-bottom: 1rem;
  color: var(--color-text-primary);
}

.placeholder p {
  font-size: 1.6rem;
}
</style>

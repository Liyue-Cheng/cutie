<script setup lang="ts">
import { ref, computed } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'

// 当前时间
const currentTime = ref(new Date())

// 问候语
const greeting = computed(() => {
  const hour = currentTime.value.getHours()
  if (hour < 6) return '夜深了'
  if (hour < 12) return '早上好'
  if (hour < 18) return '下午好'
  return '晚上好'
})

// 每分钟更新时间
setInterval(() => {
  currentTime.value = new Date()
}, 60000)
</script>

<template>
  <div class="new-home">
    <div class="center-content">
      <div class="greeting-section">
        <h1 class="greeting">{{ greeting }}</h1>
        <p class="subtitle">全新的开始</p>
      </div>

      <div class="placeholder">
        <CuteIcon name="Sparkles" :size="64" />
        <p class="placeholder-text">新的 Home 页面正在构建中...</p>
        <p class="placeholder-hint">尝试访问左侧的 "Sunsama Legacy" 查看原有布局</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.new-home {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-background-primary);
  padding: 3rem;
}

.center-content {
  text-align: center;
  max-width: 60rem;
}

.greeting-section {
  margin-bottom: 6rem;
}

.greeting {
  font-size: 4.8rem;
  font-weight: 700;
  margin: 0;
  color: transparent;
  background: linear-gradient(
    135deg,
    var(--rose-pine-foam, #56949f),
    var(--rose-pine-iris, #907aa9)
  );
  background-clip: text;
}

.subtitle {
  font-size: 1.8rem;
  margin: 1rem 0 0;
  color: var(--color-text-secondary);
}

.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2rem;
  padding: 4rem;
  background: var(--color-background-secondary);
  border: 2px dashed var(--color-border-default);
  border-radius: 1.6rem;
  color: var(--rose-pine-foam, #56949f);
}

.placeholder-text {
  font-size: 2rem;
  font-weight: 500;
  margin: 0;
  color: var(--color-text-primary);
}

.placeholder-hint {
  font-size: 1.4rem;
  margin: 0;
  color: var(--color-text-tertiary);
}
</style>

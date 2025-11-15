<template>
  <CutePane class="ai-chat-pane">
    <div class="content-wrapper">
      <h1>🤖 AI 对话模式</h1>
      <p>这是一个全屏的、没有标准布局的页面。</p>
    </div>
  </CutePane>
</template>
<script setup lang="ts">
import CutePane from '@/components/alias/CutePane.vue'
</script>
<style scoped>
.ai-chat-pane {
  background-color: var(--color-background-primary);
  color: var(--color-text-primary);
}

.content-wrapper {
  padding: 2rem;
}
</style>

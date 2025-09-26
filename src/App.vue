<template>
  <n-message-provider>
    <n-dialog-provider>
      <router-view />
      <ContextMenuHost />
      <!-- Vue-Draxis 拖放渲染器 -->
      <DragRenderer />
    </n-dialog-provider>
  </n-message-provider>
</template>

<script setup lang="ts">
import { NMessageProvider, NDialogProvider } from 'naive-ui'
import ContextMenuHost from '@/components/functional/ContextMenuHost.vue'
import DragRenderer from '@/components/DragRenderer.vue'
</script>

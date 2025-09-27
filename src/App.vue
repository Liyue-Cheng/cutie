<template>
  <n-message-provider>
    <n-dialog-provider>
      <router-view />
      <ContextMenuHost />
    </n-dialog-provider>
  </n-message-provider>
</template>

<script setup lang="ts">
import { NMessageProvider, NDialogProvider } from 'naive-ui'
import ContextMenuHost from '@/components/functional/ContextMenuHost.vue'
</script>

<template>
  <n-config-provider :preflight-style-disabled="true">
    <n-global-style />
    <n-message-provider>
      <n-dialog-provider>
        <router-view />
        <ContextMenuHost />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { NMessageProvider, NDialogProvider, NConfigProvider, NGlobalStyle } from 'naive-ui'
import ContextMenuHost from '@/components/functional/ContextMenuHost.vue'
</script>

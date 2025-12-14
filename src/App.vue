<template>
  <router-view />
  <ContextMenuHost />
  <CuteDialog />
  <TimeBlockRecurrenceEditDialog
    v-if="recurrenceDialogContext"
    :recurrence-id="recurrenceDialogContext.recurrenceId"
    @close="uiStore.closeTimeBlockRecurrenceEditDialog()"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import ContextMenuHost from '@/components/functional/ContextMenuHost.vue'
import CuteDialog from '@/components/functional/CuteDialog.vue'
import TimeBlockRecurrenceEditDialog from '@/components/parts/recurrence/TimeBlockRecurrenceEditDialog.vue'
import { useUIStore } from '@/stores/ui'

const uiStore = useUIStore()
const recurrenceDialogContext = computed(() => uiStore.timeBlockRecurrenceEditContext)
</script>

<template>
  <component :is="icon" :size="size" :stroke-width="strokeWidth" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { icons } from 'lucide-vue-next'

type IconName = keyof typeof icons

const props = defineProps({
  name: {
    type: String as () => IconName,
    required: true,
  },
  size: {
    type: Number,
    default: 16,
  },
  strokeWidth: {
    type: Number,
    default: 2,
  },
})

const icon = computed(() => icons[props.name as IconName])
</script>

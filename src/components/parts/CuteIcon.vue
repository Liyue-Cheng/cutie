<template>
  <component :is="icon" :size="size" :stroke-width="strokeWidth" :color="color" />
</template>

/** * @description A wrapper for lucide-vue-next icons, providing a consistent API. */
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
  color: {
    type: String,
    default: 'currentColor',
  },
})

const icon = computed(() => icons[props.name as IconName])
</script>

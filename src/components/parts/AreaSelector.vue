<script setup lang="ts">
import { onMounted } from 'vue'
import { useAreaStore } from '@/stores/area'

defineProps<{
  modelValue: string | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | null]
}>()

const areaStore = useAreaStore()

onMounted(async () => {
  if (areaStore.allAreas.length === 0) {
    await areaStore.fetchAreas()
  }
})

function handleChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  emit('update:modelValue', value || null)
}
</script>

<template>
  <select :value="modelValue || ''" class="area-selector" @change="handleChange">
    <option value="">无区域</option>
    <option v-for="area in areaStore.rootAreas" :key="area.id" :value="area.id">
      {{ area.name }}
    </option>
    <optgroup
      v-for="parent in areaStore.rootAreas"
      :key="parent.id + '-group'"
      :label="parent.name"
    >
      <option v-for="child in areaStore.getChildAreas(parent.id)" :key="child.id" :value="child.id">
        └─ {{ child.name }}
      </option>
    </optgroup>
  </select>
</template>

<style scoped>
.area-selector {
  width: 100%;
  padding: 0.8rem;
  font-size: 1.5rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  cursor: pointer;
}

.area-selector:focus {
  outline: none;
  border-color: var(--color-primary, #4a90e2);
}
</style>

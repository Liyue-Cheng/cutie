<template>
  <FullCalendar :options="calendarOptions" />
</template>

<script setup lang="ts">
import FullCalendar from '@fullcalendar/vue3'
import dayGridPlugin from '@fullcalendar/daygrid'
import interactionPlugin from '@fullcalendar/interaction'
import { reactive } from 'vue'

const calendarOptions = reactive({
  plugins: [dayGridPlugin, interactionPlugin],
  initialView: 'dayGridMonth',
  weekends: true,
  events: [
    { title: 'Meeting', date: '2025-09-22' },
    { title: 'Event', date: '2025-09-23' },
  ],
})
</script>

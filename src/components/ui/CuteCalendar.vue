<template>
  <FullCalendar :options="calendarOptions" :key="activityStore.activities.size" />
</template>

<script setup lang="ts">
import FullCalendar from '@fullcalendar/vue3'
import dayGridPlugin from '@fullcalendar/daygrid'
import interactionPlugin from '@fullcalendar/interaction'
import timeGridPlugin from '@fullcalendar/timegrid'
import { reactive, onMounted, computed } from 'vue'
import { useActivityStore } from '@/stores/activity'
import type { EventInput, EventChangeArg, DateSelectArg } from '@fullcalendar/core'

const activityStore = useActivityStore()

onMounted(() => {
  activityStore.fetchActivities()
})

const calendarEvents = computed((): EventInput[] => {
  return activityStore.allActivities.map((activity) => ({
    id: activity.id,
    title: activity.title ?? 'Untitled',
    start: activity.start_time,
    end: activity.end_time,
    allDay: activity.is_all_day,
    color: activity.color ?? undefined,
  }))
})

async function handleDateSelect(selectInfo: DateSelectArg) {
  const calendarApi = selectInfo.view.calendar
  calendarApi.unselect() // clear date selection

  const title = prompt('Please enter a new title for your event')
  if (title) {
    try {
      await activityStore.createActivity({
        title,
        start_time: selectInfo.start.toISOString(),
        end_time: selectInfo.end.toISOString(),
        is_all_day: selectInfo.allDay,
      })
    } catch (error) {
      console.error('Failed to create event:', error)
      alert(`Error: Could not create the event. It might be overlapping with another event.`)
      // No need to manually revert, as it was never added to the store successfully
    }
  }
}

async function handleEventChange(changeInfo: EventChangeArg) {
  const { event } = changeInfo

  try {
    await activityStore.updateActivity(event.id, {
      title: event.title,
      start_time: event.start?.toISOString(),
      end_time: event.end?.toISOString(),
      is_all_day: event.allDay,
    })
  } catch (error) {
    console.error('Failed to update event:', error)
    alert(`Error: Could not update the event. It might be overlapping with another event.`)
    changeInfo.revert() // Revert the change on the calendar
  }
}

const calendarOptions = reactive({
  plugins: [interactionPlugin, timeGridPlugin],
  headerToolbar: false as const,
  dayHeaders: false,
  initialView: 'timeGridDay',
  slotLabelFormat: {
    hour: '2-digit' as const,
    minute: '2-digit' as const,
    hour12: false,
  },
  height: '100%',
  weekends: true,
  editable: true,
  selectable: true,
  events: calendarEvents,
  select: handleDateSelect,
  eventChange: handleEventChange,
})
</script>

<style>
/*
 * FullCalendar Customizations
 *
 * To customize the calendar's appearance, you can override its CSS variables
 * or target its specific classes. Using the browser's developer tools is the
 * best way to inspect elements and find the right selectors.
 *
 * Example: Change the background color of the current day.
 */
.fc .fc-day-today {
  background-color: rgb(74 144 226 / 15%) !important; /* A light, translucent blue */
}

/* Remove the border from minor timegrid slots */
.fc .fc-timegrid-slot-label {
  transform: translateY(-50%);
}

.fc .fc-timegrid-slot-label,
.fc .fc-timegrid-slot-minor {
  border: none !important;
}

/* Hide the default vertical scrollbar */
.fc .fc-scroller::-webkit-scrollbar {
  width: 8px;
  background-color: transparent;
}

/* Style the track */
.fc .fc-scroller::-webkit-scrollbar-track {
  background-color: transparent;
}

/* Style the thumb */
.fc .fc-scroller::-webkit-scrollbar-thumb {
  background-color: var(--color-border-hover);
  border-radius: 4px;
}
</style>

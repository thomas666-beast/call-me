<script setup lang="ts">
import type { Cel } from '../api/cdr'

defineProps<{
  events: Cel[]
}>()

function formatTime(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleTimeString('en-GB', { hour12: false })
}

function eventColor(eventtype: string): string {
  switch (eventtype) {
    case 'CHAN_START':   return 'gray'
    case 'ANSWER':       return 'green'
    case 'BRIDGE_ENTER': return 'blue'
    case 'BRIDGE_EXIT':  return 'amber'
    case 'HANGUP':
    case 'CHAN_END':     return 'red'
    case 'LINKEDID_END': return 'gray'
    case 'APP_START':
    case 'APP_END':      return 'purple'
    default:             return 'slate'
  }
}
</script>

<template>
  <div class="timeline">
    <div v-if="events.length === 0" class="empty">No CEL events for this call.</div>
    <div
      v-for="evt in events"
      :key="evt.id"
      class="event"
    >
      <span class="time">{{ formatTime(evt.eventtime) }}</span>
      <span class="badge" :class="eventColor(evt.eventtype)">{{ evt.eventtype }}</span>
      <span class="channel">{{ evt.channame }}</span>
      <span v-if="evt.exten" class="exten">→ {{ evt.exten }}</span>
      <span v-if="evt.appname" class="app">{{ evt.appname }}</span>
    </div>
  </div>
</template>

<style scoped>
.timeline {
  font-family: ui-monospace, monospace;
  font-size: 12px;
  border: 1px solid #1e293b;
  border-radius: 4px;
  max-height: 400px;
  overflow-y: auto;
}
.empty { padding: 20px; text-align: center; color: #64748b; }
.event {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 5px 10px;
  border-bottom: 1px solid #0f172a;
}
.event:last-child { border-bottom: none; }
.time { color: #94a3b8; min-width: 70px; }
.badge {
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 500;
  min-width: 110px;
  text-align: center;
}
.badge.gray   { background: #334155; color: #cbd5e1; }
.badge.green  { background: #14532d; color: #86efac; }
.badge.blue   { background: #1e3a8a; color: #93c5fd; }
.badge.amber  { background: #78350f; color: #fcd34d; }
.badge.red    { background: #7f1d1d; color: #fecaca; }
.badge.purple { background: #581c87; color: #d8b4fe; }
.badge.slate  { background: #1e293b; color: #94a3b8; }
.channel { color: #38bdf8; }
.exten { color: #a78bfa; }
.app { color: #64748b; font-style: italic; }
</style>

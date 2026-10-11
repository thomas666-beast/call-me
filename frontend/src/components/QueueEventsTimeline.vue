<script setup lang="ts">
import type { QueueEvent } from '../api/cdr'

defineProps<{
  events: QueueEvent[]
}>()

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString('en-GB', { hour12: false })
}

interface EventMeta {
  label: string
  color: string
  description: string
}

function meta(event: string): EventMeta {
  switch (event) {
    case 'ENTERQUEUE':
      return { label: 'Entered queue', color: 'gray', description: 'Caller joined the queue' }
    case 'CONNECT':
      return { label: 'Connected', color: 'green', description: 'Agent answered' }
    case 'RINGNOANSWER':
      return { label: 'Rang, no answer', color: 'warn', description: 'Agent did not pick up' }
    case 'RINGCANCELED':
      return { label: 'Ring canceled', color: 'warn', description: 'Caller hung up while ringing' }
    case 'COMPLETEAGENT':
      return { label: 'Completed (agent hung up)', color: 'blue', description: 'Agent ended the call' }
    case 'COMPLETECALLER':
      return { label: 'Completed (caller hung up)', color: 'blue', description: 'Caller ended the call' }
    case 'ABANDON':
      return { label: 'Abandoned', color: 'red', description: 'Caller hung up before any agent answered' }
    case 'EXITWITHKEY':
      return { label: 'Exited (key pressed)', color: 'slate', description: 'Caller pressed a key to leave' }
    case 'EXITWITHTIMEOUT':
      return { label: 'Exited (timeout)', color: 'slate', description: 'Caller timed out' }
    case 'PAUSE':
      return { label: 'Agent paused', color: 'warn', description: 'Agent paused' }
    case 'UNPAUSE':
      return { label: 'Agent resumed', color: 'green', description: 'Agent resumed' }
    default:
      return { label: event, color: 'slate', description: '' }
  }
}
</script>

<template>
  <div class="timeline">
    <div v-if="events.length === 0" class="empty">
      No queue events for this call. The call may not have gone through a queue.
    </div>
    <div
      v-for="(evt, i) in events"
      :key="i"
      class="event"
    >
      <span class="time">{{ formatTime(evt.time) }}</span>
      <span class="badge" :class="meta(evt.event).color">
        {{ meta(evt.event).label }}
      </span>
      <span class="queue mono">Q{{ evt.queue_number }}</span>
      <span v-if="evt.agent_extension" class="agent mono">
        Agent {{ evt.agent_extension }}
      </span>
      <span v-else-if="meta(evt.event).description" class="desc">
        {{ meta(evt.event).description }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.timeline {
  font-family: ui-monospace, monospace;
  font-size: 12px;
  border: 1px solid #1e293b;
  border-radius: 4px;
  overflow: hidden;
}
.empty {
  padding: 20px; text-align: center; color: #64748b;
}
.event {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-bottom: 1px solid #0f172a;
}
.event:last-child { border-bottom: none; }
.time { color: #94a3b8; min-width: 70px; }
.badge {
  padding: 2px 10px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 500;
  min-width: 160px;
  text-align: center;
}
.badge.gray   { background: #334155; color: #cbd5e1; }
.badge.green  { background: #14532d; color: #86efac; }
.badge.blue   { background: #1e3a8a; color: #93c5fd; }
.badge.warn   { background: #78350f; color: #fcd34d; }
.badge.red    { background: #7f1d1d; color: #fecaca; }
.badge.slate  { background: #1e293b; color: #94a3b8; }
.queue { color: #38bdf8; }
.agent { color: #a78bfa; }
.desc  { color: #64748b; font-style: italic; }
</style>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import type { ActiveCall } from '../api/agent'
import { useDirectoryStore } from '../stores/directory'

const props = defineProps<{
  call: ActiveCall
}>()

const emit = defineEmits<{
  (e: 'create-contact', phone: string): void
}>()

const directory = useDirectoryStore()
const elapsed = ref(0)
let ticker: number | null = null

const callerName = computed(() => directory.nameFor(props.call.caller_number))
const hasContact = computed(() => directory.contactFor(props.call.caller_number) !== null)

function updateElapsed() {
  const start = new Date(props.call.connected_at).getTime()
  elapsed.value = Math.max(0, Math.floor((Date.now() - start) / 1000))
}

function formatElapsed(secs: number): string {
  const m = Math.floor(secs / 60).toString().padStart(2, '0')
  const s = (secs % 60).toString().padStart(2, '0')
  return `${m}:${s}`
}

onMounted(() => {
  updateElapsed()
  ticker = window.setInterval(updateElapsed, 1000)
})

onUnmounted(() => {
  if (ticker) clearInterval(ticker)
})

watch(() => props.call.callid, updateElapsed)
</script>

<template>
  <div class="card">
    <div class="pulse"></div>
    <div class="content">
      <div class="label">On call</div>
      <div class="number">{{ call.caller_number }}</div>
      <div v-if="callerName" class="name">{{ callerName }}</div>
      <div class="meta">
        Queue {{ call.queue_number }} · {{ formatElapsed(elapsed) }}
      </div>
    </div>
    <div class="actions">
      <button
        v-if="!hasContact && call.caller_number"
        class="small"
        @click="emit('create-contact', call.caller_number)"
      >
        Create contact
      </button>
    </div>
  </div>
</template>

<style scoped>
.card {
  display: flex;
  align-items: center;
  gap: 16px;
  background: #052e16;
  border: 1px solid #166534;
  border-radius: 6px;
  padding: 16px 20px;
}
.pulse {
  width: 12px; height: 12px; border-radius: 50%;
  background: #22c55e;
  box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7);
  animation: pulse 1.6s infinite;
}
@keyframes pulse {
  0%   { box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7); }
  70%  { box-shadow: 0 0 0 10px rgba(34, 197, 94, 0); }
  100% { box-shadow: 0 0 0 0 rgba(34, 197, 94, 0); }
}
.content { flex: 1; }
.label {
  font-size: 11px; color: #86efac; text-transform: uppercase;
  letter-spacing: 0.5px; font-weight: 600;
}
.number {
  font-family: ui-monospace, monospace;
  font-size: 18px;
  color: #f0fdf4;
  margin-top: 2px;
}
.name { color: #bbf7d0; font-size: 14px; margin-top: 2px; }
.meta {
  font-size: 12px; color: #86efac; margin-top: 6px;
  font-family: ui-monospace, monospace;
}
.actions { display: flex; gap: 8px; }
button.small {
  padding: 6px 12px; background: #14532d; color: #bbf7d0;
  border: 1px solid #166534; border-radius: 4px;
  cursor: pointer; font-size: 12px;
}
button.small:hover { background: #166534; }
</style>

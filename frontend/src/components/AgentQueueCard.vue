<script setup lang="ts">
import { ref } from 'vue'
import { PAUSE_REASONS, type AgentQueue } from '../api/agent'

const props = defineProps<{
  queue: AgentQueue
}>()

const emit = defineEmits<{
  (e: 'pause', queue: string, reason: string): void
  (e: 'unpause', queue: string): void
}>()

const selectedReason = ref<string>('break')
const busy = ref(false)

async function doPause() {
  busy.value = true
  try {
    emit('pause', props.queue.number, selectedReason.value)
  } finally {
    setTimeout(() => { busy.value = false }, 500)
  }
}

async function doUnpause() {
  busy.value = true
  try {
    emit('unpause', props.queue.number)
  } finally {
    setTimeout(() => { busy.value = false }, 500)
  }
}
</script>

<template>
  <div class="card" :class="{ paused: queue.paused }">
    <div class="head">
      <div>
        <div class="name">{{ queue.name }}</div>
        <div class="number">Queue {{ queue.number }}</div>
      </div>
      <span class="status" :class="queue.paused ? 'off' : 'on'">
        {{ queue.paused ? 'Paused' : 'Available' }}
      </span>
    </div>

    <div v-if="queue.paused && queue.pause_reason" class="reason">
      Reason: {{ queue.pause_reason }}
    </div>

    <div class="actions">
      <template v-if="!queue.paused">
        <select v-model="selectedReason" :disabled="busy" class="reason-select">
          <option v-for="r in PAUSE_REASONS" :key="r.value" :value="r.value">
            {{ r.label }}
          </option>
        </select>
        <button class="warn" :disabled="busy" @click="doPause">Pause</button>
      </template>
      <template v-else>
        <button class="primary" :disabled="busy" @click="doUnpause">Resume</button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.card {
  background: #1e293b;
  border: 1px solid #334155;
  border-left: 4px solid #22c55e;
  border-radius: 6px;
  padding: 14px 16px;
}
.card.paused { border-left-color: #f59e0b; }
.head { display: flex; justify-content: space-between; align-items: flex-start; }
.name { font-weight: 600; color: #e2e8f0; }
.number { font-size: 12px; color: #64748b; font-family: ui-monospace, monospace; margin-top: 2px; }
.status {
  padding: 2px 10px; border-radius: 10px; font-size: 11px; font-weight: 500;
}
.status.on  { background: #14532d; color: #86efac; }
.status.off { background: #78350f; color: #fcd34d; }
.reason {
  margin-top: 10px; font-size: 12px; color: #94a3b8;
}
.actions {
  margin-top: 12px; display: flex; gap: 8px; align-items: center;
}
.reason-select {
  flex: 1; padding: 6px 8px; background: #0f172a; color: #e2e8f0;
  border: 1px solid #334155; border-radius: 4px; font-size: 13px;
}
button {
  padding: 6px 14px; border: none; border-radius: 4px;
  cursor: pointer; font-size: 13px; color: #e2e8f0;
}
button:disabled { opacity: 0.5; cursor: not-allowed; }
button.warn { background: #b45309; }
button.warn:hover:not(:disabled) { background: #d97706; }
button.primary { background: #0369a1; }
button.primary:hover:not(:disabled) { background: #0284c7; }
</style>

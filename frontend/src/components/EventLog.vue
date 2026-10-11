<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

type AmiEvent = Record<string, string>

const events = ref<AmiEvent[]>([])
const status = ref<'connecting' | 'connected' | 'disconnected'>('connecting')
const wsUrl = 'ws://localhost:8080/ws/events'

let ws: WebSocket | null = null
let reconnectTimer: number | null = null

function connect() {
  status.value = 'connecting'
  ws = new WebSocket(wsUrl)

  ws.onopen = () => { status.value = 'connected' }
  ws.onmessage = (msg) => {
    try {
      const evt = JSON.parse(msg.data) as AmiEvent
      events.value.unshift(evt)
      if (events.value.length > 500) events.value.length = 500
    } catch (e) { console.error('Bad event', e) }
  }
  ws.onclose = () => {
    status.value = 'disconnected'
    reconnectTimer = window.setTimeout(connect, 2000)
  }
  ws.onerror = () => ws?.close()
}

onMounted(connect)
onUnmounted(() => {
  if (reconnectTimer) clearTimeout(reconnectTimer)
  ws?.close()
})
</script>

<template>
  <div>
    <div class="status-row">
      <span class="status" :class="status">● {{ status }}</span>
      <span class="count">{{ events.length }} events</span>
    </div>
    <div class="log">
      <div v-if="events.length === 0" class="empty">Waiting for AMI events…</div>
      <div v-for="(evt, i) in events" :key="i" class="event">
        <span class="event-name">{{ evt.Event ?? 'Unknown' }}</span>
        <span class="event-detail">{{ evt.Channel ?? evt.Uniqueid ?? '' }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.status-row { display: flex; justify-content: space-between; font-size: 13px; color: #94a3b8; margin-bottom: 8px; }
.status.connected { color: #4ade80; }
.status.connecting { color: #facc15; }
.status.disconnected { color: #f87171; }
.log { font-family: ui-monospace, monospace; font-size: 12px; max-height: 400px; overflow-y: auto; border: 1px solid #1e293b; border-radius: 4px; }
.empty { padding: 20px; text-align: center; color: #64748b; }
.event { display: flex; gap: 12px; padding: 4px 8px; border-bottom: 1px solid #1e293b; }
.event-name { color: #38bdf8; min-width: 180px; }
.event-detail { color: #94a3b8; }
</style>
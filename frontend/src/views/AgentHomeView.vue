<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAgentStore } from '../stores/agent'
import { useDirectoryStore } from '../stores/directory'
import { pauseInQueue, unpauseInQueue, getMyCalls } from '../api/agent'
import type { Cdr } from '../api/cdr'
import AgentQueueCard from '../components/AgentQueueCard.vue'
import Pagination from '../components/Pagination.vue'

const PAGE_SIZE = 20

const agent = useAgentStore()
const directory = useDirectoryStore()

const recentCalls = ref<Cdr[]>([])
const offset = ref(0)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)

async function load(resetOffset = false) {
  if (resetOffset) offset.value = 0
  try {
    recentCalls.value = await getMyCalls(PAGE_SIZE, offset.value)
  } catch (e: any) {
    error.value = e.message
  }
}

function nextPage() {
  offset.value += PAGE_SIZE
  load(false)
}

function prevPage() {
  offset.value = Math.max(0, offset.value - PAGE_SIZE)
  load(false)
}

async function onPause(queue: string, reason: string) {
  try {
    await pauseInQueue(queue, reason)
    await agent.refreshQueues()
    showToast(`Paused in ${queue}`)
  } catch (e: any) {
    error.value = e.message
  }
}

async function onUnpause(queue: string) {
  try {
    await unpauseInQueue(queue)
    await agent.refreshQueues()
    showToast(`Resumed in ${queue}`)
  } catch (e: any) {
    error.value = e.message
  }
}

function formatDate(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleString('en-GB', {
    day: '2-digit', month: '2-digit',
    hour: '2-digit', minute: '2-digit', hour12: false,
  })
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`
  const m = Math.floor(secs / 60)
  const s = secs % 60
  return `${m}m ${s}s`
}

/// Was the agent the receiving side of this call?
function isInbound(call: Cdr): boolean {
  const me = agent.me?.extension
  if (!me) return false
  if (call.kind === 'queue') return true
  return call.dst === me
}

/// The number of the other party on the call.
function otherParty(call: Cdr): string {
  const me = agent.me?.extension
  if (!me) return call.src || call.dst
  if (call.kind === 'queue') return call.src
  if (call.src === me) return call.dst
  if (call.dst === me) return call.src
  return call.src || call.dst
}

/// Short label describing the call from the agent's perspective.
interface CallContext {
  typeLabel: string
  typeColor: string
}

function callContext(call: Cdr): CallContext {
  if (call.kind === 'queue') {
    const queueNum = call.dst
    const queueName = directory.nameFor(queueNum)
    return {
      typeLabel: queueName ? `Queue · ${queueName}` : `Queue · ${queueNum}`,
      typeColor: 'blue',
    }
  }
  if (call.kind === 'internal') {
    return { typeLabel: 'Internal', typeColor: 'grey' }
  }
  if (call.kind === 'outbound') {
    return { typeLabel: 'External', typeColor: 'amber' }
  }
  return { typeLabel: 'External', typeColor: 'purple' }
}

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => { toast.value = null }, 3000)
}

onMounted(async () => {
  if (!directory.loaded) await directory.ensureLoaded()
  await agent.refreshQueues()
  await load()
})
</script>

<template>
  <div>
    <div class="header">
      <h2>My Panel</h2>
      <div class="subtitle">
        {{ agent.me?.username }}
        <span v-if="agent.me?.extension"> · ext {{ agent.me.extension }}</span>
      </div>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <section>
      <h3>My queues</h3>
      <div v-if="agent.queues.length === 0" class="empty">
        You are not assigned to any queues yet.
      </div>
      <div v-else class="queue-grid">
        <AgentQueueCard
          v-for="q in agent.queues"
          :key="q.number"
          :queue="q"
          @pause="onPause"
          @unpause="onUnpause"
        />
      </div>
    </section>

    <section>
      <h3>My recent calls</h3>
      <div v-if="recentCalls.length === 0" class="empty">No calls yet.</div>
      <table v-else>
        <thead>
          <tr>
            <th>Date / Time</th>
            <th>Direction</th>
            <th>With</th>
            <th>Type</th>
            <th>Talk</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="c in recentCalls" :key="c.id">
            <td class="mono small">{{ formatDate(c.calldate) }}</td>

            <td>
              <span
                class="dir-badge"
                :class="isInbound(c) ? 'in' : 'out'"
              >
                {{ isInbound(c) ? '↓ Inbound' : '↑ Outbound' }}
              </span>
            </td>

            <td class="mono">
              <template v-if="directory.nameFor(otherParty(c))">
                <span class="display-name">
                  {{ directory.nameFor(otherParty(c)) }}
                </span>
                <span class="number">({{ otherParty(c) }})</span>
              </template>
              <span v-else>{{ otherParty(c) || '—' }}</span>
            </td>

            <td>
              <span
                class="type-badge"
                :class="callContext(c).typeColor"
              >
                {{ callContext(c).typeLabel }}
              </span>
            </td>

            <td class="mono small">{{ formatDuration(c.billsec) }}</td>
          </tr>
        </tbody>
      </table>

      <Pagination
        v-if="recentCalls.length > 0 || offset > 0"
        :offset="offset"
        :limit="PAGE_SIZE"
        :count="recentCalls.length"
        @prev="prevPage"
        @next="nextPage"
      />
    </section>

    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<style scoped>
.header { margin-bottom: 24px; }
h2 { margin: 0; }
.subtitle { color: #94a3b8; font-size: 13px; margin-top: 4px; }

section { margin-bottom: 28px; }
h3 {
  margin: 0 0 12px 0; color: #cbd5e1; font-size: 13px;
  text-transform: uppercase; letter-spacing: 0.5px;
}

.queue-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align: left; padding: 10px 8px; border-bottom: 1px solid #1e293b; }
th {
  color: #94a3b8; font-weight: 500; font-size: 12px;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #cbd5e1; }
.display-name { color: #e2e8f0; }
.number { color: #64748b; font-size: 12px; margin-left: 6px; }

.dir-badge {
  display: inline-block; padding: 2px 8px; border-radius: 3px;
  font-size: 11px; font-weight: 500;
}
.dir-badge.in  { background: #14532d; color: #86efac; }
.dir-badge.out { background: #1e3a8a; color: #93c5fd; }

.type-badge {
  display: inline-block; padding: 2px 8px; border-radius: 3px;
  font-size: 11px; font-weight: 500;
}
.type-badge.blue   { background: #1e3a8a; color: #93c5fd; }
.type-badge.grey   { background: #334155; color: #cbd5e1; }
.type-badge.amber  { background: #78350f; color: #fcd34d; }
.type-badge.purple { background: #581c87; color: #d8b4fe; }

.empty {
  padding: 24px; text-align: center; color: #64748b; font-size: 13px;
  background: #1e293b; border-radius: 6px;
}
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }

.toast {
  position: fixed; bottom: 24px; right: 24px;
  background: #0369a1; color: white; padding: 12px 20px;
  border-radius: 6px; font-size: 13px; z-index: 200;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
</style>

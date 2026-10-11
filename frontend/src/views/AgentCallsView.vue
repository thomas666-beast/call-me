<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useAgentStore } from '../stores/agent'
import { useAgentTabsStore } from '../stores/agentTabs'
import { pauseInQueue, unpauseInQueue } from '../api/agent'
import AgentTabsBar from '../components/AgentTabsBar.vue'
import AgentCallTab from '../components/AgentCallTab.vue'

const agent = useAgentStore()
const tabs = useAgentTabsStore()

const hasTabs = computed(() => tabs.tabs.length > 0)
const activeTab = computed(() => tabs.activeTab)

onMounted(async () => {
  await agent.refreshQueues()
  tabs.clearUnseen()
})

async function onPause(queue: string, reason: string) {
  await pauseInQueue(queue, reason)
  await agent.refreshQueues()
}

async function onUnpause(queue: string) {
  await unpauseInQueue(queue)
  await agent.refreshQueues()
}

function selectReason(queue: string): string | null {
  const reason = prompt(
    'Pause reason (break, lunch, meeting, training, wrap-up, other):',
    'break'
  )
  return reason
}
</script>

<template>
  <div>
    <div class="header">
      <h2>Received Calls</h2>
    </div>

    <!-- Queue status pills -->
    <div v-if="agent.queues.length > 0" class="status-bar">
      <div
        v-for="q in agent.queues"
        :key="q.number"
        class="pill"
        :class="{ paused: q.paused }"
      >
        <span class="dot" :class="q.paused ? 'off' : 'on'"></span>
        <span class="pill-name">{{ q.name }}</span>
        <span v-if="q.paused && q.pause_reason" class="pill-reason">
          ({{ q.pause_reason }})
        </span>
        <button
          v-if="!q.paused"
          class="pill-btn"
          @click="onPause(q.number, selectReason(q.number) || 'break')"
        >
          Pause
        </button>
        <button v-else class="pill-btn resume" @click="onUnpause(q.number)">
          Resume
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <AgentTabsBar />

    <!-- Tab content or empty state -->
    <AgentCallTab v-if="activeTab" :tab="activeTab" />
    <div v-else class="empty-call">
      <div class="empty-icon">📞</div>
      <p>No calls for the moment.</p>
      <p class="empty-hint">
        When a call comes in, a tab will open here automatically.
      </p>
    </div>
  </div>
</template>

<style scoped>
.header { margin-bottom: 16px; }
h2 { margin: 0; }

.status-bar {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.pill {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: #1e293b;
  border: 1px solid #334155;
  border-radius: 20px;
  font-size: 13px;
  color: #cbd5e1;
}
.pill.paused { border-color: #78350f; }
.pill-name { font-weight: 500; }
.pill-reason { color: #fcd34d; font-size: 12px; }
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}
.dot.on { background: #22c55e; }
.dot.off { background: #f59e0b; }
.pill-btn {
  padding: 3px 10px;
  background: #334155;
  color: #e2e8f0;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  font-size: 11px;
}
.pill-btn:hover { background: #475569; }
.pill-btn.resume { background: #0369a1; }
.pill-btn.resume:hover { background: #0284c7; }

.empty-call {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  background: #1e293b;
  border: 1px dashed #334155;
  border-radius: 8px;
  color: #64748b;
  text-align: center;
}
.empty-icon { font-size: 48px; margin-bottom: 16px; opacity: 0.5; }
.empty-call p { margin: 4px 0; font-size: 15px; color: #94a3b8; }
.empty-hint { font-size: 13px !important; color: #64748b !important; margin-top: 12px !important; }
</style>

import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  getMe,
  getMyQueues,
  getActiveCall,
  type AgentMe,
  type AgentQueue,
  type ActiveCall,
} from '../api/agent'

export const useAgentStore = defineStore('agent', () => {
  const me = ref<AgentMe | null>(null)
  const queues = ref<AgentQueue[]>([])
  const activeCall = ref<ActiveCall | null>(null)
  const loading = ref(false)

  let pollTimer: number | null = null
  let queuesTimer: number | null = null

  async function refreshMe() {
    me.value = await getMe()
  }

  async function refreshQueues() {
    queues.value = await getMyQueues()
  }

  async function refreshActive() {
    activeCall.value = await getActiveCall()
  }

  async function loadAll() {
    loading.value = true
    try {
      await Promise.all([refreshMe(), refreshQueues(), refreshActive()])
    } finally {
      loading.value = false
    }
  }

  function startPolling() {
    stopPolling()
    // Active call: every 5 seconds
    pollTimer = window.setInterval(() => {
      refreshActive().catch(() => {})
    }, 5000)
    // Queue states: every 15 seconds (slower, less critical)
    queuesTimer = window.setInterval(() => {
      refreshQueues().catch(() => {})
    }, 15000)
  }

  function stopPolling() {
    if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
    if (queuesTimer) { clearInterval(queuesTimer); queuesTimer = null }
  }

  return {
    me,
    queues,
    activeCall,
    loading,
    loadAll,
    refreshMe,
    refreshQueues,
    refreshActive,
    startPolling,
    stopPolling,
  }
})

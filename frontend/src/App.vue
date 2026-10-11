<script setup lang="ts">
import { RouterView, useRouter, useRoute } from 'vue-router'
import { useAuthStore } from './stores/auth'
import { useAgentStore } from './stores/agent'
import { useAgentTabsStore } from './stores/agentTabs'
import { useDirectoryStore } from './stores/directory'
import { computed, watch } from 'vue'

const auth = useAuthStore()
const agent = useAgentStore()
const tabs = useAgentTabsStore()
const directory = useDirectoryStore()
const router = useRouter()
const route = useRoute()

const isAuthed = computed(() => auth.isAuthenticated)
const isAgent = computed(() => auth.user?.role === 'agent')
const isAdmin = computed(() => auth.user?.role === 'admin')

async function logout() {
  agent.stopPolling()
  await auth.logout()
  router.push('/login')
}

// Start / stop polling based on role
watch(
  () => auth.user?.role,
  async (role) => {
    if (role === 'agent') {
      await directory.ensureLoaded()
      await agent.loadAll()
      agent.startPolling()
    } else {
      agent.stopPolling()
    }
  },
  { immediate: true }
)

// Open a tab whenever an active call appears
watch(
  () => agent.activeCall?.callid,
  async () => {
    if (agent.activeCall) {
      await tabs.handleActiveCall({
        callerNumber: agent.activeCall.caller_number,
        callid: agent.activeCall.callid,
        linkedid: agent.activeCall.linkedid,
        queueNumber: agent.activeCall.queue_number,
      })
      if (route.path === '/agent/calls') {
        tabs.clearUnseen()
      }
    }
  }
)

// Clear unseen when the agent is on the calls page
watch(
  () => route.path,
  (path) => {
    if (path === '/agent/calls') {
      tabs.clearUnseen()
    }
  }
)
</script>

<template>
  <div class="app">
    <header>
      <h1>Asterisk Manager</h1>
      <nav v-if="isAuthed">
        <template v-if="isAgent">
          <RouterLink to="/agent">My Panel</RouterLink>
          <RouterLink to="/agent/calls" class="call-link">
            Received Calls
            <span v-if="tabs.unseenCount > 0" class="badge">{{ tabs.unseenCount }}</span>
          </RouterLink>
          <RouterLink to="/agent/contacts">Contacts</RouterLink>
        </template>

        <template v-if="isAdmin">
          <RouterLink to="/dashboard">Dashboard</RouterLink>
          <RouterLink to="/admin/users">Users</RouterLink>
          <RouterLink v-if="isAdmin" to="/admin/agents">Agents</RouterLink>
          <RouterLink to="/admin/extensions">Extensions</RouterLink>
          <RouterLink to="/admin/queues">Queues</RouterLink>
          <RouterLink to="/admin/calls">Calls</RouterLink>
          <RouterLink to="/admin/contacts">Contacts</RouterLink>
          <RouterLink to="/admin/settings">Settings</RouterLink>
        </template>

        <span class="user">{{ auth.user?.username }} ({{ auth.user?.role }})</span>
        <button @click="logout">Logout</button>
      </nav>
    </header>
    <main>
      <RouterView />
    </main>
  </div>
</template>

<style>
* { box-sizing: border-box; }
body { margin: 0; font-family: ui-sans-serif, system-ui, sans-serif; background: #0f172a; color: #e2e8f0; }
.app { max-width: 1200px; margin: 0 auto; padding: 20px; }
header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; padding-bottom: 12px; border-bottom: 1px solid #1e293b; }
h1 { font-size: 18px; margin: 0; }
nav { display: flex; align-items: center; gap: 16px; font-size: 14px; }
nav a { color: #94a3b8; text-decoration: none; position: relative; }
nav a.router-link-active { color: #38bdf8; }
.call-link { display: inline-flex; align-items: center; gap: 6px; }
.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: #dc2626;
  color: white;
  border-radius: 9px;
  font-size: 11px;
  font-weight: 600;
}
.user { color: #64748b; }
button { background: #1e293b; color: #e2e8f0; border: 1px solid #334155; padding: 6px 12px; border-radius: 4px; cursor: pointer; }
button:hover { background: #334155; }
</style>

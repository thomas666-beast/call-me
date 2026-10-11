<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  listAgents,
  createAgent,
  updateAgent,
  deleteAgent,
  type Agent,
  type AgentInput,
  type AgentUpdate,
  type AgentCredentials,
} from '../api/agents'
import AgentForm from '../components/AgentForm.vue'
import AgentCredentialsModal from '../components/AgentCredentialsModal.vue'

const agents = ref<Agent[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)

const formOpen = ref(false)
const editing = ref<Agent | null>(null)
const deleteTarget = ref<Agent | null>(null)

const credsOpen = ref(false)
const creds = ref<AgentCredentials | null>(null)

async function load() {
  loading.value = true
  error.value = null
  try {
    agents.value = await listAgents()
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

function openNew() {
  editing.value = null
  formOpen.value = true
}

function openEdit(a: Agent) {
  editing.value = a
  formOpen.value = true
}

function closeForm() {
  formOpen.value = false
  editing.value = null
}

async function handleCreate(payload: { input: AgentInput | AgentUpdate }) {
  error.value = null
  const input = payload.input as AgentInput
  try {
    const created = await createAgent(input)
    showToast(`Created agent ${created.username}`)
    closeForm()
    await load()

    creds.value = {
      displayName: input.display_name,
      loginUsername: created.username,
      loginPassword: input.password,
      sipUsername: created.extension_number,
      sipPassword: input.sip_secret,
    }
    credsOpen.value = true
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleUpdate(payload: { id?: string; input: AgentInput | AgentUpdate }) {
  if (!payload.id) return
  error.value = null
  const input = payload.input as AgentUpdate
  try {
    const updated = await updateAgent(payload.id, input)
    showToast(`Updated agent ${updated.username}`)
    closeForm()
    await load()

    if (input.password) {
      creds.value = {
        displayName: updated.display_name ?? updated.username,
        loginUsername: updated.username,
        loginPassword: input.password,
        sipUsername: null,
        sipPassword: null,
      }
      credsOpen.value = true
    }
  } catch (e: any) {
    error.value = e.message
  }
}

function generatePassword(): string {
  const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  let s = ''
  for (let i = 0; i < 16; i++) s += chars[Math.floor(Math.random() * chars.length)]
  return s
}

async function resetPassword(a: Agent) {
  const confirmed = confirm(
    `Reset the login password for ${a.display_name ?? a.username}?`,
  )
  if (!confirmed) return

  const newPassword = generatePassword()
  error.value = null
  try {
    const updated = await updateAgent(a.id, { password: newPassword })
    showToast(`Password reset for ${updated.username}`)
    await load()

    creds.value = {
      displayName: updated.display_name ?? updated.username,
      loginUsername: updated.username,
      loginPassword: newPassword,
      sipUsername: null,
      sipPassword: null,
    }
    credsOpen.value = true
  } catch (e: any) {
    error.value = e.message
  }
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  const username = deleteTarget.value.username
  try {
    await deleteAgent(deleteTarget.value.id)
    deleteTarget.value = null
    showToast(`Deactivated agent ${username}`)
    await load()
  } catch (e: any) {
    error.value = e.message
    deleteTarget.value = null
  }
}

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => { toast.value = null }, 3000)
}

onMounted(load)
</script>

<template>
  <div>
    <div class="header">
      <div>
        <h2>Agents</h2>
        <p class="subtitle">
          Extension is the login username. Credentials are shown once after
          creating or resetting.
        </p>
      </div>
      <button class="primary" @click="openNew">New agent</button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="loading" class="empty">Loading…</div>
    <div v-else-if="agents.length === 0" class="empty">
      No agents yet. Click "New agent" to create one.
    </div>

    <table v-else>
      <thead>
        <tr>
          <th>Login / Extension</th>
          <th>Display name</th>
          <th>Queues</th>
          <th>Status</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="a in agents" :key="a.id" :class="{ inactive: !a.active }">
          <td class="mono">{{ a.extension_number ?? '—' }}</td>
          <td>{{ a.display_name ?? '—' }}</td>
          <td class="mono small">
            {{ a.queue_numbers.length ? a.queue_numbers.join(', ') : '—' }}
          </td>
          <td>
            <span class="badge" :class="a.active ? 'on' : 'off'">
              {{ a.active ? 'active' : 'inactive' }}
            </span>
          </td>
          <td class="row-actions">
            <button @click="openEdit(a)">Edit</button>
            <button v-if="a.active" @click="resetPassword(a)">Reset password</button>
            <button v-if="a.active" class="danger" @click="deleteTarget = a">
              Deactivate
            </button>
          </td>
        </tr>
      </tbody>
    </table>

    <AgentForm
      :open="formOpen"
      :editing="editing"
      @close="closeForm"
      @submit-create="handleCreate"
      @submit-update="handleUpdate"
    />

    <AgentCredentialsModal
      :open="credsOpen"
      :credentials="creds"
      @close="credsOpen = false"
    />

    <div v-if="deleteTarget" class="overlay" @click.self="deleteTarget = null">
      <div class="modal">
        <h3>Deactivate {{ deleteTarget.username }}?</h3>
        <p>
          The agent can no longer log in, and their extension is disabled.
          Historical calls and notes are preserved.
        </p>
        <div class="actions">
          <button @click="deleteTarget = null">Cancel</button>
          <button class="danger" @click="confirmDelete">Deactivate</button>
        </div>
      </div>
    </div>

    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<style scoped>
.header {
  display: flex; justify-content: space-between; align-items: flex-start;
  margin-bottom: 20px;
}
h2 { margin: 0; }
.subtitle { color: #64748b; font-size: 13px; margin: 4px 0 0 0; max-width: 520px; }
.header .primary {
  padding: 8px 14px; background: #0369a1; color: white;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.header .primary:hover { background: #0284c7; }

table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align: left; padding: 10px 8px; border-bottom: 1px solid #1e293b; }
th {
  color: #94a3b8; font-weight: 500; font-size: 12px;
  text-transform: uppercase; letter-spacing: 0.5px;
}
tr.inactive { opacity: 0.5; }
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #94a3b8; }

.badge {
  display: inline-block; padding: 2px 8px; border-radius: 10px; font-size: 11px;
}
.badge.on  { background: #14532d; color: #86efac; }
.badge.off { background: #3f3f46; color: #a1a1aa; }

.row-actions { text-align: right; }
.row-actions button {
  padding: 4px 10px; margin-left: 6px; background: transparent;
  color: #94a3b8; border: 1px solid #334155; border-radius: 4px;
  cursor: pointer; font-size: 12px;
}
.row-actions button:hover { background: #334155; color: #e2e8f0; }
.row-actions .danger:hover {
  background: #7f1d1d; border-color: #7f1d1d; color: #fecaca;
}

.empty { padding: 40px; text-align: center; color: #64748b; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }

.overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.modal {
  background: #1e293b; padding: 24px; border-radius: 8px;
  max-width: 440px; border: 1px solid #334155;
}
.modal h3 { margin: 0 0 12px 0; }
.modal p { color: #94a3b8; font-size: 13px; margin-bottom: 20px; line-height: 1.5; }
.modal .actions { display: flex; justify-content: flex-end; gap: 8px; }
.modal button {
  padding: 8px 16px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer;
}
.modal .danger { background: #7f1d1d; }
.modal .danger:hover { background: #991b1b; }

.toast {
  position: fixed; bottom: 24px; right: 24px;
  background: #0369a1; color: white; padding: 12px 20px;
  border-radius: 6px; font-size: 13px; z-index: 200;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
</style>

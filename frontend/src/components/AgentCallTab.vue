<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAgentTabsStore, type AgentTab } from '../stores/agentTabs'
import {
  createCallNote,
  createAgentContact,
  getAgentContact,
} from '../api/agent'
import type { ContactInput } from '../api/contacts'

const props = defineProps<{
  tab: AgentTab
}>()

const tabsStore = useAgentTabsStore()

const saving = ref(false)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)

// --- Contact creation form state ---
const showCreateForm = ref(false)
const newName = ref('')
const newCompany = ref('')
const newEmail = ref('')
const newNotes = ref('')

const hasContact = computed(() => props.tab.contact !== null)
const contact = computed(() => props.tab.contact?.contact ?? null)
const callHistory = computed(() => props.tab.contact?.calls ?? [])
const isDirty = computed(() => tabsStore.isDirty(props.tab.id))

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => { toast.value = null }, 3000)
}

function openCreateForm() {
  newName.value = ''
  newCompany.value = ''
  newEmail.value = ''
  newNotes.value = ''
  showCreateForm.value = true
}

async function submitNewContact() {
  error.value = null
  if (!newName.value.trim()) {
    error.value = 'Name is required'
    return
  }
  try {
    const input: ContactInput = {
      phone_number: props.tab.phoneNumber,
      display_name: newName.value.trim(),
      company: newCompany.value.trim() || null,
      email: newEmail.value.trim() || null,
      notes: newNotes.value,
      tags: [],
    }
    const created = await createAgentContact(input)
    const full = await getAgentContact(created.id)
    tabsStore.refreshContact(props.tab.id, full)
    showCreateForm.value = false
    showToast('Contact created')
  } catch (e: any) {
    error.value = e.message
  }
}

async function saveLog() {
  if (!props.tab.callid) {
    error.value = 'No call attached to this tab.'
    return
  }
  const draft = props.tab.logDraft.trim()
  if (!draft) return
  saving.value = true
  error.value = null
  try {
    const note = await createCallNote(props.tab.callid, draft)
    tabsStore.appendNote(props.tab.id, note)
    tabsStore.markSaved(props.tab.id)
    tabsStore.updateDraft(props.tab.id, '')
    showToast('Log saved')
  } catch (e: any) {
    error.value = e.message
  } finally {
    saving.value = false
  }
}

async function saveAndClose() {
  await saveLog()
  if (!error.value) {
    tabsStore.closeTab(props.tab.id)
  }
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString('en-GB', { hour12: false })
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`
  const m = Math.floor(secs / 60)
  const s = secs % 60
  return `${m}m ${s}s`
}

function onDraftInput(e: Event) {
  const value = (e.target as HTMLTextAreaElement).value
  tabsStore.updateDraft(props.tab.id, value)
}
</script>

<template>
  <div class="tab-content">
    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="tab.loading" class="empty">Loading contact…</div>

    <div v-else>
      <!-- CONTACT INFO -->
      <section v-if="hasContact && contact" class="card">
        <div class="grid">
          <div class="field">
            <span class="label">Name</span>
            <span class="value">{{ contact.display_name }}</span>
          </div>
          <div class="field">
            <span class="label">Phone</span>
            <span class="value mono">{{ contact.phone_number }}</span>
          </div>
          <div class="field">
            <span class="label">Company</span>
            <span class="value">{{ contact.company ?? '—' }}</span>
          </div>
          <div class="field">
            <span class="label">Email</span>
            <span class="value">{{ contact.email ?? '—' }}</span>
          </div>
          <div class="field">
            <span class="label">Tags</span>
            <span class="value">
              <span v-for="t in contact.tags" :key="t" class="tag">{{ t }}</span>
              <span v-if="contact.tags.length === 0">—</span>
            </span>
          </div>
        </div>
        <div v-if="contact.notes" class="notes-block">
          <span class="label">Contact notes</span>
          <p>{{ contact.notes }}</p>
        </div>
      </section>

      <!-- NO CONTACT YET -->
      <section v-else-if="!showCreateForm" class="card no-contact">
        <p>No contact found for <span class="mono">{{ tab.phoneNumber }}</span>.</p>
        <button class="primary" @click="openCreateForm">Create contact</button>
      </section>

      <section v-else class="card">
        <h3>New contact</h3>
        <div class="form-row">
          <label>
            Display name *
            <input v-model="newName" placeholder="John Smith" />
          </label>
          <label>
            Company
            <input v-model="newCompany" placeholder="Acme" />
          </label>
        </div>
        <div class="form-row">
          <label>
            Email
            <input v-model="newEmail" type="email" placeholder="john@example.com" />
          </label>
          <label>
            Phone
            <input :value="tab.phoneNumber" readonly class="readonly" />
          </label>
        </div>
        <label>
          Notes
          <textarea v-model="newNotes" rows="3"></textarea>
        </label>
        <div class="form-actions">
          <button @click="showCreateForm = false">Cancel</button>
          <button class="primary" @click="submitNewContact">Create</button>
        </div>
      </section>

      <!-- CALL HISTORY -->
      <section v-if="hasContact && callHistory.length > 0">
        <h3>Call history ({{ callHistory.length }})</h3>
        <table>
          <thead>
            <tr>
              <th>Time</th>
              <th>From</th>
              <th>To</th>
              <th>Talk</th>
              <th>Result</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="c in callHistory" :key="c.id">
              <td class="mono small">{{ formatDate(c.calldate) }}</td>
              <td class="mono">{{ c.src }}</td>
              <td class="mono">{{ c.dst }}</td>
              <td class="mono small">{{ formatDuration(c.billsec) }}</td>
              <td class="small">{{ c.disposition }}</td>
            </tr>
          </tbody>
        </table>
      </section>

      <!-- SAVED NOTES FOR THIS CALL -->
      <section v-if="tab.notes.length > 0">
        <h3>Saved logs</h3>
        <div class="note-list">
          <div v-for="n in tab.notes" :key="n.id" class="note">
            <div class="note-head">
              <span class="author">{{ n.author_username }}</span>
              <span class="time">{{ formatDate(n.created_at) }}</span>
            </div>
            <div class="note-body">{{ n.note }}</div>
          </div>
        </div>
      </section>

      <!-- LOG TEXTAREA -->
      <section>
        <h3>
          Log this call
          <span v-if="tab.callid" class="mono small"> — call {{ tab.callid }}</span>
        </h3>
        <textarea
          class="log-input"
          rows="5"
          placeholder="Describe what happened on this call…"
          :value="tab.logDraft"
          @input="onDraftInput"
        ></textarea>
        <div class="log-actions">
          <button
            class="primary"
            :disabled="saving || !tab.logDraft.trim()"
            @click="saveLog"
          >
            {{ saving ? 'Saving…' : 'Save log' }}
          </button>
          <button
            class="primary"
            :disabled="saving || !tab.logDraft.trim()"
            @click="saveAndClose"
          >
            Save &amp; Close
          </button>
          <span v-if="isDirty" class="dirty">Unsaved changes</span>
        </div>
      </section>
    </div>

    <div v-if="toast" class="toast-inline">{{ toast }}</div>
  </div>
</template>

<style scoped>
.tab-content { display: flex; flex-direction: column; gap: 20px; }
section { display: flex; flex-direction: column; gap: 10px; }
h3 {
  margin: 0; color: #cbd5e1; font-size: 13px;
  text-transform: uppercase; letter-spacing: 0.5px;
}

.card {
  background: #1e293b; border: 1px solid #334155;
  border-radius: 6px; padding: 16px;
  display: flex; flex-direction: column; gap: 12px;
}
.no-contact { align-items: center; padding: 24px; }
.no-contact p { margin: 0; color: #94a3b8; }

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 14px;
}
.field { display: flex; flex-direction: column; gap: 4px; }
.label {
  font-size: 11px; color: #94a3b8;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.value { color: #e2e8f0; font-size: 14px; }
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #cbd5e1; }

.notes-block {
  border-top: 1px solid #334155;
  padding-top: 12px;
  display: flex; flex-direction: column; gap: 4px;
}
.notes-block p {
  margin: 0; white-space: pre-wrap; color: #cbd5e1; line-height: 1.5;
}

.tag {
  display: inline-block; padding: 1px 8px; margin-right: 4px;
  background: #1e3a8a; color: #93c5fd; border-radius: 10px; font-size: 11px;
}

table { width: 100%; border-collapse: collapse; font-size: 13px; }
th, td { text-align: left; padding: 8px; border-bottom: 1px solid #1e293b; }
th {
  color: #94a3b8; font-weight: 500; font-size: 11px;
  text-transform: uppercase; letter-spacing: 0.5px;
}

.note-list { display: flex; flex-direction: column; gap: 8px; }
.note {
  background: #1e293b; border-left: 3px solid #38bdf8;
  border-radius: 4px; padding: 8px 12px;
}
.note-head {
  display: flex; justify-content: space-between;
  font-size: 11px; color: #94a3b8; margin-bottom: 4px;
}
.author { color: #38bdf8; font-weight: 500; }
.note-body { white-space: pre-wrap; color: #e2e8f0; font-size: 13px; line-height: 1.4; }

.log-input {
  width: 100%; padding: 10px; background: #0f172a;
  border: 1px solid #334155; color: #e2e8f0;
  border-radius: 4px; font-family: inherit; font-size: 13px;
  resize: vertical; min-height: 100px;
}
.log-actions { display: flex; gap: 8px; align-items: center; }
button {
  padding: 8px 16px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
button.primary { background: #0369a1; }
button.primary:not(:disabled):hover { background: #0284c7; }
button:disabled { opacity: 0.4; cursor: not-allowed; }
.dirty { color: #f59e0b; font-size: 12px; font-style: italic; }

.form-row { display: flex; gap: 12px; }
.form-row label { flex: 1; }
label {
  display: block; font-size: 12px; color: #94a3b8; margin-bottom: 8px;
}
input, textarea {
  width: 100%; padding: 8px; margin-top: 4px;
  background: #0f172a; border: 1px solid #334155; color: #e2e8f0;
  border-radius: 4px; font-family: inherit; font-size: 13px;
}
input.readonly { color: #64748b; background: #1e293b; }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px; }

.empty { padding: 24px; text-align: center; color: #64748b; font-size: 13px; }
.error { color: #f87171; font-size: 13px; margin: 0; }
.toast-inline {
  padding: 8px 12px; background: #14532d; color: #86efac;
  border-radius: 4px; font-size: 12px; align-self: flex-start;
}
</style>

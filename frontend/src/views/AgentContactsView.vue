<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  listAgentContacts,
  getAgentContact,
  createAgentContact,
  createCallNote,
} from '../api/agent'
import type { Contact, ContactWithCalls, ContactInput } from '../api/contacts'
import Pagination from '../components/Pagination.vue'

const PAGE_SIZE = 20

const search = ref('')
const results = ref<Contact[]>([])
const searching = ref(false)
const offset = ref(0)
const selected = ref<ContactWithCalls | null>(null)
const loadingDetail = ref(false)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)

const showCreateForm = ref(false)
const newName = ref('')
const newPhone = ref('')
const newCompany = ref('')
const newEmail = ref('')

const editingCallid = ref<string | null>(null)
const noteText = ref('')
const savingNote = ref(false)

const hasResults = computed(() => results.value.length > 0)

async function doSearch(resetOffset = true) {
  if (resetOffset) offset.value = 0
  searching.value = true
  error.value = null
  try {
    results.value = await listAgentContacts(search.value.trim(), PAGE_SIZE, offset.value)
  } catch (e: any) {
    error.value = e.message
  } finally {
    searching.value = false
  }
}

function nextPage() {
  offset.value += PAGE_SIZE
  doSearch(false)
}

function prevPage() {
  offset.value = Math.max(0, offset.value - PAGE_SIZE)
  doSearch(false)
}

async function selectContact(c: Contact) {
  loadingDetail.value = true
  error.value = null
  try {
    selected.value = await getAgentContact(c.id)
  } catch (e: any) {
    error.value = e.message
  } finally {
    loadingDetail.value = false
  }
}

function clearSelection() {
  selected.value = null
  editingCallid.value = null
  noteText.value = ''
}

function openCreate() {
  newName.value = ''
  newPhone.value = search.value.match(/[+\d()\s-]+/)?.[0]?.trim() ?? ''
  newCompany.value = ''
  newEmail.value = ''
  showCreateForm.value = true
}

async function submitNewContact() {
  if (!newName.value.trim() || !newPhone.value.trim()) {
    error.value = 'Name and phone are required'
    return
  }
  error.value = null
  try {
    const input: ContactInput = {
      phone_number: newPhone.value.trim(),
      display_name: newName.value.trim(),
      company: newCompany.value.trim() || null,
      email: newEmail.value.trim() || null,
      notes: '',
      tags: [],
    }
    const created = await createAgentContact(input)
    showCreateForm.value = false
    showToast(`Contact created: ${created.display_name}`)
    await selectContact(created)
  } catch (e: any) {
    error.value = e.message
  }
}

function startNote(callid: string) {
  editingCallid.value = callid
  noteText.value = ''
}

function cancelNote() {
  editingCallid.value = null
  noteText.value = ''
}

async function saveNote() {
  if (!editingCallid.value || !noteText.value.trim()) return
  savingNote.value = true
  error.value = null
  try {
    await createCallNote(editingCallid.value, noteText.value.trim())
    showToast('Log saved')
    editingCallid.value = null
    noteText.value = ''
  } catch (e: any) {
    error.value = e.message
  } finally {
    savingNote.value = false
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

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => { toast.value = null }, 3000)
}
</script>

<template>
  <div>
    <div class="header">
      <h2>Contacts</h2>
      <button class="primary" @click="openCreate">New contact</button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="!selected" class="search-row">
      <input
        v-model="search"
        placeholder="Search by name, phone, or company…"
        @keyup.enter="doSearch()"
      />
      <button class="primary" @click="doSearch()">Search</button>
    </div>

    <div v-if="!selected">
      <div v-if="searching" class="empty">Searching…</div>
      <div v-else-if="hasResults" class="results">
        <button
          v-for="c in results"
          :key="c.id"
          class="result"
          @click="selectContact(c)"
        >
          <span class="result-name">{{ c.display_name }}</span>
          <span class="result-phone">{{ c.phone_number }}</span>
          <span v-if="c.company" class="result-company">{{ c.company }}</span>
        </button>
      </div>
      <div v-else-if="search" class="empty">No contacts match your search.</div>
      <div v-else class="empty">
        Search for a contact by name, phone, or company.
      </div>

      <Pagination
        v-if="hasResults || offset > 0"
        :offset="offset"
        :limit="PAGE_SIZE"
        :count="results.length"
        @prev="prevPage"
        @next="nextPage"
      />
    </div>

    <div v-else>
      <button class="back" @click="clearSelection">← Back to search</button>

      <div class="card">
        <div class="grid">
          <div class="field">
            <span class="label">Name</span>
            <span class="value">{{ selected.contact.display_name }}</span>
          </div>
          <div class="field">
            <span class="label">Phone</span>
            <span class="value mono">{{ selected.contact.phone_number }}</span>
          </div>
          <div class="field">
            <span class="label">Company</span>
            <span class="value">{{ selected.contact.company ?? '—' }}</span>
          </div>
          <div class="field">
            <span class="label">Email</span>
            <span class="value">{{ selected.contact.email ?? '—' }}</span>
          </div>
        </div>
        <div v-if="selected.contact.notes" class="notes-block">
          <span class="label">Notes</span>
          <p>{{ selected.contact.notes }}</p>
        </div>
      </div>

      <h3>Call history ({{ selected.calls.length }})</h3>
      <div v-if="selected.calls.length === 0" class="empty">No calls yet.</div>
      <div v-else class="call-list">
        <div v-for="call in selected.calls" :key="call.id" class="call">
          <div class="call-head">
            <span class="mono small">{{ formatDate(call.calldate) }}</span>
            <span class="mono small">Talk {{ formatDuration(call.billsec) }}</span>
            <span class="dispo small">{{ call.disposition }}</span>
            <button
              v-if="editingCallid !== call.uniqueid"
              class="log-btn"
              @click="startNote(call.uniqueid)"
            >
              Add log
            </button>
          </div>

          <div v-if="call.notes.length > 0" class="note-list">
            <div v-for="n in call.notes" :key="n.id" class="note">
              <div class="note-head">
                <span class="author">{{ n.author_username }}</span>
                <span class="time">{{ formatDate(n.created_at) }}</span>
              </div>
              <div class="note-body">{{ n.note }}</div>
            </div>
          </div>

          <div v-if="editingCallid === call.uniqueid" class="log-editor">
            <textarea
              v-model="noteText"
              rows="3"
              placeholder="What happened on this call?"
            ></textarea>
            <div class="log-actions">
              <button @click="cancelNote">Cancel</button>
              <button
                class="primary"
                :disabled="savingNote || !noteText.trim()"
                @click="saveNote"
              >
                {{ savingNote ? 'Saving…' : 'Save log' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showCreateForm" class="overlay" @click.self="showCreateForm = false">
      <div class="modal">
        <h3>New contact</h3>
        <div class="row">
          <label class="col">
            Name *
            <input v-model="newName" placeholder="John Smith" />
          </label>
          <label class="col">
            Phone *
            <input v-model="newPhone" placeholder="+15551234567" />
          </label>
        </div>
        <div class="row">
          <label class="col">
            Company
            <input v-model="newCompany" placeholder="Acme" />
          </label>
          <label class="col">
            Email
            <input v-model="newEmail" type="email" placeholder="john@example.com" />
          </label>
        </div>
        <div class="actions">
          <button @click="showCreateForm = false">Cancel</button>
          <button class="primary" @click="submitNewContact">Create</button>
        </div>
      </div>
    </div>

    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<style scoped>
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
h2 { margin: 0; }
h3 { margin: 24px 0 12px 0; color: #cbd5e1; font-size: 13px; text-transform: uppercase; letter-spacing: 0.5px; }

.header .primary { padding: 8px 14px; background: #0369a1; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; }
.header .primary:hover { background: #0284c7; }

.search-row { display: flex; gap: 8px; margin-bottom: 16px; }
.search-row input { flex: 1; padding: 8px 12px; background: #1e293b; border: 1px solid #334155; color: #e2e8f0; border-radius: 4px; font-size: 14px; }
.search-row button { padding: 8px 16px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; }
.search-row .primary { background: #0369a1; }
.search-row .primary:hover { background: #0284c7; }

.results { display: flex; flex-direction: column; gap: 4px; }
.result {
  display: flex; gap: 12px; align-items: center;
  padding: 10px 12px; background: #1e293b; border: 1px solid #334155;
  border-radius: 4px; cursor: pointer; text-align: left;
  color: #e2e8f0; font-size: 13px;
}
.result:hover { background: #334155; }
.result-name { font-weight: 500; }
.result-phone { font-family: ui-monospace, monospace; color: #94a3b8; }
.result-company { color: #64748b; font-size: 12px; }

.back { padding: 6px 12px; background: transparent; color: #94a3b8; border: 1px solid #334155; border-radius: 4px; cursor: pointer; font-size: 13px; margin-bottom: 16px; }
.back:hover { color: #e2e8f0; background: #1e293b; }

.card { background: #1e293b; border: 1px solid #334155; border-radius: 6px; padding: 16px; display: flex; flex-direction: column; gap: 12px; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 14px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.label { font-size: 11px; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.5px; }
.value { color: #e2e8f0; font-size: 14px; }
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #cbd5e1; }

.notes-block { border-top: 1px solid #334155; padding-top: 12px; display: flex; flex-direction: column; gap: 4px; }
.notes-block p { margin: 0; white-space: pre-wrap; color: #cbd5e1; line-height: 1.5; }

.call-list { display: flex; flex-direction: column; gap: 8px; }
.call { background: #1e293b; border: 1px solid #334155; border-radius: 4px; padding: 10px 12px; }
.call-head { display: flex; align-items: center; gap: 16px; flex-wrap: wrap; }
.dispo { color: #86efac; font-size: 12px; }
.log-btn { margin-left: auto; padding: 4px 12px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
.log-btn:hover { background: #475569; }

.note-list { margin-top: 10px; display: flex; flex-direction: column; gap: 6px; padding-left: 8px; border-left: 2px solid #1e3a8a; }
.note { background: #0f172a; border-radius: 4px; padding: 8px 10px; }
.note-head { display: flex; justify-content: space-between; font-size: 11px; color: #94a3b8; margin-bottom: 4px; }
.author { color: #38bdf8; font-weight: 500; }
.note-body { white-space: pre-wrap; color: #e2e8f0; font-size: 12px; line-height: 1.4; }

.log-editor { margin-top: 10px; display: flex; flex-direction: column; gap: 8px; }
.log-editor textarea { width: 100%; padding: 8px; background: #0f172a; border: 1px solid #334155; color: #e2e8f0; border-radius: 4px; font-family: inherit; font-size: 13px; resize: vertical; }
.log-actions { display: flex; justify-content: flex-end; gap: 8px; }
.log-actions button { padding: 6px 14px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
.log-actions .primary { background: #0369a1; }
.log-actions .primary:disabled { opacity: 0.5; cursor: not-allowed; }

.empty { padding: 32px; text-align: center; color: #64748b; font-size: 13px; background: #1e293b; border-radius: 6px; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }

.overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
.modal { background: #1e293b; padding: 24px; border-radius: 8px; min-width: 480px; max-width: 600px; border: 1px solid #334155; }
.modal h3 { margin: 0 0 16px 0; }
.row { display: flex; gap: 12px; }
.col { flex: 1; }
label { display: block; margin-bottom: 12px; font-size: 13px; color: #94a3b8; }
input { width: 100%; padding: 8px; margin-top: 4px; background: #0f172a; border: 1px solid #334155; color: #e2e8f0; border-radius: 4px; font-size: 14px; }
.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }
.actions button { padding: 8px 16px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; }
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }

.toast { position: fixed; bottom: 24px; right: 24px; background: #0369a1; color: white; padding: 12px 20px; border-radius: 6px; font-size: 13px; z-index: 200; box-shadow: 0 4px 12px rgba(0,0,0,0.3); }
</style>

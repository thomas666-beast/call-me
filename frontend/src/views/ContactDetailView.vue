<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getContact, type ContactWithCalls } from '../api/contacts'

const route = useRoute()
const router = useRouter()

const data = ref<ContactWithCalls | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)

async function load() {
  loading.value = true
  error.value = null
  const id = String(route.params.id ?? '')
  if (!id) {
    error.value = 'Invalid contact ID'
    loading.value = false
    return
  }
  try {
    data.value = await getContact(id)
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
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

function openCall(id: number) {
  router.push(`/admin/calls/${id}`)
}

onMounted(load)
</script>

<template>
  <div>
    <div class="header">
      <button class="back" @click="router.push('/admin/contacts')">← Back to contacts</button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
    <div v-if="loading" class="empty">Loading…</div>

    <div v-else-if="data">
      <h2>{{ data.contact.display_name }}</h2>

      <div class="card">
        <div class="grid">
          <div class="field">
            <span class="label">Phone</span>
            <span class="value mono">{{ data.contact.phone_number }}</span>
          </div>
          <div class="field">
            <span class="label">Email</span>
            <span class="value">{{ data.contact.email ?? '—' }}</span>
          </div>
          <div class="field">
            <span class="label">Company</span>
            <span class="value">{{ data.contact.company ?? '—' }}</span>
          </div>
          <div class="field">
            <span class="label">Tags</span>
            <span class="value">
              <span v-for="t in data.contact.tags" :key="t" class="tag">{{ t }}</span>
              <span v-if="data.contact.tags.length === 0">—</span>
            </span>
          </div>
        </div>

        <div v-if="data.contact.notes" class="notes">
          <span class="label">Notes</span>
          <p>{{ data.contact.notes }}</p>
        </div>
      </div>

      <h3>Call history ({{ data.calls.length }})</h3>
      <div v-if="data.calls.length === 0" class="empty">
        No calls with this contact yet.
      </div>
      <div v-else class="call-list">
        <div v-for="call in data.calls" :key="call.id" class="call">
          <div class="call-head">
            <span class="mono small">{{ formatDate(call.calldate) }}</span>
            <span class="mono">{{ call.src }}</span>
            <span class="arrow">→</span>
            <span class="mono">{{ call.dst }}</span>
            <span class="mono small">{{ formatDuration(call.billsec) }}</span>
            <span class="dispo small">{{ call.disposition }}</span>
            <button class="details-btn" @click="openCall(call.id)">Details</button>
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
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.header { margin-bottom: 16px; }
.back { padding: 6px 12px; background: transparent; color: #94a3b8; border: 1px solid #334155; border-radius: 4px; cursor: pointer; font-size: 13px; }
.back:hover { color: #e2e8f0; background: #1e293b; }

h2 { margin: 0 0 16px 0; }
h3 { margin: 24px 0 12px 0; color: #cbd5e1; font-size: 13px; text-transform: uppercase; letter-spacing: 0.5px; }

.card { background: #1e293b; border-radius: 6px; padding: 16px; border: 1px solid #334155; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 16px; }
.field { display: flex; flex-direction: column; gap: 4px; }
.label { font-size: 11px; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.5px; }
.value { color: #e2e8f0; font-size: 14px; }
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #94a3b8; }
.arrow { color: #64748b; }

.notes { margin-top: 20px; padding-top: 16px; border-top: 1px solid #334155; }
.notes p { margin: 6px 0 0 0; white-space: pre-wrap; color: #cbd5e1; line-height: 1.5; }

.tag { display: inline-block; padding: 1px 8px; margin-right: 4px; background: #1e3a8a; color: #93c5fd; border-radius: 10px; font-size: 11px; }

.call-list { display: flex; flex-direction: column; gap: 8px; }
.call { background: #1e293b; border: 1px solid #334155; border-radius: 4px; padding: 10px 12px; }
.call-head { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.dispo { color: #86efac; font-size: 12px; }
.details-btn { margin-left: auto; padding: 4px 12px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
.details-btn:hover { background: #475569; }

.note-list { margin-top: 10px; display: flex; flex-direction: column; gap: 6px; padding-left: 8px; border-left: 2px solid #1e3a8a; }
.note { background: #0f172a; border-radius: 4px; padding: 8px 10px; }
.note-head { display: flex; justify-content: space-between; font-size: 11px; color: #94a3b8; margin-bottom: 4px; }
.author { color: #38bdf8; font-weight: 500; }
.note-body { white-space: pre-wrap; color: #e2e8f0; font-size: 12px; line-height: 1.4; }

.empty { padding: 40px; text-align: center; color: #64748b; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }
</style>

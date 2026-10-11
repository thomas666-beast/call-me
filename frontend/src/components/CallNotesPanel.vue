<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { listCallNotes, createCallNote, type CallNote } from '../api/agent'

const props = defineProps<{
  uniqueid: string
}>()

const notes = ref<CallNote[]>([])
const newNote = ref('')
const loading = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)

async function load() {
  loading.value = true
  error.value = null
  try {
    notes.value = await listCallNotes(props.uniqueid)
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

async function save() {
  const text = newNote.value.trim()
  if (!text) return
  saving.value = true
  error.value = null
  try {
    const created = await createCallNote(props.uniqueid, text)
    notes.value.push(created)
    newNote.value = ''
  } catch (e: any) {
    error.value = e.message
  } finally {
    saving.value = false
  }
}

function formatTime(iso: string): string {
  return new Date(iso).toLocaleString('en-GB', { hour12: false })
}

onMounted(load)
watch(() => props.uniqueid, load)
</script>

<template>
  <div class="notes">
    <div v-if="loading" class="empty">Loading…</div>
    <div v-else-if="notes.length === 0" class="empty">No notes yet.</div>
    <div v-else class="list">
      <div v-for="n in notes" :key="n.id" class="note">
        <div class="head">
          <span class="author">{{ n.author_username }}</span>
          <span class="time">{{ formatTime(n.created_at) }}</span>
        </div>
        <div class="body">{{ n.note }}</div>
      </div>
    </div>

    <div class="composer">
      <textarea
        v-model="newNote"
        rows="2"
        placeholder="Add a note about this call…"
        @keydown.ctrl.enter="save"
      ></textarea>
      <button :disabled="saving || !newNote.trim()" @click="save">
        {{ saving ? 'Saving…' : 'Add note' }}
      </button>
    </div>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.notes { display: flex; flex-direction: column; gap: 12px; }
.empty { padding: 16px; text-align: center; color: #64748b; font-size: 13px; }
.list { max-height: 300px; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; }
.note {
  background: #1e293b; border-left: 3px solid #38bdf8;
  border-radius: 4px; padding: 8px 12px;
}
.head {
  display: flex; justify-content: space-between;
  font-size: 11px; color: #94a3b8;
}
.author { color: #38bdf8; font-weight: 500; }
.body {
  font-size: 13px; color: #e2e8f0; margin-top: 4px;
  white-space: pre-wrap; line-height: 1.4;
}
.composer { display: flex; flex-direction: column; gap: 8px; }
textarea {
  width: 100%; padding: 8px; background: #0f172a;
  border: 1px solid #334155; color: #e2e8f0;
  border-radius: 4px; font-family: inherit; font-size: 13px;
  resize: vertical;
}
button {
  align-self: flex-end;
  padding: 6px 14px; background: #0369a1; color: white;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
button:disabled { opacity: 0.5; cursor: not-allowed; }
button:not(:disabled):hover { background: #0284c7; }
.error { color: #f87171; font-size: 12px; margin: 0; }
</style>

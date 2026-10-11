<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  listExtensions,
  createExtension,
  updateExtension,
  deleteExtension,
  reloadAsterisk,
  type Extension,
  type ExtensionInput,
} from '../api/extensions'
import ExtensionForm from '../components/ExtensionForm.vue'

const extensions = ref<Extension[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)

const formOpen = ref(false)
const editing = ref<Extension | null>(null)

const deleteTarget = ref<Extension | null>(null)

const sorted = computed(() =>
  [...extensions.value].sort((a, b) => a.number.localeCompare(b.number))
)

async function load() {
  loading.value = true
  error.value = null
  try {
    extensions.value = await listExtensions()
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

function openEdit(ext: Extension) {
  editing.value = ext
  formOpen.value = true
}

function closeForm() {
  formOpen.value = false
  editing.value = null
}

async function handleSubmit(input: ExtensionInput) {
  error.value = null
  try {
    if (editing.value) {
      await updateExtension(editing.value.id, input)
      showToast(`Updated ${input.number}`)
    } else {
      await createExtension(input)
      showToast(`Created ${input.number}`)
    }
    closeForm()
    await load()
  } catch (e: any) {
    error.value = e.message
  }
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  const num = deleteTarget.value.number
  try {
    await deleteExtension(deleteTarget.value.id)
    deleteTarget.value = null
    showToast(`Deleted ${num}`)
    await load()
  } catch (e: any) {
    error.value = e.message
    deleteTarget.value = null
  }
}

async function handleReload() {
  try {
    await reloadAsterisk()
    showToast('Asterisk reloaded')
  } catch (e: any) {
    error.value = e.message
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
      <h2>Extensions</h2>
      <div class="actions">
        <button @click="handleReload">Reload Asterisk</button>
        <button class="primary" @click="openNew">New extension</button>
      </div>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="loading" class="empty">Loading…</div>

    <div v-else-if="sorted.length === 0" class="empty">
      No extensions yet. Click "New extension" to create one.
    </div>

    <table v-else>
      <thead>
        <tr>
          <th>Number</th>
          <th>Name</th>
          <th>Context</th>
          <th>Codecs</th>
          <th>Active</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="ext in sorted" :key="ext.id">
          <td class="mono">{{ ext.number }}</td>
          <td>{{ ext.display_name }}</td>
          <td class="mono">{{ ext.context }}</td>
          <td class="mono small">{{ ext.codecs.join(', ') }}</td>
          <td>
            <span class="badge" :class="ext.active ? 'on' : 'off'">
              {{ ext.active ? 'yes' : 'no' }}
            </span>
          </td>
          <td class="row-actions">
            <button @click="openEdit(ext)">Edit</button>
            <button class="danger" @click="deleteTarget = ext">Delete</button>
          </td>
        </tr>
      </tbody>
    </table>

    <ExtensionForm
      :open="formOpen"
      :editing="editing"
      @close="closeForm"
      @submit="handleSubmit"
    />

    <div v-if="deleteTarget" class="overlay" @click.self="deleteTarget = null">
      <div class="modal">
        <h3>Delete extension {{ deleteTarget.number }}?</h3>
        <p>This removes the endpoint from Asterisk. The softphone will stop working.</p>
        <div class="actions">
          <button @click="deleteTarget = null">Cancel</button>
          <button class="danger" @click="confirmDelete">Delete</button>
        </div>
      </div>
    </div>

    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<style scoped>
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.actions { display: flex; gap: 8px; }
.actions button { padding: 8px 14px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; }
.actions button:hover { background: #475569; }
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }
.actions .danger { background: #7f1d1d; }
.actions .danger:hover { background: #991b1b; }

table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align: left; padding: 10px 8px; border-bottom: 1px solid #1e293b; }
th { color: #94a3b8; font-weight: 500; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; }
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #94a3b8; }
.badge { display: inline-block; padding: 2px 8px; border-radius: 10px; font-size: 11px; }
.badge.on { background: #14532d; color: #86efac; }
.badge.off { background: #3f3f46; color: #a1a1aa; }
.row-actions { text-align: right; }
.row-actions button { padding: 4px 10px; margin-left: 6px; background: transparent; color: #94a3b8; border: 1px solid #334155; border-radius: 4px; cursor: pointer; font-size: 12px; }
.row-actions button:hover { background: #334155; color: #e2e8f0; }
.row-actions .danger:hover { background: #7f1d1d; border-color: #7f1d1d; color: #fecaca; }

.empty { padding: 40px; text-align: center; color: #64748b; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }

.overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
.modal { background: #1e293b; padding: 24px; border-radius: 8px; max-width: 420px; border: 1px solid #334155; }
.modal h3 { margin: 0 0 12px 0; }
.modal p { color: #94a3b8; font-size: 13px; margin-bottom: 20px; }

.toast {
  position: fixed; bottom: 24px; right: 24px;
  background: #0369a1; color: white; padding: 12px 20px;
  border-radius: 6px; font-size: 13px; z-index: 200;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
</style>

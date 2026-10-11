<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  listQueues,
  createQueue,
  updateQueue,
  deleteQueue,
  reloadAsterisk,
  type Queue,
  type QueueInput,
} from '../api/queues'
import QueueForm from '../components/QueueForm.vue'

const queues = ref<Queue[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)

const formOpen = ref(false)
const editing = ref<Queue | null>(null)
const deleteTarget = ref<Queue | null>(null)

const sorted = computed(() =>
  [...queues.value].sort((a, b) => a.number.localeCompare(b.number))
)

async function load() {
  loading.value = true
  error.value = null
  try {
    queues.value = await listQueues()
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

function openEdit(queue: Queue) {
  editing.value = queue
  formOpen.value = true
}

function closeForm() {
  formOpen.value = false
  editing.value = null
}

async function handleSubmit(input: QueueInput) {
  error.value = null
  try {
    if (editing.value) {
      await updateQueue(editing.value.id, input)
      showToast(`Updated queue ${input.number}`)
    } else {
      await createQueue(input)
      showToast(`Created queue ${input.number}`)
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
    await deleteQueue(deleteTarget.value.id)
    deleteTarget.value = null
    showToast(`Deleted queue ${num}`)
    await load()
  } catch (e: any) {
    error.value = e.message
    deleteTarget.value = null
  }
}

async function handleReload() {
  try {
    await reloadAsterisk()
    showToast('Queues reloaded')
  } catch (e: any) {
    error.value = e.message
  }
}

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => {
    toast.value = null
  }, 3000)
}

onMounted(load)
</script>

<template>
  <div>
    <div class="header">
      <h2>Queues</h2>
      <div class="actions">
        <button @click="handleReload">Reload Queues</button>
        <button class="primary" @click="openNew">New queue</button>
      </div>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="loading" class="empty">Loading…</div>

    <div v-else-if="sorted.length === 0" class="empty">
      No queues yet. Click "New queue" to create one.
    </div>

    <table v-else>
      <thead>
        <tr>
          <th>Number</th>
          <th>Name</th>
          <th>Strategy</th>
          <th>Timeout</th>
          <th>ACW</th>
          <th>Members</th>
          <th>Active</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="q in sorted" :key="q.id">
          <td class="mono">{{ q.number }}</td>
          <td>{{ q.name }}</td>
          <td class="mono small">{{ q.strategy }}</td>
          <td class="mono small">{{ q.timeout }}s</td>
          <td class="mono small">{{ q.acw_seconds }}s</td>
          <td class="mono small">
            {{ q.members.length ? q.members.join(', ') : '—' }}
          </td>
          <td>
            <span class="badge" :class="q.active ? 'on' : 'off'">
              {{ q.active ? 'yes' : 'no' }}
            </span>
          </td>
          <td class="row-actions">
            <button @click="openEdit(q)">Edit</button>
            <button class="danger" @click="deleteTarget = q">Delete</button>
          </td>
        </tr>
      </tbody>
    </table>

    <QueueForm
      :open="formOpen"
      :editing="editing"
      @close="closeForm"
      @submit="handleSubmit"
    />

    <div v-if="deleteTarget" class="overlay" @click.self="deleteTarget = null">
      <div class="modal">
        <h3>Delete queue {{ deleteTarget.number }}?</h3>
        <p>
          This removes the queue from Asterisk. Any callers waiting in it will
          be dropped.
        </p>
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
.header {
  display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;
}
.actions { display: flex; gap: 8px; }
.actions button {
  padding: 8px 14px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.actions button:hover { background: #475569; }
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }
.actions .danger { background: #7f1d1d; }
.actions .danger:hover { background: #991b1b; }

table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align: left; padding: 10px 8px; border-bottom: 1px solid #1e293b; }
th {
  color: #94a3b8; font-weight: 500; font-size: 12px;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #94a3b8; }
.badge {
  display: inline-block; padding: 2px 8px; border-radius: 10px; font-size: 11px;
}
.badge.on { background: #14532d; color: #86efac; }
.badge.off { background: #3f3f46; color: #a1a1aa; }
.row-actions { text-align: right; }
.row-actions button {
  padding: 4px 10px; margin-left: 6px; background: transparent;
  color: #94a3b8; border: 1px solid #334155; border-radius: 4px;
  cursor: pointer; font-size: 12px;
}
.row-actions button:hover { background: #334155; color: #e2e8f0; }
.row-actions .danger:hover { background: #7f1d1d; border-color: #7f1d1d; color: #fecaca; }

.empty { padding: 40px; text-align: center; color: #64748b; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }

.overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.modal {
  background: #1e293b; padding: 24px; border-radius: 8px;
  max-width: 420px; border: 1px solid #334155;
}
.modal h3 { margin: 0 0 12px 0; }
.modal p { color: #94a3b8; font-size: 13px; margin-bottom: 20px; }

.toast {
  position: fixed; bottom: 24px; right: 24px;
  background: #0369a1; color: white; padding: 12px 20px;
  border-radius: 6px; font-size: 13px; z-index: 200;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
</style>

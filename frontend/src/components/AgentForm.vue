<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { listQueues, type Queue } from '../api/queues'
import type { Agent, AgentInput, AgentUpdate } from '../api/agents'

const props = defineProps<{
  open: boolean
  editing: Agent | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (
    e: 'submit-create' | 'submit-update',
    payload: { id?: string; input: AgentInput | AgentUpdate },
  ): void
}>()

const queues = ref<Queue[]>([])
const queuesLoaded = ref(false)

// Create fields
const extensionNumber = ref('')
const displayName = ref('')
const sipSecret = ref('')
const password = ref('')

// Shared fields
const selectedQueues = ref<string[]>([])
const active = ref(true)

async function loadQueues() {
  if (queuesLoaded.value) return
  try {
    queues.value = await listQueues()
    queuesLoaded.value = true
  } catch (e) {
    console.error('Failed to load queues', e)
  }
}

watch(
  () => props.open,
  async (isOpen) => {
    if (!isOpen) return
    await loadQueues()

    if (props.editing) {
      extensionNumber.value = props.editing.extension_number ?? ''
      displayName.value = props.editing.display_name ?? ''
      sipSecret.value = '' // only used on submit if changed
      password.value = ''
      selectedQueues.value = [...props.editing.queue_numbers]
      active.value = props.editing.active
    } else {
      extensionNumber.value = ''
      displayName.value = ''
      sipSecret.value = ''
      password.value = ''
      selectedQueues.value = []
      active.value = true
    }
  }
)

function generateSecret(): string {
  const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  let s = ''
  for (let i = 0; i < 20; i++) s += chars[Math.floor(Math.random() * chars.length)]
  return s
}

function fillSipSecret() {
  sipSecret.value = generateSecret()
}

function fillPassword() {
  password.value = generateSecret()
}

function toggleQueue(number: string) {
  const idx = selectedQueues.value.indexOf(number)
  if (idx >= 0) selectedQueues.value.splice(idx, 1)
  else selectedQueues.value.push(number)
}

function submit() {
  if (props.editing) {
    const update: AgentUpdate = {
      display_name: displayName.value.trim(),
      queue_numbers: [...selectedQueues.value],
      active: active.value,
    }
    // Only send secrets if the admin typed new ones.
    if (sipSecret.value.trim()) update.sip_secret = sipSecret.value.trim()
    if (password.value.trim()) update.password = password.value.trim()

    emit('submit-update', { id: props.editing.id, input: update })
  } else {
    const create: AgentInput = {
      extension_number: extensionNumber.value.trim(),
      display_name: displayName.value.trim(),
      sip_secret: sipSecret.value,
      password: password.value,
      queue_numbers: [...selectedQueues.value],
      active: active.value,
    }
    emit('submit-create', { input: create })
  }
}

onMounted(() => {
  loadQueues()
})
</script>

<template>
  <div v-if="open" class="overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>{{ editing ? 'Edit agent' : 'New agent' }}</h3>

      <form @submit.prevent="submit">
        <div class="row">
          <label class="col">
            Extension number
            <input
              v-model="extensionNumber"
              placeholder="1001"
              :disabled="!!editing"
            />
            <span v-if="editing" class="hint">Extension cannot be changed.</span>
          </label>
          <label class="col">
            Display name
            <input v-model="displayName" placeholder="Alice Smith" />
          </label>
        </div>

        <div class="row">
          <label class="col">
            SIP secret {{ editing ? '(leave blank to keep)' : '' }}
            <div class="secret-row">
              <input
                v-model="sipSecret"
                :placeholder="editing ? '••••••••' : 'min 6 characters'"
              />
              <button type="button" @click="fillSipSecret">Generate</button>
            </div>
          </label>
        </div>

        <div class="row">
          <label class="col">
            Login password {{ editing ? '(leave blank to keep)' : '' }}
            <div class="secret-row">
              <input
                v-model="password"
                type="text"
                :placeholder="editing ? '••••••••' : 'min 6 characters'"
              />
              <button type="button" @click="fillPassword">Generate</button>
            </div>
          </label>
        </div>

        <fieldset>
          <legend>Queues</legend>
          <div v-if="queues.length === 0" class="empty-queues">
            No queues exist yet. Create one first.
          </div>
          <div v-else class="queue-list">
            <label
              v-for="q in queues"
              :key="q.id"
              class="queue-item"
            >
              <input
                type="checkbox"
                :checked="selectedQueues.includes(q.number)"
                @change="toggleQueue(q.number)"
              />
              <span class="mono">{{ q.number }}</span>
              <span class="qname">{{ q.name }}</span>
            </label>
          </div>
        </fieldset>

        <label class="checkbox">
          <input v-model="active" type="checkbox" />
          Active
        </label>

        <div class="actions">
          <button type="button" @click="emit('close')">Cancel</button>
          <button type="submit" class="primary">
            {{ editing ? 'Save' : 'Create' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.modal {
  background: #1e293b; padding: 24px; border-radius: 8px;
  min-width: 520px; max-width: 640px; max-height: 90vh; overflow-y: auto;
  border: 1px solid #334155;
}
h3 { margin: 0 0 16px 0; }
label { display: block; margin-bottom: 12px; font-size: 13px; color: #94a3b8; }
.hint { display: block; font-size: 11px; color: #64748b; margin-top: 2px; }
input {
  width: 100%; padding: 8px; margin-top: 4px;
  background: #0f172a; border: 1px solid #334155; color: #e2e8f0;
  border-radius: 4px; font-size: 14px;
}
input:disabled { opacity: 0.6; cursor: not-allowed; }
input[type="checkbox"] { width: auto; margin: 0; }
.row { display: flex; gap: 12px; }
.col { flex: 1; }
.secret-row { display: flex; gap: 8px; margin-top: 4px; }
.secret-row input { margin-top: 0; flex: 1; }
.secret-row button {
  padding: 8px 12px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer; font-size: 12px;
}
.secret-row button:hover { background: #475569; }

fieldset {
  border: 1px solid #334155; border-radius: 4px;
  padding: 12px; margin: 16px 0;
}
legend {
  padding: 0 6px; font-size: 12px; color: #94a3b8;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.queue-list { max-height: 200px; overflow-y: auto; }
.queue-item {
  display: flex; align-items: center; gap: 8px;
  padding: 4px 0; cursor: pointer;
}
.queue-item .mono { font-family: ui-monospace, monospace; min-width: 60px; }
.queue-item .qname { color: #cbd5e1; }
.empty-queues { color: #64748b; font-size: 13px; padding: 8px; text-align: center; }

.checkbox { display: flex; align-items: center; gap: 8px; }

.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }
.actions button {
  padding: 8px 16px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer;
}
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }
</style>

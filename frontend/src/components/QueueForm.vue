<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { STRATEGIES, type Queue, type QueueInput } from '../api/queues'
import { listExtensions, type Extension } from '../api/extensions'

const props = defineProps<{
  open: boolean
  editing: Queue | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'submit', input: QueueInput): void
}>()

const extensions = ref<Extension[]>([])
const extensionsLoaded = ref(false)

const number = ref('')
const name = ref('')
const strategy = ref<string>('ringall')
const timeout = ref(15)
const retry = ref(5)
const maxlen = ref(10)
const acwSeconds = ref(15)
const musicclass = ref('default')
const joinempty = ref(true)
const leavewhenempty = ref(false)
const ringinuse = ref(false)
const active = ref(true)
const selectedMembers = ref<string[]>([])

async function loadExtensions() {
  if (extensionsLoaded.value) return
  try {
    extensions.value = await listExtensions()
    extensionsLoaded.value = true
  } catch (e) {
    console.error('Failed to load extensions', e)
  }
}

watch(
  () => props.open,
  async (isOpen) => {
    if (!isOpen) return
    await loadExtensions()

    if (props.editing) {
      number.value = props.editing.number
      name.value = props.editing.name
      strategy.value = props.editing.strategy
      timeout.value = props.editing.timeout
      retry.value = props.editing.retry
      maxlen.value = props.editing.maxlen
      acwSeconds.value = props.editing.acw_seconds
      musicclass.value = props.editing.musicclass
      joinempty.value = props.editing.joinempty
      leavewhenempty.value = props.editing.leavewhenempty
      ringinuse.value = props.editing.ringinuse
      active.value = props.editing.active
      selectedMembers.value = [...props.editing.members]
    } else {
      number.value = ''
      name.value = ''
      strategy.value = 'ringall'
      timeout.value = 15
      retry.value = 5
      maxlen.value = 10
      acwSeconds.value = 15
      musicclass.value = 'default'
      joinempty.value = true
      leavewhenempty.value = false
      ringinuse.value = false
      active.value = true
      selectedMembers.value = []
    }
  }
)

onMounted(() => {
  // Preload extensions so the first modal open is instant.
  loadExtensions()
})

function toggleMember(num: string) {
  const idx = selectedMembers.value.indexOf(num)
  if (idx >= 0) selectedMembers.value.splice(idx, 1)
  else selectedMembers.value.push(num)
}

function submit() {
  emit('submit', {
    number: number.value.trim(),
    name: name.value.trim(),
    strategy: strategy.value,
    timeout: timeout.value,
    retry: retry.value,
    maxlen: maxlen.value,
    acw_seconds: acwSeconds.value,
    musicclass: musicclass.value.trim(),
    joinempty: joinempty.value,
    leavewhenempty: leavewhenempty.value,
    ringinuse: ringinuse.value,
    active: active.value,
    members: [...selectedMembers.value],
  })
}
</script>

<template>
  <div v-if="open" class="overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>{{ editing ? 'Edit queue' : 'New queue' }}</h3>

      <form @submit.prevent="submit">
        <div class="row">
          <label class="col">
            Number
            <input v-model="number" placeholder="2000" />
          </label>
          <label class="col">
            Name
            <input v-model="name" placeholder="Sales" />
          </label>
        </div>

        <label>
          Strategy
          <select v-model="strategy">
            <option v-for="s in STRATEGIES" :key="s" :value="s">{{ s }}</option>
          </select>
        </label>

        <div class="row">
          <label class="col">
            Timeout (s)
            <input v-model.number="timeout" type="number" min="1" max="300" />
          </label>
          <label class="col">
            Retry (s)
            <input v-model.number="retry" type="number" min="1" max="300" />
          </label>
          <label class="col">
            Max waiting
            <input v-model.number="maxlen" type="number" min="1" max="1000" />
          </label>
        </div>

        <div class="row">
          <label class="col">
            ACW (s)
            <input v-model.number="acwSeconds" type="number" min="0" max="600" />
          </label>
          <label class="col">
            Music class
            <input v-model="musicclass" placeholder="default" />
          </label>
        </div>

        <fieldset>
          <legend>Behavior</legend>
          <label class="checkbox">
            <input v-model="joinempty" type="checkbox" />
            Join empty (accept calls when no members available)
          </label>
          <label class="checkbox">
            <input v-model="leavewhenempty" type="checkbox" />
            Leave when empty (kick callers if all members leave)
          </label>
          <label class="checkbox">
            <input v-model="ringinuse" type="checkbox" />
            Ring in use (ring members already on a call)
          </label>
          <label class="checkbox">
            <input v-model="active" type="checkbox" />
            Active
          </label>
        </fieldset>

        <fieldset>
          <legend>Members</legend>
          <div v-if="extensions.length === 0" class="empty-members">
            No extensions available. Create extensions first.
          </div>
          <div v-else class="member-list">
            <label
              v-for="ext in extensions"
              :key="ext.id"
              class="checkbox member"
              :class="{ disabled: !ext.active }"
            >
              <input
                type="checkbox"
                :checked="selectedMembers.includes(ext.number)"
                :disabled="!ext.active"
                @change="toggleMember(ext.number)"
              />
              <span class="mono">{{ ext.number }}</span>
              <span class="name">{{ ext.display_name }}</span>
              <span v-if="!ext.active" class="inactive">(inactive)</span>
            </label>
          </div>
        </fieldset>

        <div class="actions">
          <button type="button" @click="emit('close')">Cancel</button>
          <button type="submit" class="primary">{{ editing ? 'Save' : 'Create' }}</button>
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
input, select {
  width: 100%; padding: 8px; margin-top: 4px;
  background: #0f172a; border: 1px solid #334155; color: #e2e8f0; border-radius: 4px;
}
input[type="number"] { width: 100%; }
.row { display: flex; gap: 12px; }
.col { flex: 1; }
fieldset {
  border: 1px solid #334155; border-radius: 4px;
  padding: 12px; margin: 16px 0;
}
legend { padding: 0 6px; font-size: 12px; color: #94a3b8; text-transform: uppercase; }
.checkbox { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.checkbox input { width: auto; margin: 0; }
.member-list { max-height: 200px; overflow-y: auto; }
.member { margin-bottom: 6px; }
.member .mono { font-family: ui-monospace, monospace; min-width: 60px; }
.member .name { color: #cbd5e1; flex: 1; }
.member.disabled { opacity: 0.5; }
.inactive { color: #f87171; font-size: 11px; }
.empty-members { color: #64748b; font-size: 13px; padding: 8px; text-align: center; }
.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }
.actions button {
  padding: 8px 16px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer;
}
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }
</style>

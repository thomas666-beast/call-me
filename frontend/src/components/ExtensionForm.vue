<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Extension, ExtensionInput } from '../api/extensions'

const props = defineProps<{
  open: boolean
  editing: Extension | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'submit', input: ExtensionInput): void
}>()

const availableCodecs = ['ulaw', 'alaw', 'g722', 'opus', 'gsm']

const number = ref('')
const displayName = ref('')
const secret = ref('')
const context = ref('from-internal')
const codecs = ref<string[]>(['ulaw', 'alaw'])
const maxContacts = ref(1)
const active = ref(true)

watch(
  () => props.open,
  (isOpen) => {
    if (!isOpen) return
    if (props.editing) {
      number.value = props.editing.number
      displayName.value = props.editing.display_name
      secret.value = props.editing.secret
      context.value = props.editing.context
      codecs.value = [...props.editing.codecs]
      maxContacts.value = props.editing.max_contacts
      active.value = props.editing.active
    } else {
      number.value = ''
      displayName.value = ''
      secret.value = ''
      context.value = 'from-internal'
      codecs.value = ['ulaw', 'alaw']
      maxContacts.value = 1
      active.value = true
    }
  }
)

function generateSecret() {
  const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  let s = ''
  for (let i = 0; i < 20; i++) s += chars[Math.floor(Math.random() * chars.length)]
  secret.value = s
}

function toggleCodec(c: string) {
  const idx = codecs.value.indexOf(c)
  if (idx >= 0) codecs.value.splice(idx, 1)
  else codecs.value.push(c)
}

function submit() {
  emit('submit', {
    number: number.value.trim(),
    display_name: displayName.value.trim(),
    secret: secret.value,
    context: context.value.trim(),
    codecs: codecs.value,
    max_contacts: maxContacts.value,
    active: active.value,
  })
}
</script>

<template>
  <div v-if="open" class="overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>{{ editing ? 'Edit extension' : 'New extension' }}</h3>

      <form @submit.prevent="submit">
        <label>
          Number
          <input v-model="number" placeholder="1001" />
        </label>

        <label>
          Display name
          <input v-model="displayName" placeholder="Alice Smith" />
        </label>

        <label>
          SIP secret
          <div class="secret-row">
            <input v-model="secret" placeholder="at least 6 characters" />
            <button type="button" @click="generateSecret">Generate</button>
          </div>
        </label>

        <label>
          Context
          <input v-model="context" placeholder="from-internal" />
        </label>

        <label>
          Codecs
          <div class="codecs">
            <button
              v-for="c in availableCodecs"
              :key="c"
              type="button"
              class="codec"
              :class="{ on: codecs.includes(c) }"
              @click="toggleCodec(c)"
            >
              {{ c }}
            </button>
          </div>
        </label>

        <label>
          Max contacts
          <input v-model.number="maxContacts" type="number" min="1" />
        </label>

        <label class="checkbox">
          <input v-model="active" type="checkbox" />
          Active
        </label>

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
  min-width: 420px; max-width: 520px; max-height: 90vh; overflow-y: auto;
  border: 1px solid #334155;
}
h3 { margin: 0 0 16px 0; }
label { display: block; margin-bottom: 12px; font-size: 13px; color: #94a3b8; }
input[type="text"], input:not([type]) {
  width: 100%; padding: 8px; margin-top: 4px;
  background: #0f172a; border: 1px solid #334155; color: #e2e8f0; border-radius: 4px;
}
input[type="number"] {
  width: 100px; padding: 8px; margin-top: 4px;
  background: #0f172a; border: 1px solid #334155; color: #e2e8f0; border-radius: 4px;
}
.secret-row { display: flex; gap: 8px; margin-top: 4px; }
.secret-row input { flex: 1; margin-top: 0; }
.secret-row button { padding: 8px 12px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; }
.codecs { display: flex; gap: 8px; margin-top: 4px; flex-wrap: wrap; }
.codec {
  padding: 6px 12px; background: #0f172a; color: #94a3b8;
  border: 1px solid #334155; border-radius: 4px; cursor: pointer; font-size: 12px;
}
.codec.on { background: #0369a1; color: white; border-color: #0369a1; }
.checkbox { display: flex; align-items: center; gap: 8px; }
.checkbox input { width: auto; margin: 0; }
.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }
.actions button { padding: 8px 16px; background: #334155; color: #e2e8f0; border: none; border-radius: 4px; cursor: pointer; }
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }
</style>

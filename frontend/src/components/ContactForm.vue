<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Contact, ContactInput } from '../api/contacts'

const props = defineProps<{
  open: boolean
  editing: Contact | null
  presetPhone?: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'submit', input: ContactInput): void
}>()

const phoneNumber = ref('')
const displayName = ref('')
const email = ref('')
const company = ref('')
const notes = ref('')
const tagsInput = ref('')

watch(
  () => props.open,
  (isOpen) => {
    if (!isOpen) return
    if (props.editing) {
      phoneNumber.value = props.editing.phone_number
      displayName.value = props.editing.display_name
      email.value = props.editing.email ?? ''
      company.value = props.editing.company ?? ''
      notes.value = props.editing.notes
      tagsInput.value = props.editing.tags.join(', ')
    } else {
      phoneNumber.value = props.presetPhone ?? ''
      displayName.value = ''
      email.value = ''
      company.value = ''
      notes.value = ''
      tagsInput.value = ''
    }
  }
)

function parseTags(input: string): string[] {
  return input
    .split(',')
    .map((t) => t.trim().toLowerCase())
    .filter((t) => t.length > 0)
}

function submit() {
  emit('submit', {
    phone_number: phoneNumber.value.trim(),
    display_name: displayName.value.trim(),
    email: email.value.trim() || null,
    company: company.value.trim() || null,
    notes: notes.value,
    tags: parseTags(tagsInput.value),
  })
}
</script>

<template>
  <div v-if="open" class="overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>{{ editing ? 'Edit contact' : 'New contact' }}</h3>
      <form @submit.prevent="submit">
        <div class="row">
          <label class="col">
            Phone number
            <input v-model="phoneNumber" placeholder="+15551234567" />
          </label>
          <label class="col">
            Display name
            <input v-model="displayName" placeholder="Acme Corp" />
          </label>
        </div>

        <div class="row">
          <label class="col">
            Email
            <input v-model="email" type="email" placeholder="contact@acme.com" />
          </label>
          <label class="col">
            Company
            <input v-model="company" placeholder="Acme" />
          </label>
        </div>

        <label>
          Tags (comma separated)
          <input v-model="tagsInput" placeholder="vip, sales" />
        </label>

        <label>
          Notes
          <textarea v-model="notes" rows="4" placeholder="Any notes about this contact…"></textarea>
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
  min-width: 520px; max-width: 640px; max-height: 90vh; overflow-y: auto;
  border: 1px solid #334155;
}
h3 { margin: 0 0 16px 0; }
label { display: block; margin-bottom: 12px; font-size: 13px; color: #94a3b8; }
input, textarea {
  width: 100%; padding: 8px; margin-top: 4px;
  background: #0f172a; border: 1px solid #334155; color: #e2e8f0;
  border-radius: 4px; font-family: inherit; font-size: 14px;
}
textarea { resize: vertical; }
.row { display: flex; gap: 12px; }
.col { flex: 1; }
.actions {
  display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px;
}
.actions button {
  padding: 8px 16px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer;
}
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }
</style>

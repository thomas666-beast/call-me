<script setup lang="ts">
import { ref, watch } from 'vue'
import { PANEL_ROLES, type User, type UserInput, type UserUpdate } from '../api/users'

const props = defineProps<{
  open: boolean
  editing: User | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'submit-create', input: UserInput): void
  (e: 'submit-update', id: string, input: UserUpdate): void
}>()

const username = ref('')
const email = ref('')
const password = ref('')
const role = ref<'admin' | 'supervisor'>('admin')
const active = ref(true)

watch(
  () => props.open,
  (isOpen) => {
    if (!isOpen) return
    if (props.editing) {
      username.value = props.editing.username
      email.value = props.editing.email ?? ''
      password.value = ''
      role.value = props.editing.role === 'supervisor' ? 'supervisor' : 'admin'
      active.value = props.editing.active
    } else {
      username.value = ''
      email.value = ''
      password.value = ''
      role.value = 'admin'
      active.value = true
    }
  }
)

function generatePassword() {
  const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
  let s = ''
  for (let i = 0; i < 16; i++) s += chars[Math.floor(Math.random() * chars.length)]
  password.value = s
}

function submit() {
  if (props.editing) {
    const update: UserUpdate = {
      email: email.value.trim() || null,
      active: active.value,
    }
    if (password.value.trim()) update.password = password.value.trim()
    emit('submit-update', props.editing.id, update)
  } else {
    emit('submit-create', {
      username: username.value.trim(),
      email: email.value.trim() || null,
      password: password.value,
      role: role.value,
      active: active.value,
    })
  }
}
</script>

<template>
  <div v-if="open" class="overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>{{ editing ? 'Edit user' : 'New user' }}</h3>

      <form @submit.prevent="submit">
        <label>
          Username
          <input
            v-model="username"
            placeholder="jdoe"
            :disabled="!!editing"
          />
          <span v-if="editing" class="hint">Username cannot be changed.</span>
          <span v-else class="hint">3–32 chars: letters, digits, dot, dash, underscore.</span>
        </label>

        <label>
          Email (optional)
          <input v-model="email" type="email" placeholder="jdoe@example.com" />
        </label>

        <div class="row">
          <label class="col">
            {{ editing ? 'New password (leave blank to keep current)' : 'Password' }}
            <div class="secret-row">
              <input
                v-model="password"
                type="text"
                :placeholder="editing ? '••••••••' : 'min 8 characters'"
              />
              <button type="button" @click="generatePassword">Generate</button>
            </div>
          </label>
        </div>

        <label v-if="!editing">
          Role
          <select v-model="role">
            <option v-for="r in PANEL_ROLES" :key="r.value" :value="r.value">
              {{ r.label }}
            </option>
          </select>
          <span class="hint">
            Only admins and supervisors can access the panel. Agents are
            created from the Agents page.
          </span>
        </label>

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
  min-width: 480px; max-width: 560px; max-height: 90vh; overflow-y: auto;
  border: 1px solid #334155;
}
h3 { margin: 0 0 16px 0; }
label { display: block; margin-bottom: 12px; font-size: 13px; color: #94a3b8; }
.hint { display: block; font-size: 11px; color: #64748b; margin-top: 3px; }
input, select {
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
.checkbox { display: flex; align-items: center; gap: 8px; }
.actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }
.actions button {
  padding: 8px 16px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer;
}
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }
</style>

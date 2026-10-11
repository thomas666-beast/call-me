<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  listUsers,
  createUser,
  updateUser,
  deleteUser,
  type User,
  type UserInput,
  type UserUpdate,
} from '../api/users'
import UserForm from '../components/UserForm.vue'

const users = ref<User[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)

const formOpen = ref(false)
const editing = ref<User | null>(null)
const deleteTarget = ref<User | null>(null)

async function load() {
  loading.value = true
  error.value = null
  try {
    users.value = await listUsers()
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

function openEdit(u: User) {
  editing.value = u
  formOpen.value = true
}

function closeForm() {
  formOpen.value = false
  editing.value = null
}

async function handleCreate(input: UserInput) {
  error.value = null
  try {
    const created = await createUser(input)
    showToast(`Created ${created.username}`)
    closeForm()
    await load()
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleUpdate(id: string, input: UserUpdate) {
  error.value = null
  try {
    const updated = await updateUser(id, input)
    showToast(`Updated ${updated.username}`)
    closeForm()
    await load()
  } catch (e: any) {
    error.value = e.message
  }
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  const username = deleteTarget.value.username
  try {
    await deleteUser(deleteTarget.value.id)
    deleteTarget.value = null
    showToast(`Deactivated ${username}`)
    await load()
  } catch (e: any) {
    error.value = e.message
    deleteTarget.value = null
  }
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString('en-GB', { hour12: false })
}

function roleBadge(role: string): string {
  return role === 'admin' ? 'admin' : 'supervisor'
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
      <div>
        <h2>Users</h2>
        <p class="subtitle">Panel accounts: admins and supervisors.</p>
      </div>
      <button class="primary" @click="openNew">New user</button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="loading" class="empty">Loading…</div>
    <div v-else-if="users.length === 0" class="empty">
      No panel users yet. Click "New user" to create one.
    </div>

    <table v-else>
      <thead>
        <tr>
          <th>Username</th>
          <th>Role</th>
          <th>Email</th>
          <th>Created</th>
          <th>Status</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="u in users" :key="u.id" :class="{ inactive: !u.active }">
          <td class="mono">{{ u.username }}</td>
          <td>
            <span class="role-badge" :class="roleBadge(u.role)">
              {{ u.role }}
            </span>
          </td>
          <td class="small">{{ u.email ?? '—' }}</td>
          <td class="mono small">{{ formatDate(u.created_at) }}</td>
          <td>
            <span class="badge" :class="u.active ? 'on' : 'off'">
              {{ u.active ? 'active' : 'inactive' }}
            </span>
          </td>
          <td class="row-actions">
            <button @click="openEdit(u)">Edit</button>
            <button
              v-if="u.active"
              class="danger"
              @click="deleteTarget = u"
            >
              Deactivate
            </button>
          </td>
        </tr>
      </tbody>
    </table>

    <UserForm
      :open="formOpen"
      :editing="editing"
      @close="closeForm"
      @submit-create="handleCreate"
      @submit-update="handleUpdate"
    />

    <div v-if="deleteTarget" class="overlay" @click.self="deleteTarget = null">
      <div class="modal">
        <h3>Deactivate {{ deleteTarget.username }}?</h3>
        <p>
          The user can no longer log in. History is preserved. You can
          reactivate them later by editing and checking "Active".
        </p>
        <div class="actions">
          <button @click="deleteTarget = null">Cancel</button>
          <button class="danger" @click="confirmDelete">Deactivate</button>
        </div>
      </div>
    </div>

    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<style scoped>
.header {
  display: flex; justify-content: space-between; align-items: flex-start;
  margin-bottom: 20px;
}
h2 { margin: 0; }
.subtitle { color: #64748b; font-size: 13px; margin: 4px 0 0 0; }
.header .primary {
  padding: 8px 14px; background: #0369a1; color: white;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.header .primary:hover { background: #0284c7; }

table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align: left; padding: 10px 8px; border-bottom: 1px solid #1e293b; }
th {
  color: #94a3b8; font-weight: 500; font-size: 12px;
  text-transform: uppercase; letter-spacing: 0.5px;
}
tr.inactive { opacity: 0.5; }
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #94a3b8; }

.role-badge {
  display: inline-block; padding: 2px 10px; border-radius: 10px;
  font-size: 11px; font-weight: 500;
}
.role-badge.admin      { background: #1e3a8a; color: #93c5fd; }
.role-badge.supervisor { background: #581c87; color: #d8b4fe; }

.badge {
  display: inline-block; padding: 2px 8px; border-radius: 10px; font-size: 11px;
}
.badge.on  { background: #14532d; color: #86efac; }
.badge.off { background: #3f3f46; color: #a1a1aa; }

.row-actions { text-align: right; }
.row-actions button {
  padding: 4px 10px; margin-left: 6px; background: transparent;
  color: #94a3b8; border: 1px solid #334155; border-radius: 4px;
  cursor: pointer; font-size: 12px;
}
.row-actions button:hover { background: #334155; color: #e2e8f0; }
.row-actions .danger:hover {
  background: #7f1d1d; border-color: #7f1d1d; color: #fecaca;
}

.empty { padding: 40px; text-align: center; color: #64748b; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }

.overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.modal {
  background: #1e293b; padding: 24px; border-radius: 8px;
  max-width: 440px; border: 1px solid #334155;
}
.modal h3 { margin: 0 0 12px 0; }
.modal p { color: #94a3b8; font-size: 13px; margin-bottom: 20px; line-height: 1.5; }
.modal .actions { display: flex; justify-content: flex-end; gap: 8px; }
.modal button {
  padding: 8px 16px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer;
}
.modal .danger { background: #7f1d1d; }
.modal .danger:hover { background: #991b1b; }

.toast {
  position: fixed; bottom: 24px; right: 24px;
  background: #0369a1; color: white; padding: 12px 20px;
  border-radius: 6px; font-size: 13px; z-index: 200;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
</style>

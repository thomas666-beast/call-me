<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  listContacts,
  createContact,
  updateContact,
  deleteContact,
  type Contact,
  type ContactInput,
} from '../api/contacts'
import { useDirectoryStore } from '../stores/directory'
import ContactForm from '../components/ContactForm.vue'
import Pagination from '../components/Pagination.vue'

const PAGE_SIZE = 20

const router = useRouter()
const directory = useDirectoryStore()

const contacts = ref<Contact[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)

const search = ref('')
const tagFilter = ref('')
const offset = ref(0)

const formOpen = ref(false)
const editing = ref<Contact | null>(null)
const deleteTarget = ref<Contact | null>(null)

async function load(resetOffset = false) {
  if (resetOffset) offset.value = 0
  loading.value = true
  error.value = null
  try {
    contacts.value = await listContacts({
      search: search.value || undefined,
      tag: tagFilter.value || undefined,
      limit: PAGE_SIZE,
      offset: offset.value,
    })
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

function nextPage() {
  offset.value += PAGE_SIZE
  load(false)
}

function prevPage() {
  offset.value = Math.max(0, offset.value - PAGE_SIZE)
  load(false)
}

function openNew() {
  editing.value = null
  formOpen.value = true
}

function openEdit(c: Contact) {
  editing.value = c
  formOpen.value = true
}

function closeForm() {
  formOpen.value = false
  editing.value = null
}

async function handleSubmit(input: ContactInput) {
  error.value = null
  try {
    if (editing.value) {
      await updateContact(editing.value.id, input)
      showToast(`Updated ${input.display_name}`)
    } else {
      await createContact(input)
      showToast(`Created ${input.display_name}`)
    }
    closeForm()
    await directory.refresh()
    await load()
  } catch (e: any) {
    error.value = e.message
  }
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  const name = deleteTarget.value.display_name
  try {
    await deleteContact(deleteTarget.value.id)
    deleteTarget.value = null
    showToast(`Deleted ${name}`)
    await directory.refresh()
    await load()
  } catch (e: any) {
    error.value = e.message
    deleteTarget.value = null
  }
}

function openContact(c: Contact) {
  router.push(`/admin/contacts/${c.id}`)
}

function clearSearch() {
  search.value = ''
  tagFilter.value = ''
  load(true)
}

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => { toast.value = null }, 3000)
}

onMounted(() => load())
</script>

<template>
  <div>
    <div class="header">
      <h2>Contacts</h2>
      <button class="primary" @click="openNew">New contact</button>
    </div>

    <div class="filters">
      <input
        v-model="search"
        placeholder="Search name, phone, company…"
        @keyup.enter="load(true)"
      />
      <input
        v-model="tagFilter"
        placeholder="Tag"
        @keyup.enter="load(true)"
      />
      <button class="primary" @click="load(true)">Search</button>
      <button v-if="search || tagFilter" @click="clearSearch">Clear</button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="loading" class="empty">Loading…</div>
    <div v-else-if="contacts.length === 0" class="empty">
      {{ search || tagFilter ? 'No contacts match your search.' : 'No contacts yet. Click "New contact" to create one.' }}
    </div>

    <table v-else>
      <thead>
        <tr>
          <th>Name</th>
          <th>Phone</th>
          <th>Company</th>
          <th>Tags</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="c in contacts" :key="c.id">
          <td>
            <a href="#" @click.prevent="openContact(c)" class="link">
              {{ c.display_name }}
            </a>
          </td>
          <td class="mono small">{{ c.phone_number }}</td>
          <td class="small">{{ c.company ?? '—' }}</td>
          <td>
            <span v-for="t in c.tags" :key="t" class="tag">{{ t }}</span>
            <span v-if="c.tags.length === 0" class="muted">—</span>
          </td>
          <td class="row-actions">
            <button @click="openEdit(c)">Edit</button>
            <button class="danger" @click="deleteTarget = c">Delete</button>
          </td>
        </tr>
      </tbody>
    </table>

    <Pagination
      v-if="contacts.length > 0 || offset > 0"
      :offset="offset"
      :limit="PAGE_SIZE"
      :count="contacts.length"
      @prev="prevPage"
      @next="nextPage"
    />

    <ContactForm
      :open="formOpen"
      :editing="editing"
      @close="closeForm"
      @submit="handleSubmit"
    />

    <div v-if="deleteTarget" class="overlay" @click.self="deleteTarget = null">
      <div class="modal">
        <h3>Delete {{ deleteTarget.display_name }}?</h3>
        <p>This removes the contact from the directory. Call records remain.</p>
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
  display: flex; justify-content: space-between; align-items: center;
  margin-bottom: 16px;
}
h2 { margin: 0; }
.header .primary {
  padding: 8px 14px; background: #0369a1; color: white;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.header .primary:hover { background: #0284c7; }

.filters {
  display: flex; gap: 8px; margin-bottom: 16px; flex-wrap: wrap;
}
.filters input {
  padding: 8px 10px; background: #1e293b; border: 1px solid #334155;
  color: #e2e8f0; border-radius: 4px; font-size: 13px; min-width: 220px;
}
.filters button {
  padding: 8px 14px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.filters .primary { background: #0369a1; }
.filters .primary:hover { background: #0284c7; }

table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align: left; padding: 10px 8px; border-bottom: 1px solid #1e293b; }
th {
  color: #94a3b8; font-weight: 500; font-size: 12px;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #94a3b8; }
.muted { color: #475569; }
.link { color: #38bdf8; text-decoration: none; }
.link:hover { text-decoration: underline; }

.tag {
  display: inline-block;
  padding: 1px 8px;
  margin-right: 4px;
  background: #1e3a8a;
  color: #93c5fd;
  border-radius: 10px;
  font-size: 11px;
}

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

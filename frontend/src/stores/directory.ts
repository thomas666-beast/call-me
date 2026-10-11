import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listExtensions } from '../api/extensions'
import { listContacts, type Contact } from '../api/contacts'
import { listAgentContacts, listAllQueues, listAllExtensions } from '../api/agent'
import { listQueues } from '../api/queues'
import { useAuthStore } from './auth'

export const useDirectoryStore = defineStore('directory', () => {
  const extensionNames = ref<Record<string, string>>({})
  const queueNames = ref<Record<string, string>>({})
  const contactsByPhone = ref<Record<string, Contact>>({})
  const loading = ref(false)
  const loaded = ref(false)

  async function tryLoadExtensions(isAgent: boolean) {
    try {
      const exts = isAgent
        ? await listAllExtensions()
        : await listExtensions()
      const map: Record<string, string> = {}
      for (const e of exts) map[e.number] = e.display_name
      extensionNames.value = map
    } catch {
      extensionNames.value = {}
    }
  }

  async function tryLoadQueues(isAgent: boolean) {
    try {
      const qs = isAgent ? await listAllQueues() : await listQueues()
      const map: Record<string, string> = {}
      for (const q of qs) map[q.number] = q.name
      queueNames.value = map
    } catch {
      queueNames.value = {}
    }
  }

  async function tryLoadContacts() {
    try {
      const contacts = await listContacts({ limit: 500 })
      const map: Record<string, Contact> = {}
      for (const c of contacts) map[c.phone_number] = c
      contactsByPhone.value = map
      return
    } catch {
      // fall through
    }
    try {
      const contacts = await listAgentContacts()
      const map: Record<string, Contact> = {}
      for (const c of contacts) map[c.phone_number] = c
      contactsByPhone.value = map
    } catch {
      contactsByPhone.value = {}
    }
  }

  async function ensureLoaded() {
    if (loaded.value || loading.value) return
    const auth = useAuthStore()
    const isAgent = auth.user?.role === 'agent'

    loading.value = true
    try {
      await Promise.all([
        tryLoadExtensions(isAgent),
        tryLoadQueues(isAgent),
        tryLoadContacts(),
      ])
      loaded.value = true
    } finally {
      loading.value = false
    }
  }

  function nameFor(number: string): string | null {
    if (!number) return null
    if (extensionNames.value[number]) return extensionNames.value[number]
    if (queueNames.value[number]) return queueNames.value[number]
    const contact = contactsByPhone.value[number]
    if (contact) return contact.display_name
    return null
  }

  function contactFor(number: string): Contact | null {
    if (!number) return null
    return contactsByPhone.value[number] ?? null
  }

  async function refresh() {
    loaded.value = false
    await ensureLoaded()
  }

  return {
    extensionNames,
    queueNames,
    contactsByPhone,
    loading,
    loaded,
    ensureLoaded,
    refresh,
    nameFor,
    contactFor,
  }
})

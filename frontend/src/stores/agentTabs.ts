import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  lookupAgentContact,
  getAgentContact,
  listCallNotes,
  type CallNote,
} from '../api/agent'
import type { ContactWithCalls } from '../api/contacts'

export interface AgentTab {
  id: string
  phoneNumber: string
  contactId: string | null
  contact: ContactWithCalls | null
  callid: string | null
  linkedid: string | null
  queueNumber: string | null
  logDraft: string
  savedLog: string
  notes: CallNote[]
  loading: boolean
  error: string | null
  unseen: boolean
}

let idCounter = 0
function nextId(): string {
  idCounter += 1
  return `tab-${Date.now()}-${idCounter}`
}

export const useAgentTabsStore = defineStore('agentTabs', () => {
  const tabs = ref<AgentTab[]>([])
  const activeTabId = ref<string | null>(null)

  const activeTab = computed(
    () => tabs.value.find((t) => t.id === activeTabId.value) ?? null
  )
  const unseenCount = computed(() => tabs.value.filter((t) => t.unseen).length)

  function findTabByPhone(phone: string): AgentTab | undefined {
    return tabs.value.find((t) => t.phoneNumber === phone)
  }

  function findTabByCallid(callid: string): AgentTab | undefined {
    return tabs.value.find((t) => t.callid === callid)
  }

  async function loadTabData(tabId: string) {
    const tab = tabs.value.find((t) => t.id === tabId)
    if (!tab) return

    tab.loading = true
    tab.error = null
    try {
      const contact = await lookupAgentContact(tab.phoneNumber)
      if (contact) {
        tab.contactId = contact.id
        const full = await getAgentContact(contact.id)
        const t2 = tabs.value.find((t) => t.id === tabId)
        if (t2) t2.contact = full
      } else {
        const t2 = tabs.value.find((t) => t.id === tabId)
        if (t2) {
          t2.contactId = null
          t2.contact = null
        }
      }
      if (tab.callid) {
        const notes = await listCallNotes(tab.callid)
        const t3 = tabs.value.find((t) => t.id === tabId)
        if (t3) t3.notes = notes
      }
    } catch (e: any) {
      const t4 = tabs.value.find((t) => t.id === tabId)
      if (t4) t4.error = e.message ?? String(e)
    } finally {
      const t5 = tabs.value.find((t) => t.id === tabId)
      if (t5) t5.loading = false
    }
  }

  async function handleActiveCall(params: {
    callerNumber: string
    callid: string
    linkedid: string | null
    queueNumber: string
  }) {
    const existing = findTabByPhone(params.callerNumber)
    if (existing) {
      if (existing.callid !== params.callid) {
        existing.callid = params.callid
        existing.linkedid = params.linkedid
        existing.queueNumber = params.queueNumber
        existing.logDraft = ''
        existing.savedLog = ''
        existing.notes = []
        existing.unseen = true
        await loadTabData(existing.id)
      }
      activeTabId.value = existing.id
      return existing
    }

    const id = nextId()
    tabs.value.push({
      id,
      phoneNumber: params.callerNumber,
      contactId: null,
      contact: null,
      callid: params.callid,
      linkedid: params.linkedid,
      queueNumber: params.queueNumber,
      logDraft: '',
      savedLog: '',
      notes: [],
      loading: true,
      error: null,
      unseen: true,
    })
    activeTabId.value = id
    await loadTabData(id)
  }

  async function openContactTab(phoneNumber: string) {
    const existing = findTabByPhone(phoneNumber)
    if (existing) {
      activeTabId.value = existing.id
      return
    }
    const id = nextId()
    tabs.value.push({
      id,
      phoneNumber,
      contactId: null,
      contact: null,
      callid: null,
      linkedid: null,
      queueNumber: null,
      logDraft: '',
      savedLog: '',
      notes: [],
      loading: true,
      error: null,
      unseen: false,
    })
    activeTabId.value = id
    await loadTabData(id)
  }

  function setActive(id: string) {
    activeTabId.value = id
  }

  function updateDraft(id: string, draft: string) {
    const tab = tabs.value.find((t) => t.id === id)
    if (tab) tab.logDraft = draft
  }

  function markSaved(id: string) {
    const tab = tabs.value.find((t) => t.id === id)
    if (tab) tab.savedLog = tab.logDraft
  }

  function appendNote(id: string, note: CallNote) {
    const tab = tabs.value.find((t) => t.id === id)
    if (tab) tab.notes.push(note)
  }

  function refreshContact(id: string, contact: ContactWithCalls) {
    const tab = tabs.value.find((t) => t.id === id)
    if (tab) {
      tab.contact = contact
      tab.contactId = contact.contact.id
    }
  }

  function closeTab(id: string) {
    const idx = tabs.value.findIndex((t) => t.id === id)
    if (idx < 0) return
    tabs.value.splice(idx, 1)
    if (activeTabId.value === id) {
      activeTabId.value =
        tabs.value.length > 0
          ? tabs.value[Math.min(idx, tabs.value.length - 1)].id
          : null
    }
  }

  function clearUnseen() {
    for (const t of tabs.value) t.unseen = false
  }

  function isDirty(id: string): boolean {
    const tab = tabs.value.find((t) => t.id === id)
    if (!tab) return false
    return tab.logDraft.trim() !== tab.savedLog.trim() && tab.logDraft.trim() !== ''
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    unseenCount,
    findTabByPhone,
    findTabByCallid,
    handleActiveCall,
    openContactTab,
    setActive,
    updateDraft,
    markSaved,
    appendNote,
    refreshContact,
    closeTab,
    clearUnseen,
    isDirty,
  }
})

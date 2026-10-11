import { apiFetch } from './client'
import type { Cdr } from './cdr'
import type { Contact, ContactInput, ContactWithCalls } from './contacts'

export interface AgentMe {
  username: string
  role: string
  extension: string | null
}

export interface AgentQueue {
  number: string
  name: string
  paused: boolean
  pause_reason: string | null
}

export interface ActiveCall {
  callid: string
  linkedid: string | null
  queue_number: string
  caller_number: string
  connected_at: string
}

export interface CallNote {
  id: string
  call_uniqueid: string
  call_linkedid: string
  author_id: string
  author_username: string
  note: string
  created_at: string
  updated_at: string
}

export const PAUSE_REASONS = [
  { value: 'break',    label: 'Break' },
  { value: 'lunch',    label: 'Lunch' },
  { value: 'meeting',  label: 'Meeting' },
  { value: 'training', label: 'Training' },
  { value: 'wrap-up',  label: 'Wrap-up' },
  { value: 'other',    label: 'Other' },
] as const

export interface ExtensionLookup {
  number: string
  display_name: string
}

export async function listAllExtensions(): Promise<ExtensionLookup[]> {
  const res = await apiFetch('/api/agent/extensions')
  if (!res.ok) throw new Error(`Failed to load extensions (${res.status})`)
  return res.json()
}

// -----------------------------------------------------------------------------
// Agent state
// -----------------------------------------------------------------------------

export async function getMe(): Promise<AgentMe> {
  const res = await apiFetch('/api/agent/me')
  if (!res.ok) throw new Error(`Failed to load agent info (${res.status})`)
  return res.json()
}

export async function getMyQueues(): Promise<AgentQueue[]> {
  const res = await apiFetch('/api/agent/me/queues')
  if (!res.ok) throw new Error(`Failed to load queues (${res.status})`)
  return res.json()
}

export async function pauseInQueue(queue: string, reason: string): Promise<AgentQueue> {
  const res = await apiFetch(`/api/agent/me/queues/${queue}/pause`, {
    method: 'POST',
    body: JSON.stringify({ reason }),
  })
  if (!res.ok) throw new Error(`Failed to pause (${res.status})`)
  return res.json()
}

export async function unpauseInQueue(queue: string): Promise<AgentQueue> {
  const res = await apiFetch(`/api/agent/me/queues/${queue}/unpause`, {
    method: 'POST',
  })
  if (!res.ok) throw new Error(`Failed to resume (${res.status})`)
  return res.json()
}

export async function getMyCalls(limit = 20, offset = 0): Promise<Cdr[]> {
  const res = await apiFetch(`/api/agent/me/calls?limit=${limit}&offset=${offset}`)
  if (!res.ok) throw new Error(`Failed to load calls (${res.status})`)
  return res.json()
}

export async function getActiveCall(): Promise<ActiveCall | null> {
  const res = await apiFetch('/api/agent/me/active')
  if (!res.ok) throw new Error(`Failed to load active call (${res.status})`)
  return res.json()
}

// -----------------------------------------------------------------------------
// Call notes
// -----------------------------------------------------------------------------

export async function listCallNotes(uniqueid: string): Promise<CallNote[]> {
  const res = await apiFetch(`/api/agent/me/calls/${encodeURIComponent(uniqueid)}/notes`)
  if (!res.ok) throw new Error(`Failed to load notes (${res.status})`)
  return res.json()
}

export async function createCallNote(uniqueid: string, note: string): Promise<CallNote> {
  const res = await apiFetch(`/api/agent/me/calls/${encodeURIComponent(uniqueid)}/notes`, {
    method: 'POST',
    body: JSON.stringify({ note }),
  })
  if (!res.ok) throw new Error(`Failed to save note (${res.status})`)
  return res.json()
}

// -----------------------------------------------------------------------------
// Contacts (agent-scoped: read + create only)
// -----------------------------------------------------------------------------

export async function listAgentContacts(
  search?: string,
  limit = 20,
  offset = 0,
): Promise<Contact[]> {
  const params = new URLSearchParams()
  if (search) params.set('search', search)
  params.set('limit', String(limit))
  params.set('offset', String(offset))
  const res = await apiFetch(`/api/agent/contacts?${params.toString()}`)
  if (!res.ok) throw new Error(`Failed to load contacts (${res.status})`)
  return res.json()
}

export async function lookupAgentContact(number: string): Promise<Contact | null> {
  const res = await apiFetch(`/api/agent/contacts/lookup/${encodeURIComponent(number)}`)
  if (!res.ok) throw new Error(`Lookup failed (${res.status})`)
  return res.json()
}

export async function getAgentContact(id: string): Promise<ContactWithCalls> {
  const res = await apiFetch(`/api/agent/contacts/${id}`)
  if (!res.ok) throw new Error(`Failed to load contact (${res.status})`)
  return res.json()
}

export async function createAgentContact(input: ContactInput): Promise<Contact> {
  const res = await apiFetch('/api/agent/contacts', {
    method: 'POST',
    body: JSON.stringify(input),
  })
  if (!res.ok) {
    if (res.status === 409) throw new Error('A contact with that number already exists.')
    if (res.status === 400) throw new Error('Invalid input — check phone number and display name.')
    throw new Error(`Failed to create contact (${res.status})`)
  }
  return res.json()
}

export interface QueueLookup {
  number: string
  name: string
}

export async function listAllQueues(): Promise<QueueLookup[]> {
  const res = await apiFetch('/api/agent/queues')
  if (!res.ok) throw new Error(`Failed to load queues (${res.status})`)
  return res.json()
}

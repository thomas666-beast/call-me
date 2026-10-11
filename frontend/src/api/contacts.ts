import { apiFetch } from './client'
import type { CallNote } from './agent'

export interface Contact {
  id: string
  phone_number: string
  display_name: string
  email: string | null
  company: string | null
  notes: string
  tags: string[]
  created_by: string | null
  created_at: string
  updated_at: string
}

export interface ContactInput {
  phone_number: string
  display_name: string
  email?: string | null
  company?: string | null
  notes?: string
  tags?: string[]
}

export interface ContactCallSummary {
  id: number
  calldate: string
  src: string
  dst: string
  duration: number
  billsec: number
  disposition: string
  uniqueid: string
  linkedid: string
  notes: CallNote[]
}

export interface ContactWithCalls {
  contact: Contact
  calls: ContactCallSummary[]
}

export interface ContactFilters {
  search?: string
  tag?: string
  limit?: number
  offset?: number
}

export async function listContacts(filters: ContactFilters = {}): Promise<Contact[]> {
  const params = new URLSearchParams()
  if (filters.search) params.set('search', filters.search)
  if (filters.tag) params.set('tag', filters.tag)
  params.set('limit', String(filters.limit ?? 20))
  params.set('offset', String(filters.offset ?? 0))

  const res = await apiFetch(`/api/admin/contacts?${params.toString()}`)
  if (!res.ok) throw new Error(`Failed to load contacts (${res.status})`)
  return res.json()
}

export async function getContact(id: string): Promise<ContactWithCalls> {
  const res = await apiFetch(`/api/admin/contacts/${id}`)
  if (!res.ok) throw new Error(`Failed to load contact (${res.status})`)
  return res.json()
}

export async function createContact(input: ContactInput): Promise<Contact> {
  const res = await apiFetch('/api/admin/contacts', {
    method: 'POST',
    body: JSON.stringify(input),
  })
  if (!res.ok) throw new Error(await errorMessage(res))
  return res.json()
}

export async function updateContact(id: string, input: ContactInput): Promise<Contact> {
  const res = await apiFetch(`/api/admin/contacts/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
  if (!res.ok) throw new Error(await errorMessage(res))
  return res.json()
}

export async function deleteContact(id: string): Promise<void> {
  const res = await apiFetch(`/api/admin/contacts/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(`Failed to delete (${res.status})`)
}

async function errorMessage(res: Response): Promise<string> {
  switch (res.status) {
    case 400: return 'Invalid input — check phone number, display name, and email format.'
    case 403: return 'You do not have permission to do this.'
    case 404: return 'Contact not found.'
    case 409: return 'A contact with that phone number already exists.'
    case 500: return 'Server error. Check the backend log.'
    default:  return `Request failed (${res.status})`
  }
}

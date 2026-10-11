import { apiFetch } from './client'

export interface Extension {
  id: string
  number: string
  display_name: string
  secret: string
  context: string
  transport: string
  codecs: string[]
  max_contacts: number
  active: boolean
}

export interface ExtensionInput {
  number: string
  display_name: string
  secret: string
  context?: string
  transport?: string
  codecs?: string[]
  max_contacts?: number
  active?: boolean
}

export async function listExtensions(): Promise<Extension[]> {
  const res = await apiFetch('/api/admin/extensions')
  if (!res.ok) throw new Error(`Failed to list extensions (${res.status})`)
  return res.json()
}

export async function createExtension(input: ExtensionInput): Promise<Extension> {
  const res = await apiFetch('/api/admin/extensions', {
    method: 'POST',
    body: JSON.stringify(input),
  })
  if (!res.ok) {
    const msg = await errorMessage(res)
    throw new Error(msg)
  }
  return res.json()
}

export async function updateExtension(id: string, input: ExtensionInput): Promise<Extension> {
  const res = await apiFetch(`/api/admin/extensions/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
  if (!res.ok) {
    const msg = await errorMessage(res)
    throw new Error(msg)
  }
  return res.json()
}

export async function deleteExtension(id: string): Promise<void> {
  const res = await apiFetch(`/api/admin/extensions/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(`Failed to delete (${res.status})`)
}

export async function reloadAsterisk(): Promise<void> {
  const res = await apiFetch('/api/admin/extensions/reload', { method: 'POST' })
  if (!res.ok) throw new Error(`Reload failed (${res.status})`)
}

async function errorMessage(res: Response): Promise<string> {
  switch (res.status) {
    case 400: return 'Invalid input — check number (digits only, 2–10 chars), display name, and secret (min 6 chars).'
    case 403: return 'You do not have permission to do this.'
    case 404: return 'Extension not found.'
    case 409: return 'An extension with that number already exists.'
    case 500: return 'Server error. The reload may have failed — check the backend log.'
    default:  return `Request failed (${res.status})`
  }
}

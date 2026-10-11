import { apiFetch } from './client'

export interface User {
  id: string
  username: string
  email: string | null
  role: string
  extension: string | null
  active: boolean
  created_at: string
}

export interface UserInput {
  username: string
  email?: string | null
  password: string
  role: 'admin' | 'supervisor'
  active: boolean
}

export interface UserUpdate {
  email?: string | null
  password?: string
  active?: boolean
}

export const PANEL_ROLES = [
  { value: 'admin',      label: 'Admin' },
  { value: 'supervisor', label: 'Supervisor' },
] as const

export async function listUsers(): Promise<User[]> {
  const res = await apiFetch('/api/admin/users')
  if (!res.ok) throw new Error(`Failed to load users (${res.status})`)
  return res.json()
}

export async function createUser(input: UserInput): Promise<User> {
  const res = await apiFetch('/api/admin/users', {
    method: 'POST',
    body: JSON.stringify(input),
  })
  if (!res.ok) throw new Error(await errorMessage(res))
  return res.json()
}

export async function updateUser(id: string, input: UserUpdate): Promise<User> {
  const res = await apiFetch(`/api/admin/users/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
  if (!res.ok) throw new Error(await errorMessage(res))
  return res.json()
}

export async function deleteUser(id: string): Promise<void> {
  const res = await apiFetch(`/api/admin/users/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(await errorMessage(res))
}

async function errorMessage(res: Response): Promise<string> {
  switch (res.status) {
    case 400:
      return 'Invalid input — username 3–32 chars (letters, digits, . _ -), password min 8 chars. You cannot deactivate the last admin.'
    case 403:
      return 'You do not have permission to do this.'
    case 404:
      return 'User not found.'
    case 409:
      return 'A user with that username already exists.'
    case 500:
      return 'Server error. Check the backend log.'
    default:
      return `Request failed (${res.status})`
  }
}

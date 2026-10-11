import { apiFetch } from './client'

export interface Queue {
  id: string
  number: string
  name: string
  strategy: string
  timeout: number
  retry: number
  maxlen: number
  acw_seconds: number
  musicclass: string
  joinempty: boolean
  leavewhenempty: boolean
  ringinuse: boolean
  active: boolean
  members: string[]
}

export interface QueueInput {
  number: string
  name: string
  strategy?: string
  timeout?: number
  retry?: number
  maxlen?: number
  acw_seconds?: number
  musicclass?: string
  joinempty?: boolean
  leavewhenempty?: boolean
  ringinuse?: boolean
  active?: boolean
  members?: string[]
}

export const STRATEGIES = [
  'ringall',
  'leastrecent',
  'fewestcalls',
  'random',
  'rrmemory',
  'rrordered',
  'linear',
  'wrandom',
] as const

export async function listQueues(): Promise<Queue[]> {
  const res = await apiFetch('/api/admin/queues')
  if (!res.ok) throw new Error(`Failed to list queues (${res.status})`)
  return res.json()
}

export async function createQueue(input: QueueInput): Promise<Queue> {
  const res = await apiFetch('/api/admin/queues', {
    method: 'POST',
    body: JSON.stringify(input),
  })
  if (!res.ok) {
    const msg = await errorMessage(res)
    throw new Error(msg)
  }
  return res.json()
}

export async function updateQueue(id: string, input: QueueInput): Promise<Queue> {
  const res = await apiFetch(`/api/admin/queues/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
  if (!res.ok) {
    const msg = await errorMessage(res)
    throw new Error(msg)
  }
  return res.json()
}

export async function deleteQueue(id: string): Promise<void> {
  const res = await apiFetch(`/api/admin/queues/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(`Failed to delete (${res.status})`)
}

export async function reloadAsterisk(): Promise<void> {
  const res = await apiFetch('/api/admin/queues/reload', { method: 'POST' })
  if (!res.ok) throw new Error(`Reload failed (${res.status})`)
}

async function errorMessage(res: Response): Promise<string> {
  switch (res.status) {
    case 400:
      return 'Invalid input — check number (4 digits, starts with 2), name, timeout, retry, and members.'
    case 403:
      return 'You do not have permission to do this.'
    case 404:
      return 'Queue not found.'
    case 409:
      return 'A queue with that number already exists.'
    case 500:
      return 'Server error. The reload may have failed — check the backend log.'
    default:
      return `Request failed (${res.status})`
  }
}

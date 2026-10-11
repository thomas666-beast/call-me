import { apiFetch } from './client'

export interface Agent {
  id: string
  username: string
  extension_number: string | null
  display_name: string | null
  sip_secret: string | null
  queue_numbers: string[]
  active: boolean
}

export interface AgentInput {
  extension_number: string
  display_name: string
  sip_secret: string
  password: string
  queue_numbers: string[]
  active: boolean
}

export interface AgentUpdate {
  display_name?: string
  sip_secret?: string
  password?: string
  queue_numbers?: string[]
  active?: boolean
}

export interface AgentCredentials {
  displayName: string
  loginUsername: string
  loginPassword: string
  sipUsername: string | null
  sipPassword: string | null
}

export async function listAgents(): Promise<Agent[]> {
  const res = await apiFetch('/api/admin/agents')
  if (!res.ok) throw new Error(`Failed to load agents (${res.status})`)
  return res.json()
}

export async function getAgent(id: string): Promise<Agent> {
  const res = await apiFetch(`/api/admin/agents/${id}`)
  if (!res.ok) throw new Error(`Failed to load agent (${res.status})`)
  return res.json()
}

export async function createAgent(input: AgentInput): Promise<Agent> {
  const res = await apiFetch('/api/admin/agents', {
    method: 'POST',
    body: JSON.stringify(input),
  })
  if (!res.ok) throw new Error(await errorMessage(res))
  return res.json()
}

export async function updateAgent(id: string, input: AgentUpdate): Promise<Agent> {
  const res = await apiFetch(`/api/admin/agents/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
  if (!res.ok) throw new Error(await errorMessage(res))
  return res.json()
}

export async function deleteAgent(id: string): Promise<void> {
  const res = await apiFetch(`/api/admin/agents/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(`Failed to delete agent (${res.status})`)
}

async function errorMessage(res: Response): Promise<string> {
  switch (res.status) {
    case 400:
      return 'Invalid input — check extension number (digits only), display name, and secrets (min 6 chars).'
    case 403:
      return 'You do not have permission to do this.'
    case 404:
      return 'Agent not found.'
    case 409:
      return 'An agent with that extension already exists.'
    case 500:
      return 'Server error. Check the backend log.'
    default:
      return `Request failed (${res.status})`
  }
}

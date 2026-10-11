import { apiFetch } from './client'

export interface Settings {
  recordings_retention_days: number
}

export interface SettingsUpdate {
  recordings_retention_days?: number
}

export async function getSettings(): Promise<Settings> {
  const res = await apiFetch('/api/admin/settings')
  if (!res.ok) throw new Error(`Failed to load settings (${res.status})`)
  return res.json()
}

export async function updateSettings(input: SettingsUpdate): Promise<Settings> {
  const res = await apiFetch('/api/admin/settings', {
    method: 'PUT',
    body: JSON.stringify(input),
  })
  if (!res.ok) throw new Error(`Failed to save settings (${res.status})`)
  return res.json()
}

export interface CleanupReport {
  deleted: number
  skipped: number
  retention_days: number
}

export async function triggerRecordingsCleanup(): Promise<CleanupReport> {
  const res = await apiFetch('/api/admin/recordings/cleanup', { method: 'POST' })
  if (!res.ok) throw new Error(`Cleanup failed (${res.status})`)
  return res.json()
}

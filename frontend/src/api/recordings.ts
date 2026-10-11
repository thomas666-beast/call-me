import { apiFetch, absoluteUrl } from './client'

export interface Recording {
  id: string
  uniqueid: string
  linkedid: string
  path: string
  filename: string
  format: string
  size_bytes: number
  duration_secs: number | null
  call_date: string | null
  src: string
  dst: string
  queue_number: string | null
  agent_extension: string | null
  indexed_at: string
  file_mtime: string | null
  last_played_at: string | null
  last_played_by: string | null
}

export interface RecordingFilters {
  from?: string
  to?: string
  src?: string
  dst?: string
  limit?: number
  offset?: number
}

export async function listRecordings(filters: RecordingFilters = {}): Promise<Recording[]> {
  const params = new URLSearchParams()
  if (filters.from) params.set('from', filters.from)
  if (filters.to) params.set('to', filters.to)
  if (filters.src) params.set('src', filters.src)
  if (filters.dst) params.set('dst', filters.dst)
  params.set('limit', String(filters.limit ?? 50))
  params.set('offset', String(filters.offset ?? 0))

  const res = await apiFetch(`/api/admin/recordings?${params.toString()}`)
  if (!res.ok) throw new Error(`Failed to load recordings (${res.status})`)
  return res.json()
}

/**
 * Fetch a short-lived signed URL for the audio file.
 *
 * The URL is valid for ~5 minutes and can be used directly in an
 * `<audio src="...">` tag without any Authorization header — which the
 * browser would refuse to send anyway.
 */
export async function getPlaybackUrl(recordingId: string): Promise<string> {
  const res = await apiFetch(`/api/admin/recordings/${recordingId}/play-token`, {
    method: 'POST',
  })
  if (!res.ok) throw new Error(`Failed to get playback URL (${res.status})`)
  const data = await res.json()
  return absoluteUrl(data.url)
}

export async function findRecordingForCall(
  uniqueid: string,
  linkedid: string,
): Promise<Recording | null> {
  const recordings = await listRecordings({ limit: 200 })
  return (
    recordings.find((r) => r.uniqueid === uniqueid || r.linkedid === linkedid) ?? null
  )
}

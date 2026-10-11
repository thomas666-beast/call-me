import { apiFetch } from './client'

export interface Cdr {
  id: number
  calldate: string
  start_time: string | null
  answer_time: string | null
  end_time: string | null
  clid: string
  src: string
  dst: string
  dcontext: string
  channel: string
  dstchannel: string
  lastapp: string
  lastdata: string
  duration: number
  billsec: number
  disposition: string
  amaflags: number
  accountcode: string
  uniqueid: string
  userfield: string
  peeraccount: string
  linkedid: string
  sequence: number
  kind: 'queue' | 'internal' | 'external' | 'outbound' | null
  outcome: string | null
  answered_by: string | null
  agent_extension: string | null
}

export interface CdrFilters {
  from?: string
  to?: string
  src?: string
  dst?: string
  disposition?: string
  include_internal?: boolean
  limit?: number
  offset?: number
}

export interface Cel {
  id: number
  eventtype: string
  eventtime: string
  userdeftype: string
  cid_name: string
  cid_num: string
  cid_ani: string
  cid_rdnis: string
  cid_dnid: string
  exten: string
  context: string
  channame: string
  appname: string
  appdata: string
  amaflags: number
  accountcode: string
  peer: string
  userfield: string
  uniqueid: string
  linkedid: string
  extra: string
}

export interface QueueEvent {
  event: string
  time: string
  queue_number: string
  agent_extension: string | null
  data: string
}

export interface CallSummary {
  kind: 'queue' | 'internal' | 'external' | 'outbound'
  outcome: string | null
  answered_by: string | null
  queue_number: string | null
}

export interface CdrWithCel {
  cdr: Cdr
  cel: Cel[]
  queue_events: QueueEvent[]
  summary: CallSummary
}

export interface CdrFilters {
  from?: string
  to?: string
  src?: string
  dst?: string
  disposition?: string
  limit?: number
  offset?: number
}

export async function listCdr(filters: CdrFilters = {}): Promise<Cdr[]> {
  const params = new URLSearchParams()
  if (filters.from) params.set('from', filters.from)
  if (filters.to) params.set('to', filters.to)
  if (filters.src) params.set('src', filters.src)
  if (filters.dst) params.set('dst', filters.dst)
  if (filters.disposition) params.set('disposition', filters.disposition)
  if (filters.include_internal) params.set('include_internal', 'true')
  params.set('limit', String(filters.limit ?? 50))
  params.set('offset', String(filters.offset ?? 0))

  const res = await apiFetch(`/api/admin/cdr?${params.toString()}`)
  if (!res.ok) throw new Error(`Failed to load CDRs (${res.status})`)
  return res.json()
}

export async function getCdr(id: number): Promise<CdrWithCel> {
  const res = await apiFetch(`/api/admin/cdr/${id}`)
  if (!res.ok) throw new Error(`Failed to load CDR (${res.status})`)
  return res.json()
}

export const DISPOSITIONS = [
  'ANSWERED',
  'NO ANSWER',
  'BUSY',
  'FAILED',
  'CONGESTION',
] as const

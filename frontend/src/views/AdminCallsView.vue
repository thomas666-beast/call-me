<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { listCdr, DISPOSITIONS, type Cdr } from '../api/cdr'
import { useDirectoryStore } from '../stores/directory'

const router = useRouter()
const directory = useDirectoryStore()

const cdr = ref<Cdr[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

const from = ref('')
const to = ref('')
const src = ref('')
const dst = ref('')
const disposition = ref('')
const includeInternal = ref(true)
const limit = ref(20)
const offset = ref(0)

const hasFilters = computed(() =>
  from.value || to.value || src.value || dst.value || disposition.value
)

async function load(resetOffset = false) {
  if (resetOffset) offset.value = 0
  loading.value = true
  error.value = null
  try {
    cdr.value = await listCdr({
      from: from.value ? new Date(from.value).toISOString() : undefined,
      to: to.value ? new Date(to.value).toISOString() : undefined,
      src: src.value || undefined,
      dst: dst.value || undefined,
      disposition: disposition.value || undefined,
      include_internal: includeInternal.value,
      limit: limit.value,
      offset: offset.value,
    })
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

function clearFilters() {
  from.value = ''
  to.value = ''
  src.value = ''
  dst.value = ''
  disposition.value = ''
  includeInternal.value = true
  load(true)
}

function nextPage() {
  offset.value += limit.value
  load(false)
}

function prevPage() {
  offset.value = Math.max(0, offset.value - limit.value)
  load(false)
}

function openCall(row: Cdr) {
  router.push(`/admin/calls/${row.id}`)
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString('en-GB', { hour12: false })
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`
  const m = Math.floor(secs / 60)
  const s = secs % 60
  return `${m}m ${s}s`
}

function dispositionClass(d: string): string {
  switch (d) {
    case 'ANSWERED':   return 'ok'
    case 'NO ANSWER':  return 'warn'
    case 'BUSY':       return 'warn'
    case 'FAILED':     return 'err'
    case 'CONGESTION': return 'err'
    default:           return 'neutral'
  }
}

interface OutcomeDisplay {
  label: string
  color: 'green' | 'red' | 'amber' | 'grey'
}

function outcomeDisplay(row: Cdr): OutcomeDisplay | null {
  if (!row.outcome) return null

  if (row.outcome.startsWith('answered:')) {
    return { label: 'Answered', color: 'green' }
  }

  switch (row.outcome) {
    case 'abandoned': return { label: 'Abandoned', color: 'red' }
    case 'missed':    return { label: 'Missed', color: 'amber' }
    case 'cancelled': return { label: 'Cancelled', color: 'grey' }
    case 'timed_out': return { label: 'Timed out', color: 'grey' }
    default:          return { label: row.outcome, color: 'grey' }
  }
}

function kindBadge(row: Cdr): { label: string; color: string } | null {
  switch (row.kind) {
    case 'queue':    return { label: 'QUEUE', color: 'blue' }
    case 'internal': return { label: 'INTERNAL', color: 'grey' }
    case 'external': return { label: 'EXTERNAL', color: 'purple' }
    case 'outbound': return { label: 'OUTBOUND', color: 'amber' }
    default:         return null
  }
}

/// Show the agent extension resolved to a name when possible.
/// Format: "Alice Cooper (1001)" or "1001" or null.
function agentLabel(row: Cdr): string | null {
  if (!row.agent_extension) return null
  const name = directory.nameFor(row.agent_extension)
  return name ? `${name} (${row.agent_extension})` : row.agent_extension
}

/// Show the other party in the Caller/Destination cell.
function partyLabel(num: string): { name: string | null; number: string } {
  return { name: directory.nameFor(num), number: num }
}

onMounted(async () => {
  await directory.ensureLoaded()
  await load()
})
</script>

<template>
  <div>
    <div class="header">
      <h2>Call Log</h2>
    </div>

    <div class="filters">
      <label>
        From
        <input v-model="from" type="datetime-local" />
      </label>
      <label>
        To
        <input v-model="to" type="datetime-local" />
      </label>
      <label>
        Caller
        <input v-model="src" placeholder="1001 or number" />
      </label>
      <label>
        Destination
        <input v-model="dst" placeholder="2000 or number" />
      </label>
      <label>
        Disposition
        <select v-model="disposition">
          <option value="">All</option>
          <option v-for="d in DISPOSITIONS" :key="d" :value="d">{{ d }}</option>
        </select>
      </label>
      <label class="checkbox">
        <input v-model="includeInternal" type="checkbox" @change="load(true)" />
        Show internal calls
      </label>
      <div class="filter-actions">
        <button class="primary" @click="load(true)">Apply</button>
        <button v-if="hasFilters" @click="clearFilters">Clear</button>
      </div>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="loading" class="empty">Loading…</div>
    <div v-else-if="cdr.length === 0" class="empty">
      No calls match the current filters.
    </div>

    <table v-else>
      <thead>
        <tr>
          <th>Date / Time</th>
          <th>Kind</th>
          <th>Caller</th>
          <th>Destination</th>
          <th>Agent</th>
          <th>Result</th>
          <th>Talk</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in cdr" :key="row.id">
          <td class="mono small">{{ formatDate(row.calldate) }}</td>

          <td>
            <span
              v-if="kindBadge(row)"
              class="kind-badge"
              :class="kindBadge(row)!.color"
            >
              {{ kindBadge(row)!.label }}
            </span>
          </td>

          <td class="mono">
            <template v-if="partyLabel(row.src).name">
              <span class="display-name">{{ partyLabel(row.src).name }}</span>
              <span class="number">({{ row.src }})</span>
            </template>
            <span v-else>{{ row.src || '—' }}</span>
          </td>

          <td class="mono">
            <template v-if="partyLabel(row.dst).name">
              <span class="display-name">{{ partyLabel(row.dst).name }}</span>
              <span class="number">({{ row.dst }})</span>
            </template>
            <span v-else>{{ row.dst || '—' }}</span>
          </td>

          <td class="mono">
            <span v-if="agentLabel(row)" class="agent-name">
              {{ agentLabel(row) }}
            </span>
            <span v-else class="muted">—</span>
          </td>

          <td>
            <span
              v-if="outcomeDisplay(row)"
              class="outcome"
              :class="outcomeDisplay(row)!.color"
            >
              {{ outcomeDisplay(row)!.label }}
            </span>
            <span
              v-else
              class="badge"
              :class="dispositionClass(row.disposition)"
            >
              {{ row.disposition || '—' }}
            </span>
          </td>

          <td class="mono small">{{ formatDuration(row.billsec) }}</td>

          <td class="row-actions">
            <button @click="openCall(row)">Details</button>
          </td>
        </tr>
      </tbody>
    </table>

    <div v-if="cdr.length > 0 || offset > 0" class="pagination">
      <button :disabled="offset === 0" @click="prevPage">Previous</button>
      <span class="page-info">Showing {{ offset + 1 }}–{{ offset + cdr.length }}</span>
      <button :disabled="cdr.length < limit" @click="nextPage">Next</button>
    </div>
  </div>
</template>

<style scoped>
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
h2 { margin: 0; }

.filters {
  display: flex; flex-wrap: wrap; gap: 12px; align-items: flex-end;
  padding: 12px; background: #1e293b; border-radius: 6px; margin-bottom: 16px;
}
.filters label {
  display: flex; flex-direction: column; font-size: 12px; color: #94a3b8; gap: 4px;
}
.filters input, .filters select {
  padding: 6px 8px; background: #0f172a; border: 1px solid #334155;
  color: #e2e8f0; border-radius: 4px; font-size: 13px; min-width: 150px;
}
.filters .checkbox {
  flex-direction: row; align-items: center; gap: 6px; padding-bottom: 8px;
}
.filters .checkbox input { min-width: 0; width: auto; }
.filter-actions { display: flex; gap: 8px; }
.filter-actions button {
  padding: 6px 14px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.filter-actions .primary { background: #0369a1; }
.filter-actions .primary:hover { background: #0284c7; }

table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { text-align: left; padding: 10px 8px; border-bottom: 1px solid #1e293b; }
th {
  color: #94a3b8; font-weight: 500; font-size: 12px;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; color: #cbd5e1; }
.display-name { color: #e2e8f0; }
.number { color: #64748b; font-size: 12px; margin-left: 6px; }
.muted { color: #475569; }
.agent-name { color: #a78bfa; }

.kind-badge {
  display: inline-block; padding: 2px 8px; border-radius: 3px;
  font-size: 10px; font-weight: 600; letter-spacing: 0.5px;
}
.kind-badge.blue   { background: #1e3a8a; color: #93c5fd; }
.kind-badge.grey   { background: #334155; color: #cbd5e1; }
.kind-badge.purple { background: #581c87; color: #d8b4fe; }
.kind-badge.amber  { background: #78350f; color: #fcd34d; }

.outcome { padding: 2px 8px; border-radius: 3px; font-size: 12px; }
.outcome.green { background: #14532d; color: #86efac; }
.outcome.red   { background: #7f1d1d; color: #fecaca; }
.outcome.amber { background: #78350f; color: #fcd34d; }
.outcome.grey  { background: #334155; color: #cbd5e1; }

.badge {
  display: inline-block; padding: 2px 8px; border-radius: 10px; font-size: 11px;
}
.badge.ok      { background: #14532d; color: #86efac; }
.badge.warn    { background: #78350f; color: #fcd34d; }
.badge.err     { background: #7f1d1d; color: #fecaca; }
.badge.neutral { background: #334155; color: #cbd5e1; }

.row-actions { text-align: right; }
.row-actions button {
  padding: 4px 12px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer; font-size: 12px;
}
.row-actions button:hover { background: #475569; }

.empty { padding: 40px; text-align: center; color: #64748b; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }

.pagination {
  display: flex; justify-content: space-between; align-items: center;
  margin-top: 16px; font-size: 13px; color: #94a3b8;
}
.pagination button {
  padding: 6px 14px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.page-info { font-family: ui-monospace, monospace; }
</style>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getCdr, type CdrWithCel } from '../api/cdr'
import { findRecordingForCall, type Recording } from '../api/recordings'
import { useDirectoryStore } from '../stores/directory'
import CelTimeline from '../components/CelTimeline.vue'
import QueueEventsTimeline from '../components/QueueEventsTimeline.vue'
import AudioPlayer from '../components/AudioPlayer.vue'

const route = useRoute()
const router = useRouter()
const directory = useDirectoryStore()

const data = ref<CdrWithCel | null>(null)
const recording = ref<Recording | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)

const srcName = computed(() =>
  data.value ? directory.nameFor(data.value.cdr.src) : null
)
const dstName = computed(() =>
  data.value ? directory.nameFor(data.value.cdr.dst) : null
)

/// Human-friendly summary of the call. Feeds the banner at the top.
interface BannerContent {
  color: 'green' | 'red' | 'amber' | 'grey' | 'blue' | 'purple'
  title: string
  subtitle: string
}

const banner = computed<BannerContent | null>(() => {
  if (!data.value) return null
  const s = data.value.summary
  const call = data.value.cdr

  const durationText =
    call.billsec > 0
      ? `Talk ${formatDuration(call.billsec)}`
      : `Duration ${formatDuration(call.duration)}`

  // Queue calls
  if (s.kind === 'queue') {
    const queueLabel = s.queue_number ? `Queue ${s.queue_number}` : 'Queue'

    if (s.outcome && s.outcome.startsWith('answered:')) {
      const ext = s.outcome.slice('answered:'.length)
      const name = directory.nameFor(ext)
      const agentLabel = name ? `${name} (${ext})` : ext
      return {
        color: 'green',
        title: `Answered by ${agentLabel}`,
        subtitle: `${queueLabel} · ${durationText}`,
      }
    }

    if (s.outcome === 'abandoned') {
      return {
        color: 'red',
        title: 'Abandoned',
        subtitle: `${queueLabel} · Caller hung up before any agent answered`,
      }
    }

    if (s.outcome === 'missed') {
      return {
        color: 'amber',
        title: 'Missed',
        subtitle: `${queueLabel} · Agents rang but did not answer`,
      }
    }

    if (s.outcome === 'cancelled') {
      return {
        color: 'grey',
        title: 'Cancelled',
        subtitle: `${queueLabel} · Caller hung up while ringing`,
      }
    }

    if (s.outcome === 'timed_out') {
      return {
        color: 'grey',
        title: 'Timed out',
        subtitle: `${queueLabel} · Caller waited too long and left`,
      }
    }

    return {
      color: 'grey',
      title: 'Queue call',
      subtitle: `${queueLabel} · No answer recorded`,
    }
  }

  if (s.kind === 'internal') {
    return {
      color: 'grey',
      title: 'Internal call',
      subtitle: `Extension to extension · ${durationText}`,
    }
  }

  if (s.kind === 'outbound') {
    return {
      color: 'blue',
      title: 'Outbound call',
      subtitle: `Placed by ${srcName.value ?? call.src} · ${durationText}`,
    }
  }

  return {
    color: 'purple',
    title: 'External call',
    subtitle: durationText,
  }
})

async function load() {
  loading.value = true
  error.value = null
  const id = Number(route.params.id)
  if (!Number.isFinite(id)) {
    error.value = 'Invalid call ID'
    loading.value = false
    return
  }
  try {
    data.value = await getCdr(id)
    recording.value = await findRecordingForCall(
      data.value.cdr.uniqueid,
      data.value.cdr.linkedid
    )
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

function formatDate(iso: string | null): string {
  if (!iso) return '—'
  return new Date(iso).toLocaleString('en-GB', { hour12: false })
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`
  const m = Math.floor(secs / 60)
  const s = secs % 60
  return `${m}m ${s}s`
}

onMounted(async () => {
  await directory.ensureLoaded()
  await load()
})
</script>

<template>
  <div>
    <div class="header">
      <button class="back" @click="router.push('/admin/calls')">← Back to call log</button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
    <div v-if="loading" class="empty">Loading…</div>

    <div v-else-if="data">
      <h2>Call #{{ data.cdr.id }}</h2>

      <!-- Outcome banner -->
      <div v-if="banner" class="banner" :class="banner.color">
        <div class="banner-dot"></div>
        <div class="banner-content">
          <div class="banner-title">{{ banner.title }}</div>
          <div class="banner-subtitle">{{ banner.subtitle }}</div>
        </div>
      </div>

      <!-- Detail fields -->
      <div class="card">
        <div class="grid">
          <div class="field">
            <span class="label">Time</span>
            <span class="value">{{ formatDate(data.cdr.calldate) }}</span>
          </div>

          <div class="field">
            <span class="label">Caller</span>
            <span class="value mono" v-if="srcName">
              {{ srcName }}
              <span class="number">({{ data.cdr.src }})</span>
            </span>
            <span class="value mono" v-else>{{ data.cdr.src || '—' }}</span>
          </div>

          <div class="field">
            <span class="label">Destination</span>
            <span class="value mono" v-if="dstName">
              {{ dstName }}
              <span class="number">({{ data.cdr.dst }})</span>
            </span>
            <span class="value mono" v-else>{{ data.cdr.dst || '—' }}</span>
          </div>

          <div v-if="data.summary.answered_by" class="field">
            <span class="label">Answered by</span>
            <span class="value mono">
              {{ directory.nameFor(data.summary.answered_by) ?? '—' }}
              <span class="number">({{ data.summary.answered_by }})</span>
            </span>
          </div>

          <div class="field">
            <span class="label">Result</span>
            <span class="value">{{ data.cdr.disposition }}</span>
          </div>
          <div class="field">
            <span class="label">Duration</span>
            <span class="value mono">{{ formatDuration(data.cdr.duration) }}</span>
          </div>
          <div class="field">
            <span class="label">Talk time</span>
            <span class="value mono">{{ formatDuration(data.cdr.billsec) }}</span>
          </div>
          <div class="field">
            <span class="label">Last app</span>
            <span class="value mono">{{ data.cdr.lastapp || '—' }}</span>
          </div>
          <div class="field">
            <span class="label">Unique ID</span>
            <span class="value mono small">{{ data.cdr.uniqueid }}</span>
          </div>
        </div>
      </div>

      <h3>Queue events ({{ data.queue_events.length }})</h3>
      <QueueEventsTimeline :events="data.queue_events" />

      <h3>Recording</h3>
      <AudioPlayer :recording="recording" />

      <h3>Event timeline ({{ data.cel.length }} events)</h3>
      <CelTimeline :events="data.cel" />
    </div>
  </div>
</template>

<style scoped>
.header { margin-bottom: 16px; }
.back {
  padding: 6px 12px; background: transparent; color: #94a3b8;
  border: 1px solid #334155; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.back:hover { color: #e2e8f0; background: #1e293b; }

h2 { margin: 0 0 16px 0; }
h3 { margin: 24px 0 12px 0; color: #cbd5e1; font-size: 13px; text-transform: uppercase; letter-spacing: 0.5px; }

.banner {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 20px;
  border-radius: 6px;
  margin-bottom: 16px;
  border-left: 4px solid;
}
.banner.green  { background: #052e16; border-left-color: #22c55e; }
.banner.red    { background: #3f1d1d; border-left-color: #ef4444; }
.banner.amber  { background: #3b2a10; border-left-color: #f59e0b; }
.banner.grey   { background: #1e293b; border-left-color: #64748b; }
.banner.blue   { background: #0c1f3a; border-left-color: #3b82f6; }
.banner.purple { background: #2a103f; border-left-color: #a855f7; }

.banner-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}
.banner.green  .banner-dot { background: #22c55e; }
.banner.red    .banner-dot { background: #ef4444; }
.banner.amber  .banner-dot { background: #f59e0b; }
.banner.grey   .banner-dot { background: #64748b; }
.banner.blue   .banner-dot { background: #3b82f6; }
.banner.purple .banner-dot { background: #a855f7; }

.banner-content { flex: 1; }
.banner-title {
  font-size: 15px;
  font-weight: 600;
  color: #f1f5f9;
  margin-bottom: 2px;
}
.banner-subtitle {
  font-size: 12px;
  color: #94a3b8;
  font-family: ui-monospace, monospace;
}

.card {
  background: #1e293b; border-radius: 6px; padding: 16px;
  border: 1px solid #334155;
}
.grid {
  display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}
.field { display: flex; flex-direction: column; gap: 4px; }
.label {
  font-size: 11px; color: #94a3b8;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.value { color: #e2e8f0; font-size: 14px; }
.number { color: #64748b; margin-left: 6px; font-size: 12px; }
.mono { font-family: ui-monospace, monospace; }
.small { font-size: 12px; }

.empty { padding: 40px; text-align: center; color: #64748b; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }
</style>

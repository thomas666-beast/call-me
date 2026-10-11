<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  getSettings,
  updateSettings,
  triggerRecordingsCleanup,
  type Settings,
  type CleanupReport,
} from '../api/settings'

const settings = ref<Settings | null>(null)
const retentionDays = ref(0)
const loading = ref(true)
const saving = ref(false)
const error = ref<string | null>(null)
const toast = ref<string | null>(null)
const cleanupReport = ref<CleanupReport | null>(null)

const PRESETS = [
  { value: 0,    label: 'Keep forever' },
  { value: 30,   label: '30 days' },
  { value: 90,   label: '90 days' },
  { value: 180,  label: '180 days' },
  { value: 365,  label: '1 year' },
  { value: 730,  label: '2 years' },
]

async function load() {
  loading.value = true
  error.value = null
  try {
    settings.value = await getSettings()
    retentionDays.value = settings.value.recordings_retention_days
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

async function save() {
  saving.value = true
  error.value = null
  try {
    const updated = await updateSettings({ recordings_retention_days: retentionDays.value })
    settings.value = updated
    showToast('Settings saved')
  } catch (e: any) {
    error.value = e.message
  } finally {
    saving.value = false
  }
}

async function runCleanup() {
  error.value = null
  cleanupReport.value = null
  try {
    cleanupReport.value = await triggerRecordingsCleanup()
    showToast('Cleanup complete')
  } catch (e: any) {
    error.value = e.message
  }
}

function showToast(msg: string) {
  toast.value = msg
  setTimeout(() => { toast.value = null }, 3000)
}

onMounted(load)
</script>

<template>
  <div>
    <div class="header">
      <h2>Settings</h2>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
    <div v-if="loading" class="empty">Loading…</div>

    <div v-else class="sections">
      <section class="card">
        <h3>Recordings retention</h3>
        <p class="desc">
          Delete call recordings older than the selected period.
          Files on disk and their database entries are removed.
          Set to <strong>Keep forever</strong> to disable automatic deletion.
        </p>

        <label class="field">
          Retention policy
          <select v-model.number="retentionDays">
            <option v-for="p in PRESETS" :key="p.value" :value="p.value">
              {{ p.label }}
            </option>
          </select>
        </label>

        <div class="actions">
          <button class="primary" :disabled="saving" @click="save">
            {{ saving ? 'Saving…' : 'Save settings' }}
          </button>
        </div>
      </section>

      <section class="card">
        <h3>Manual cleanup</h3>
        <p class="desc">
          Run the retention task immediately instead of waiting for the daily
          schedule. Useful after changing the policy or to free disk space.
        </p>

        <div class="actions">
          <button @click="runCleanup">Run cleanup now</button>
        </div>

        <div v-if="cleanupReport" class="report">
          <div><span class="label">Retention</span> {{ cleanupReport.retention_days }} days</div>
          <div><span class="label">Deleted</span> {{ cleanupReport.deleted }}</div>
          <div><span class="label">Skipped</span> {{ cleanupReport.skipped }}</div>
        </div>
      </section>
    </div>

    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<style scoped>
.header { margin-bottom: 24px; }
h2 { margin: 0; }

.sections { display: flex; flex-direction: column; gap: 20px; }

.card {
  background: #1e293b;
  border: 1px solid #334155;
  border-radius: 6px;
  padding: 20px;
}
h3 {
  margin: 0 0 8px 0;
  color: #e2e8f0;
  font-size: 15px;
}
.desc {
  color: #94a3b8;
  font-size: 13px;
  line-height: 1.5;
  margin: 0 0 16px 0;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
  color: #94a3b8;
  max-width: 320px;
  margin-bottom: 16px;
}
select {
  padding: 8px;
  background: #0f172a;
  border: 1px solid #334155;
  color: #e2e8f0;
  border-radius: 4px;
  font-size: 14px;
}

.actions { display: flex; gap: 8px; }
button {
  padding: 8px 16px;
  background: #334155;
  color: #e2e8f0;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
button:hover { background: #475569; }
button.primary { background: #0369a1; }
button.primary:hover:not(:disabled) { background: #0284c7; }
button:disabled { opacity: 0.5; cursor: not-allowed; }

.report {
  margin-top: 16px;
  padding: 12px;
  background: #0f172a;
  border-radius: 4px;
  font-family: ui-monospace, monospace;
  font-size: 13px;
  color: #cbd5e1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.report .label {
  display: inline-block;
  min-width: 100px;
  color: #64748b;
}

.empty { padding: 40px; text-align: center; color: #64748b; }
.error { color: #f87171; font-size: 13px; margin-bottom: 12px; }

.toast {
  position: fixed;
  bottom: 24px;
  right: 24px;
  background: #0369a1;
  color: white;
  padding: 12px 20px;
  border-radius: 6px;
  font-size: 13px;
  z-index: 200;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
</style>

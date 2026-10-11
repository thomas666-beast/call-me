<script setup lang="ts">
import { ref, watch } from 'vue'
import { getPlaybackUrl, type Recording } from '../api/recordings'

const props = defineProps<{
  recording: Recording | null
}>()

const audioSrc = ref<string | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

async function loadUrl() {
  error.value = null
  audioSrc.value = null

  if (!props.recording) return

  loading.value = true
  try {
    audioSrc.value = await getPlaybackUrl(props.recording.id)
  } catch (e: any) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

watch(() => props.recording?.id, loadUrl, { immediate: true })

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${(n / 1024 / 1024).toFixed(1)} MB`
}
</script>

<template>
  <div class="player">
    <div v-if="!recording" class="empty">
      No recording available for this call.
    </div>

    <div v-else-if="loading" class="empty">
      Preparing playback…
    </div>

    <div v-else-if="error" class="error-box">
      Could not load recording: {{ error }}
    </div>

    <div v-else>
      <audio
        v-if="audioSrc"
        controls
        preload="metadata"
        :src="audioSrc"
        class="audio"
      ></audio>
      <div class="meta">
        <span class="filename">{{ recording.filename }}</span>
        <span class="size">{{ formatBytes(recording.size_bytes) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.player { margin: 12px 0; }
.audio { width: 100%; }
.meta {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #94a3b8;
  margin-top: 6px;
  font-family: ui-monospace, monospace;
}
.filename { color: #38bdf8; }
.empty {
  padding: 20px;
  text-align: center;
  color: #64748b;
  font-size: 13px;
  border: 1px dashed #334155;
  border-radius: 4px;
}
.error-box {
  padding: 16px;
  background: #3f1d1d;
  border: 1px solid #7f1d1d;
  color: #fecaca;
  border-radius: 4px;
  font-size: 13px;
}
</style>

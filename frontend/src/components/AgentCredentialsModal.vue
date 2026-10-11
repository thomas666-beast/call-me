<script setup lang="ts">
import { computed, ref } from 'vue'
import type { AgentCredentials } from '../api/agents'

const props = defineProps<{
  open: boolean
  credentials: AgentCredentials | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const copied = ref(false)

const fullText = computed(() => {
  if (!props.credentials) return ''
  const c = props.credentials
  const lines: string[] = [
    `Agent: ${c.displayName}`,
    '',
    '--- Web panel login ---',
    'URL:      http://<your-server>',
    `Username: ${c.loginUsername}`,
    `Password: ${c.loginPassword}`,
  ]
  if (c.sipUsername && c.sipPassword) {
    lines.push(
      '',
      '--- Softphone (SIP) ---',
      `Username: ${c.sipUsername}`,
      `Password: ${c.sipPassword}`,
      'Server:   <your-asterisk-ip>',
      'Port:     5060 (UDP)',
    )
  }
  return lines.join('\n')
})

async function copyAll() {
  if (!fullText.value) return
  try {
    await navigator.clipboard.writeText(fullText.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch (e) {
    console.error('Copy failed', e)
  }
}
</script>

<template>
  <div v-if="open && credentials" class="overlay" @click.self="emit('close')">
    <div class="modal">
      <h3>Credentials for {{ credentials.displayName }}</h3>

      <p class="warn">
        Copy these now — the password will not be shown again.
        If it's lost, use <strong>Reset password</strong> from the agent list.
      </p>

      <div class="section">
        <div class="section-title">Web panel login</div>
        <div class="cred">
          <span class="label">Username</span>
          <span class="value mono">{{ credentials.loginUsername }}</span>
        </div>
        <div class="cred">
          <span class="label">Password</span>
          <span class="value mono highlight">{{ credentials.loginPassword }}</span>
        </div>
      </div>

      <div
        v-if="credentials.sipUsername && credentials.sipPassword"
        class="section"
      >
        <div class="section-title">Softphone (SIP)</div>
        <div class="cred">
          <span class="label">Username</span>
          <span class="value mono">{{ credentials.sipUsername }}</span>
        </div>
        <div class="cred">
          <span class="label">Password</span>
          <span class="value mono highlight">{{ credentials.sipPassword }}</span>
        </div>
      </div>

      <div class="actions">
        <button @click="copyAll">{{ copied ? 'Copied!' : 'Copy all' }}</button>
        <button class="primary" @click="emit('close')">Done</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.7);
  display: flex; align-items: center; justify-content: center; z-index: 150;
}
.modal {
  background: #1e293b; padding: 24px; border-radius: 8px;
  min-width: 480px; max-width: 560px; border: 1px solid #334155;
}
h3 { margin: 0 0 12px 0; }
.warn {
  color: #fcd34d; font-size: 12px; margin: 0 0 20px 0;
  padding: 10px 12px; background: #3b2a10; border-radius: 4px;
  border-left: 3px solid #f59e0b; line-height: 1.5;
}
.section { margin-bottom: 16px; }
.section-title {
  font-size: 11px; color: #94a3b8;
  text-transform: uppercase; letter-spacing: 0.5px;
  margin-bottom: 8px;
}
.cred {
  display: flex; justify-content: space-between; align-items: center;
  padding: 10px 12px; background: #0f172a; border-radius: 4px;
  margin-bottom: 6px;
}
.label { font-size: 12px; color: #94a3b8; }
.value { font-size: 14px; color: #e2e8f0; }
.mono { font-family: ui-monospace, monospace; }
.highlight { color: #86efac; font-weight: 500; }
.actions {
  display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px;
}
.actions button {
  padding: 8px 16px; background: #334155; color: #e2e8f0;
  border: none; border-radius: 4px; cursor: pointer;
}
.actions button:hover { background: #475569; }
.actions .primary { background: #0369a1; }
.actions .primary:hover { background: #0284c7; }
</style>

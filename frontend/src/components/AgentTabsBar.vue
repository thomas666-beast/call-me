<script setup lang="ts">
import { computed } from 'vue'
import { useAgentTabsStore } from '../stores/agentTabs'
import { useAgentStore } from '../stores/agent'

const tabsStore = useAgentTabsStore()
const agent = useAgentStore()

function isTabLive(callid: string | null): boolean {
  if (!callid) return false
  return agent.activeCall?.callid === callid
}

function tabTitle(id: string): string {
  const tab = tabsStore.tabs.find((t) => t.id === id)
  if (!tab) return ''
  const name = tab.contact?.contact.display_name
  if (name && tab.phoneNumber) return `${name} (${tab.phoneNumber})`
  if (name) return name
  if (tab.phoneNumber) return tab.phoneNumber
  return 'Call'
}

function onClick(id: string) {
  tabsStore.setActive(id)
}

function onClose(id: string, event: MouseEvent) {
  event.stopPropagation()
  if (tabsStore.isDirty(id)) {
    if (!confirm('This tab has unsaved text. Discard and close?')) return
  }
  tabsStore.closeTab(id)
}

const tabs = computed(() => tabsStore.tabs)
const activeId = computed(() => tabsStore.activeTabId)
</script>

<template>
  <div v-if="tabs.length > 0" class="tab-bar">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="tab"
      :class="{ active: tab.id === activeId }"
      @click="onClick(tab.id)"
    >
      <span v-if="isTabLive(tab.callid)" class="live-dot" title="On call"></span>
      <span class="title">{{ tabTitle(tab.id) }}</span>
      <span class="close" @click="onClose(tab.id, $event)">×</span>
    </button>
  </div>
</template>

<style scoped>
.tab-bar {
  display: flex;
  gap: 2px;
  border-bottom: 1px solid #1e293b;
  margin-bottom: 16px;
  overflow-x: auto;
}
.tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px 8px 14px;
  background: #1e293b;
  color: #94a3b8;
  border: none;
  border-bottom: 2px solid transparent;
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
  font-family: inherit;
}
.tab:hover { background: #334155; color: #e2e8f0; }
.tab.active {
  background: #0f172a;
  color: #38bdf8;
  border-bottom-color: #38bdf8;
}
.title { max-width: 180px; overflow: hidden; text-overflow: ellipsis; }
.close {
  padding: 0 4px;
  color: #64748b;
  font-size: 16px;
  line-height: 1;
  border-radius: 3px;
}
.close:hover { background: #7f1d1d; color: #fecaca; }
.live-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #22c55e;
  box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7);
  animation: pulse 1.6s infinite;
}
@keyframes pulse {
  0%   { box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7); }
  70%  { box-shadow: 0 0 0 10px rgba(34, 197, 94, 0); }
  100% { box-shadow: 0 0 0 0 rgba(34, 197, 94, 0); }
}
</style>

<script setup lang="ts">
defineProps<{
  offset: number
  limit: number
  count: number
}>()

const emit = defineEmits<{
  (e: 'prev'): void
  (e: 'next'): void
}>()

function onPrev() { emit('prev') }
function onNext() { emit('next') }
</script>

<template>
  <div class="pagination">
    <button :disabled="offset === 0" @click="onPrev">Previous</button>
    <span class="info">
      Showing {{ count === 0 ? 0 : offset + 1 }}–{{ offset + count }}
    </span>
    <button :disabled="count < limit" @click="onNext">Next</button>
  </div>
</template>

<style scoped>
.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
  font-size: 13px;
  color: #94a3b8;
}
.pagination button {
  padding: 6px 14px;
  background: #334155;
  color: #e2e8f0;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination button:not(:disabled):hover { background: #475569; }
.info { font-family: ui-monospace, monospace; }
</style>
